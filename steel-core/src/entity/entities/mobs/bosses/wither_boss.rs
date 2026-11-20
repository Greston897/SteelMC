//! Vanilla `net.minecraft.world.entity.boss.wither.WitherBoss`.
//!
//! A three-headed boss that hovers, fires wither skulls and regenerates while
//! its invulnerability timer runs.
//!
//! Server-side ranged combat, independent side-head targets and tracker-scoped
//! boss bar; spawn protection remains separate from the combat AI.
use std::iter::empty;
use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_entity_data::WitherEntityData;
use steel_registry::{TaggedRegistryExt as _, level_events, sound_events, vanilla_attributes};
use steel_utils::locks::SyncMutex;
use steel_utils::nbt::NbtValueInput as _;
use steel_utils::{DowncastType, DowncastTypeKey};

use crate::entity::ai::goal::{
    FloatGoal, HurtByTargetGoal, LookAtPlayerGoal, RangedAttackGoal, NearestAttackableTargetGoal,
    RandomLookAroundGoal, WaterAvoidingRandomStrollGoal,
};
use crate::entity::damage::DamageSource;
use crate::entity::{
    DefaultEntityPush, Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, EntityPose,
    EntitySpawnReason, EntitySyncedData, LivingEntity, LivingEntityBase, Mob, MobBase, Monster,
    PathfinderMob, SharedEntity, SpawnGroupData, sync_dirty_mob_effects,
};
use crate::physics::MoveResult;
use crate::world::{ExplosionInteraction, ExplosionOptions, World};

const DEFAULT_STEP_HEIGHT: f32 = 0.6;

/// Vanilla `WitherBoss` spawns with `DATA_ID_INV` set to 220 ticks.
pub const SPAWN_INVULNERABLE_TICKS: i32 = 220;

/// Vanilla `WitherBoss` blast power when the spawn sequence finishes.
pub const SPAWN_EXPLOSION_POWER: f32 = 7.0;

/// Vanilla caps the wither's health at this value.
const MAX_HEALTH: f32 = 300.0;

/// Vanilla `heal(10.0F)` every 10 ticks while `DATA_ID_INV` counts down.
const SPAWN_HEAL_PER_STEP: f32 = 10.0;

/// Vanilla `net.minecraft.world.entity.boss.wither.WitherBoss`.
#[entity_behavior(class = "WitherBoss")]
pub struct WitherBossEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    entity_data: SyncMutex<WitherEntityData>,
    head_timers: SyncMutex<([i32; 2], [i32; 2])>,
    last_bar_progress: SyncMutex<f32>,
    destroy_blocks_tick: SyncMutex<i32>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `WitherBossEntity`.
unsafe impl DowncastType for WitherBossEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/wither");
}

