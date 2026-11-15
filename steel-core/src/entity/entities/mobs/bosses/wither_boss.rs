//! Vanilla `net.minecraft.world.entity.boss.wither.WitherBoss`.
//!
//! A three-headed boss that hovers, fires wither skulls and regenerates while
//! its invulnerability timer runs.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * the boss bar and the three skull volleys are not implemented; the spawn
//!   phase (220 protected ticks, escalating regeneration, closing blast) and the
//!   synced target/invulnerability fields are.
use std::iter::empty;
use std::sync::Weak;

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_entity_data::WitherEntityData;
use steel_registry::{level_events, sound_events, vanilla_attributes};
use steel_utils::locks::SyncMutex;
use steel_utils::{DowncastType, DowncastTypeKey};

use crate::entity::ai::goal::{
    FloatGoal, HurtByTargetGoal, LookAtPlayerGoal, MeleeAttackGoal, NearestAttackableTargetGoal,
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
        let living_base = LivingEntityBase::new(entity_type);
        let mob_base = MobBase::new();

        let mut entity_data = WitherEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, FloatGoal::new(&mob_base));
            goal_selector.add_goal(2, MeleeAttackGoal::new(1.0, false));
            goal_selector.add_goal(7, WaterAvoidingRandomStrollGoal::new(1.0));
            goal_selector.add_goal(8, LookAtPlayerGoal::new(8.0));
            goal_selector.add_goal(9, RandomLookAroundGoal::new());

            let mut target_selector = mob_base.target_selector().lock();
            target_selector.add_goal(1, HurtByTargetGoal::new().set_alert_others(empty()));
            target_selector.add_goal(
                2,
                NearestAttackableTargetGoal::new_for_players(true, |_, _| true),
            );
        }

        Self {
            base,
            entity_type,
            living_base,
            mob_base,
            entity_data: SyncMutex::new(entity_data),
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
    /// 220 protected ticks at full health, healing while the counter runs.
    pub fn begin_spawn_phase(&self) {
        self.set_invulnerable_ticks(SPAWN_INVULNERABLE_TICKS);
        self.set_health(MAX_HEALTH);
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
        // `heal(10.0F)` every 10 ticks is what keeps a spawning wither alive.
        // Vanilla heals before it decrements, so the first tick already heals.
        if ticks % 10 == 0 {
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
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
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
    }

    fn ai_step(&self, entity: &SharedEntity) -> Option<MoveResult> {
        if self.invulnerable_ticks() > 0 {
            return None;
        }
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
        self.begin_spawn_phase();
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

impl PathfinderMob for WitherBossEntity {}

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
        assert_eq!(wither.get_health(), wither.get_max_health());
        assert_eq!(wither.get_health(), MAX_HEALTH);
    }
}