impl WitherBossEntity {
    /// Creates a new entity of this vanilla type.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Loads a saved entity of this vanilla type.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self::new_with_base(
            EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
        )
    }

    fn new_with_base(base: EntityBase, entity_type: EntityTypeRef) -> Self {
        base.set_no_gravity(true);
        let living_base = LivingEntityBase::new(entity_type);
        let mob_base = MobBase::new();
        mob_base.set_xp_reward(50);

        let mut entity_data = WitherEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, FloatGoal::new(&mob_base));
            goal_selector.add_goal(2, RangedAttackGoal::new(1.0, 40, 20.0));
            goal_selector.add_goal(7, WaterAvoidingRandomStrollGoal::new(1.0));
            goal_selector.add_goal(8, LookAtPlayerGoal::new(8.0));
            goal_selector.add_goal(9, RandomLookAroundGoal::new());

            let mut target_selector = mob_base.target_selector().lock();
            target_selector.add_goal(1, HurtByTargetGoal::new().set_alert_others(empty()));
            target_selector.add_goal(
                2,
                NearestAttackableTargetGoal::new(true, |target, _| {
                    !steel_registry::REGISTRY.entity_types.is_in_tag(target.entity_type(), &steel_registry::vanilla_entity_type_tags::EntityTypeTag::WITHER_FRIENDS)
                }),
            );
        }

        Self {
            base,
            entity_type,
            living_base,
            mob_base,
            entity_data: SyncMutex::new(entity_data),
            head_timers: SyncMutex::new(([0; 2], [0; 2])),
            last_bar_progress: SyncMutex::new(-1.0),
            destroy_blocks_tick: SyncMutex::new(0),
        }
    }

    /// Full bar pairing is sent only to players actually tracking this boss.
    pub(crate) fn boss_bar_packet(&self) -> steel_protocol::packets::game::CBossEvent {
        use steel_protocol::packets::game::{BossBarColor, BossBarOverlay, CBossEvent};
        CBossEvent::add(self.uuid(), self.display_name(), self.bar_progress(), BossBarColor::Purple, BossBarOverlay::Progress, 1)
    }

    fn bar_progress(&self) -> f32 {
        if self.invulnerable_ticks() > 0 {
            1.0 - self.invulnerable_ticks() as f32 / SPAWN_INVULNERABLE_TICKS as f32
        } else { self.get_health() / self.get_max_health() }
    }

    fn sync_boss_bar(&self) {
        let Some(world) = self.level() else { return; };
        let progress = self.bar_progress().clamp(0.0, 1.0);
        let mut last = self.last_bar_progress.lock();
        if progress.to_bits() != last.to_bits() {
            world.broadcast_to_entity_trackers(self.id(), steel_protocol::packets::game::CBossEvent::update_progress(self.uuid(), progress), None);
            *last = progress;
        }
        if self.tick_count() % 20 == 0 {
            world.broadcast_to_entity_trackers(self.id(), steel_protocol::packets::game::CBossEvent::update_name(self.uuid(), self.display_name()), None);
        }
    }

    fn set_head_target(&self, head: usize, id: i32) {
        let mut data = self.entity_data.lock();
        match head {
            0 => data.wither_boss_mut().target_a.set(id),
            1 => data.wither_boss_mut().target_b.set(id),
            _ => data.wither_boss_mut().target_c.set(id),
        }
    }

    fn fire_skull(&self, shooter: &SharedEntity, head: usize, target: DVec3, dangerous: bool) {
        use crate::entity::{Projectile, next_entity_id};
        use crate::entity::entities::objects::projectiles::WitherSkullEntity;
        let Some(world) = self.level() else { return; };
        let scale = f64::from(self.get_scale());
        let angle = (self.y_body_rot() + 180.0 * head.saturating_sub(1) as f32).to_radians();
        let position = self.position() + if head == 0 { DVec3::new(0.0, 3.0 * scale, 0.0) }
            else { DVec3::new(f64::from(steel_math::trig::cos(angle)) * 1.3 * scale, 2.2 * scale, f64::from(steel_math::trig::sin(angle)) * 1.3 * scale) };
        let skull = Arc::new(WitherSkullEntity::new(&steel_registry::vanilla_entities::WITHER_SKULL, next_entity_id(), position, Arc::downgrade(&world)));
        skull.set_owner_entity(Some(shooter));
        skull.set_dangerous(dangerous);
        skull.set_no_gravity(true);
        skull.shoot(target - position, 0.1, 0.0);
        let projectile: SharedEntity = skull;
        if world.try_add_entity(projectile).is_ok() && !self.is_silent() {
            world.level_event(level_events::SOUND_WITHER_BOSS_SHOOT, self.block_position(), 0, None);
        }
    }

    fn tick_side_heads(&self, shooter: &SharedEntity) {
        use steel_registry::vanilla_entity_type_tags::EntityTypeTag;
        let Some(world) = self.level() else { return; };
        self.set_head_target(0, self.target().map_or(0, |target| target.id()));
        let mut timers = self.head_timers.lock();
        for side in 0..2 {
            if self.tick_count() < timers.0[side] { continue; }
            timers.0[side] = self.tick_count() + rand::random_range(10..20);
            if matches!(world.difficulty(), steel_utils::types::Difficulty::Normal | steel_utils::types::Difficulty::Hard) {
                let previous = timers.1[side];
                timers.1[side] += 1;
                if previous > 15 {
                    let destination = self.position() + DVec3::new(rand::random_range(-10.0..10.0), rand::random_range(-5.0..5.0), rand::random_range(-10.0..10.0));
                    self.fire_skull(shooter, side + 2, destination, true);
                    timers.1[side] = 0;
                }
            }
            let current = world.get_entity_by_id(self.alternative_target(side + 1));
            if let Some(target) = current.filter(|target| target.as_living_entity().is_some_and(|living| Mob::can_attack(self, living))
                && !steel_registry::REGISTRY.entity_types.is_in_tag(target.entity_type(), &EntityTypeTag::WITHER_FRIENDS)
                && target.position().distance_squared(self.position()) <= 900.0 && self.has_line_of_sight(target.as_ref())) {
                self.fire_skull(shooter, side + 2, target.position() + DVec3::Y * (target.bounding_box().height() * 0.5), false);
                timers.0[side] = self.tick_count() + rand::random_range(40..60);
                timers.1[side] = 0;
            } else {
                self.set_head_target(side + 1, 0);
                let targets: Vec<_> = world.get_entities_in_aabb(&self.bounding_box().inflate_xyz(20.0, 8.0, 20.0)).into_iter()
                    .filter(|target| target.id() != self.id() && target.as_living_entity().is_some_and(|living| Mob::can_attack(self, living))
                        && !steel_registry::REGISTRY.entity_types.is_in_tag(target.entity_type(), &EntityTypeTag::WITHER_FRIENDS) && self.has_line_of_sight(target.as_ref())).collect();
                if !targets.is_empty() {
                    self.set_head_target(side + 1, targets[rand::random_range(0..targets.len())].id());
                }
            }
        }
        if self.tick_count() % 20 == 0 { self.heal(1.0); }
        let destroy = {
            let mut countdown = self.destroy_blocks_tick.lock();
            if *countdown > 0 { *countdown -= 1; *countdown == 0 } else { false }
        };
        if destroy && world.get_game_rule(&steel_registry::vanilla_game_rules::MOB_GRIEFING) {
            use steel_registry::blocks::block_state_ext::BlockStateExt as _;
            use crate::world::LevelReader as _;
            let center = self.block_position();
            let width = (self.bounding_box().width() * 0.5 + 1.0).floor() as i32;
            let height = self.bounding_box().height().floor() as i32;
            let mut destroyed = false;
            for x in -width..=width { for z in -width..=width { for y in 0..=height {
                let pos = steel_utils::BlockPos::new(center.x() + x, center.y() + y, center.z() + z);
                let state = world.get_block_state(pos);
                if !state.is_air() && !steel_registry::REGISTRY.blocks.is_in_tag(state.get_block(), &steel_registry::vanilla_block_tags::BlockTag::WITHER_IMMUNE) {
                    destroyed = world.destroy_block_by_entity(pos, true, self) || destroyed;
                }
            } } }
            if destroyed { world.level_event(level_events::SOUND_WITHER_BLOCK_BREAK, center, 0, None); }
        }
    }

    /// Vanilla `WitherBoss.getInvulnerableTicks` (`DATA_ID_INV`).
    #[must_use]
    pub fn invulnerable_ticks(&self) -> i32 {
        *self.entity_data.lock().wither_boss().id_inv.get()
    }

    /// Sets the vanilla invulnerability timer.
    pub fn set_invulnerable_ticks(&self, ticks: i32) {
        self.entity_data.lock().wither_boss_mut().id_inv.set(ticks);
    }

    /// Vanilla `WitherBoss.getAlternativeTarget(head)`; `head` is `0..=2`.
    #[must_use]
    pub fn alternative_target(&self, head: usize) -> i32 {
        let data = self.entity_data.lock();
        let boss = data.wither_boss();
        match head {
            0 => *boss.target_a.get(),
            1 => *boss.target_b.get(),
            _ => *boss.target_c.get(),
        }
    }

    /// Vanilla `WitherBoss.setInvulnerableTicks`: starts the spawn sequence.
    ///
    /// Vanilla calls this from the entity constructor, so a wither is born with
    /// 220 protected ticks at one third health, healing while the counter runs.
    pub fn begin_spawn_phase(&self) {
        self.set_invulnerable_ticks(SPAWN_INVULNERABLE_TICKS);
        self.set_health(self.get_max_health() / 3.0);
    }

    /// Vanilla `WitherBoss.customServerAiStep` spawn phase.
    ///
    /// While `DATA_ID_INV` counts down the wither neither moves nor thinks: it
    /// regenerates 10 health every 10 ticks and, on the tick the counter reaches
    /// zero, blasts 7.0 power from its eye height and roars. Returns `true`
    /// while the phase still owns the tick.
    fn tick_spawn_phase(&self, entity: &SharedEntity) -> bool {
        let ticks = self.invulnerable_ticks();
        if ticks <= 0 {
            return false;
        }

        let next = ticks - 1;
        if next <= 0 {
            // Vanilla `getEyeY()` centre, power 7.0, no fire, mob interaction.
            if let Some(world) = self.level() {
                let mut options = ExplosionOptions::new(
                    self.position(),
                    SPAWN_EXPLOSION_POWER,
                    ExplosionInteraction::Mob,
                );
                options.center.y = self.get_eye_y();
                options.source = Some(std::sync::Arc::clone(entity));
                world.explode(options);
                if !self.is_silent() {
                    world.global_level_event(
                        level_events::SOUND_WITHER_BOSS_SPAWN,
                        self.block_position(),
                        0,
                    );
                }
            }
        }

        self.set_invulnerable_ticks(next);
        // Vanilla uses entity tickCount, not the remaining invulnerability timer.
        if self.tick_count() % 10 == 0 {
            let health = self.get_health();
            if health > 0.0 && health < MAX_HEALTH {
                self.set_health((health + SPAWN_HEAL_PER_STEP).min(MAX_HEALTH));
            }
        }
        true
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        sync_dirty_mob_effects(&self.living_base, &self.entity_data);
    }
}

impl DefaultEntityPush for WitherBossEntity {}

impl Entity for WitherBossEntity {
    fn can_use_portal(&self, _ignore_passenger: bool) -> bool { false }

    fn check_despawn(&self) {
        if self.level().is_some_and(|world| world.difficulty() == steel_utils::types::Difficulty::Peaceful) {
            self.set_removed(crate::entity::RemovalReason::Discarded);
        }
        self.set_no_action_time(0);
    }

    fn is_hard_collision_relevant(&self) -> bool {
        false
    }

    fn as_default_entity_push(&self) -> Option<&dyn DefaultEntityPush> {
        Some(self)
    }

    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn movement_emission(&self) -> EntityMovementEmission {
        EntityMovementEmission::Events
    }

    fn base_tick(&self) {
        Mob::base_tick_mob(self);
        self.sync_boss_bar();
    }

    fn dimensions_for_pose(&self, _pose: EntityPose) -> EntityDimensions {
        let scale = LivingEntity::get_scale(self);
        if self.entity_type.fixed {
            self.entity_type.dimensions
        } else {
            self.entity_type.dimensions.scale(scale)
        }
    }

    fn synced_data(&self) -> Option<&dyn EntitySyncedData> {
        Some(&self.entity_data)
    }

    fn update_data_before_sync(&self) {
        self.update_dirty_mob_effect_entity_data();
    }

    fn max_up_step(&self) -> f32 {
        self.attributes()
            .lock()
            .get_value(vanilla_attributes::STEP_HEIGHT)
            .unwrap_or(f64::from(DEFAULT_STEP_HEIGHT)) as f32
    }

    fn sound_source(&self) -> SoundSource {
        Monster::sound_source_monster(self)
    }

    fn fall_sounds(&self) -> (SoundEventRef, SoundEventRef) {
        Monster::fall_sounds_monster(self)
    }

    fn swim_sound(&self) -> SoundEventRef {
        Monster::swim_sound_monster(self)
    }

    fn swim_splash_sound(&self) -> SoundEventRef {
        Monster::swim_splash_sound_monster(self)
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_mob(nbt);
        nbt.insert("Invul", self.invulnerable_ticks());
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
        self.set_invulnerable_ticks(nbt.int_value("Invul").unwrap_or(0).max(0));
    }
}

impl LivingEntity for WitherBossEntity {
    fn living_base(&self) -> &LivingEntityBase {
        &self.living_base
    }

    fn get_health(&self) -> f32 {
        *self.entity_data.lock().living_entity().health.get()
    }

    fn set_health(&self, health: f32) {
        let max_health = self.get_max_health();
        let clamped = health.clamp(0.0, max_health);
        self.entity_data
            .lock()
            .living_entity_mut()
            .health
            .set(clamped);
    }

    fn is_invulnerable_to(&self, _world: &World, source: &DamageSource) -> bool {
        use steel_registry::vanilla_damage_type_tags::DamageTypeTag;
        use steel_registry::vanilla_entity_type_tags::EntityTypeTag;
        self.default_is_invulnerable_to(source)
            || crate::enchantment_helper::is_immune_to_damage(self, source)
            || source.is(&DamageTypeTag::WITHER_IMMUNE_TO)
            || (self.invulnerable_ticks() > 0 && !source.bypasses_invulnerability())
            || source.causing_entity().is_some_and(|entity| steel_registry::REGISTRY.entity_types.is_in_tag(entity.entity_type(), &EntityTypeTag::WITHER_FRIENDS)
                || entity.entity_type() == &steel_registry::vanilla_entities::WITHER)
            || (self.get_health() <= self.get_max_health() * 0.5 && source.direct_entity().is_some_and(|entity|
                entity.is_abstract_arrow() || entity.entity_type() == &steel_registry::vanilla_entities::WIND_CHARGE))
    }

    fn hurt_server(&self, world: &World, source: &DamageSource, amount: f32) -> bool {
        if self.is_invulnerable_to(world, source) { return false; }
        {
            let mut countdown = self.destroy_blocks_tick.lock();
            if *countdown <= 0 { *countdown = 20; }
        }
        for idle in &mut self.head_timers.lock().1 { *idle += 3; }
        self.default_hurt_server(world, source, amount)
    }

    fn add_mob_effect_with_source(&self, _effect: crate::entity::MobEffectInstance, _source: Option<&dyn Entity>) -> bool { false }

    fn drop_custom_death_loot(&self, _source: &DamageSource, _killed_by_player: bool) {
        if let Some(star) = self.spawn_at_location(steel_registry::item_stack::ItemStack::new(&steel_registry::vanilla_items::NETHER_STAR), 0.0) {
            star.set_age(-6000);
        }
    }

    fn hurt_sound(&self, _source: &DamageSource) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_WITHER_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_WITHER_DEATH)
    }

    fn should_drop_experience(&self) -> bool {
        Monster::should_drop_experience_monster(self)
    }

    fn should_drop_loot(&self, world: &World) -> bool {
        Monster::should_drop_loot_monster(self, world)
    }

    fn server_ai_step(&self, entity: &SharedEntity) {
        // Vanilla keeps the whole spawn sequence inside `aiStep` and returns
        // early, so goals and navigation stay idle until the blast.
        if self.tick_spawn_phase(entity) {
            return;
        }
        Mob::mob_server_ai_step(self, entity);
        self.tick_side_heads(entity);
    }

    fn ai_step(&self, entity: &SharedEntity) -> Option<MoveResult> {
        if self.invulnerable_ticks() > 0 {
            return None;
        }
        let mut velocity = self.velocity() * DVec3::new(1.0, 0.6, 1.0);
        if let Some(target) = self.target() {
            let delta = target.position() - self.position();
            if delta.y > 0.0 || (self.get_health() > self.get_max_health() * 0.5 && delta.y > -5.0) {
                velocity.y = velocity.y.max(0.0);
                velocity.y += 0.3 - velocity.y * 0.6;
            }
            let horizontal = DVec3::new(delta.x, 0.0, delta.z);
            if horizontal.length_squared() > 9.0 {
                let direction = horizontal.normalize();
                velocity.x += direction.x * 0.3 - velocity.x * 0.6;
                velocity.z += direction.z * 0.3 - velocity.z * 0.6;
            }
        }
        self.set_velocity(velocity);
        Monster::monster_ai_step(self, entity)
    }
}

impl Mob for WitherBossEntity {
    fn mob_base(&self) -> &MobBase {
        &self.mob_base
    }

    /// Vanilla `WitherBoss` starts its spawn sequence from the constructor, so
    /// every path that creates one (spawn egg, summon, skull pyramid, spawner)
    /// begins the 220 protected ticks here.
    fn finalize_spawn(
        &self,
        world: &std::sync::Arc<World>,
        spawn_reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        if spawn_reason == EntitySpawnReason::Event { self.begin_spawn_phase(); }
        self.finalize_spawn_mob_base(world, spawn_reason, group_data)
    }

    fn tick_goal_selectors(&self, entity: &SharedEntity) {
        PathfinderMob::tick_pathfinder_goal_selectors(self, entity);
    }

    fn tick_path_navigation(&self) {
        PathfinderMob::tick_pathfinder_path_navigation(self);
    }

    fn ambient_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_WITHER_AMBIENT)
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }
}

impl Monster for WitherBossEntity {}

impl PathfinderMob for WitherBossEntity {
    fn perform_ranged_attack(&self, shooter: &SharedEntity, target: &SharedEntity, _power: f32) {
        self.fire_skull(shooter, 0, target.position() + DVec3::Y * (target.bounding_box().height() * 0.5), rand::random::<f32>() < 0.001);
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_entities};

    use super::*;

    fn wither() -> WitherBossEntity {
        init_vanilla_registry();
        WitherBossEntity::new(&vanilla_entities::WITHER, 1, DVec3::ZERO, Weak::new())
    }

    #[test]
    fn spawn_phase_matches_vanilla_numbers() {
        // Vanilla `DATA_ID_INV` starts at 220 and the closing blast has power 7.
        assert_eq!(SPAWN_INVULNERABLE_TICKS, 220);
        assert_eq!(SPAWN_EXPLOSION_POWER, 7.0);
        assert_eq!(SPAWN_HEAL_PER_STEP, 10.0);
        assert_eq!(MAX_HEALTH, 300.0);
    }

    #[test]
    fn begin_spawn_phase_sets_the_invulnerability_timer() {
        let wither = wither();
        assert_eq!(wither.invulnerable_ticks(), 0);
        wither.begin_spawn_phase();
        assert_eq!(wither.invulnerable_ticks(), SPAWN_INVULNERABLE_TICKS);
        // A spawning wither must never start at zero health, otherwise the heal
        // branch would never fire and the boss would die to its own blast.
        assert_eq!(wither.get_health(), wither.get_max_health() / 3.0);
        assert_eq!(wither.get_health(), MAX_HEALTH / 3.0);
    }
}
