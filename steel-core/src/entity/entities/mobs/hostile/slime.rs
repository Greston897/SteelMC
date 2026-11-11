//! Vanilla `net.minecraft.world.entity.monster.cubemob.Slime`.
//!
//! Size-scaled hostile cube mob: the synced `id_size` drives the hitbox, health,
//! movement speed, melee damage and experience reward. The mob hops instead of
//! walking, targets players and iron golems, damages players it touches, and
//! splits into two to four half-sized children when it dies above size one.
//!
//! In 26.3 `Slime` is no longer above `MagmaCube` in a class chain; both extend
//! `AbstractCubeMob`. Steel keeps the shared formulas in [`super::cube_mob`].
//!
//! Deviations (documented in `PARITY-26.3-RU.md`):
//! * natural spawning is not wired: `SpawnRule` hands a `&dyn LevelReader` while
//!   the slime-chunk and swamp checks need a full `&World`, so the entity type is
//!   registered without a spawn rule and only spawners and commands create it;
//! * the hopping goals are collapsed into `custom_server_ai_step` instead of a
//!   `SlimeKeepOnJumpingGoal` / `SlimeRandomDirectionGoal` pair, so a slime
//!   always hops along its current facing instead of picking a random direction;
//! * `AbstractCubeMob.push` contact damage against iron golems is not wired,
//!   because Steel has no entity-push hook a mob can react to. The player path
//!   (`playerTouch`) is wired through [`Entity::player_touch`];
//! * the squish animation is client-side state in vanilla and is not synced.

use std::iter::empty;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::item_stack::ItemStack;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_entity_data::SlimeEntityData;
use steel_registry::vanilla_entities;
use steel_registry::{sound_events, vanilla_attributes};
use steel_utils::locks::SyncMutex;
use steel_utils::{BlockPos, BlockStateId, Downcast as _, DowncastType, DowncastTypeKey};

use super::cube_mob;
use crate::entity::ai::goal::{
    FloatGoal, HurtByTargetGoal, LookAtPlayerGoal, NearestAttackableTargetGoal,
    RandomLookAroundGoal, WaterAvoidingRandomStrollGoal,
};
use crate::entity::damage::DamageSource;
use crate::entity::{
    DefaultEntityPush, Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, EntityPose,
    EntitySpawnReason, EntitySyncedData, LivingEntity, LivingEntityBase, Mob, MobBase, Monster,
    PathfinderMob, SharedEntity, SpawnGroupData, sync_dirty_mob_effects,
};
use crate::physics::MoveResult;
use crate::player::Player;
use crate::world::World;

const DEFAULT_STEP_HEIGHT: f32 = 0.6;

/// Vanilla `Slime`.
#[entity_behavior(class = "Slime")]
pub struct SlimeEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    entity_data: SyncMutex<SlimeEntityData>,
    jump_delay: AtomicI32,
}

// SAFETY: This key is owned by Steel and uniquely identifies `SlimeEntity`.
unsafe impl DowncastType for SlimeEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/slime");
}

impl SlimeEntity {
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

        let mut entity_data = SlimeEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(1, FloatGoal::new(&mob_base));
            goal_selector.add_goal(4, WaterAvoidingRandomStrollGoal::new(1.0));
            goal_selector.add_goal(5, LookAtPlayerGoal::new(8.0));
            goal_selector.add_goal(6, RandomLookAroundGoal::new());

            let mut target_selector = mob_base.target_selector().lock();
            target_selector.add_goal(1, HurtByTargetGoal::new().set_alert_others(empty()));
            target_selector.add_goal(
                2,
                NearestAttackableTargetGoal::new_for_players(true, |_, _| true),
            );
            target_selector.add_goal(
                3,
                NearestAttackableTargetGoal::new(true, |entity, _| {
                    entity.entity_type() == &vanilla_entities::IRON_GOLEM
                }),
            );
        }

        let slime = Self {
            base,
            entity_type,
            living_base,
            mob_base,
            entity_data: SyncMutex::new(entity_data),
            jump_delay: AtomicI32::new(cube_mob::initial_jump_delay()),
        };
        slime.set_size(cube_mob::MIN_SIZE, true);
        slime
    }

    /// Vanilla cube size, always within `1..=127`.
    #[must_use]
    pub fn size(&self) -> i32 {
        *self.entity_data.lock().id_size.get()
    }

    /// Sets the synced size and every value vanilla derives from it.
    ///
    /// `reset_health` mirrors vanilla's `setSize(size, resetHealth)`: it is
    /// `false` while loading from disk so a saved cube keeps its health.
    pub fn set_size(&self, size: i32, reset_health: bool) {
        let size = size.clamp(cube_mob::MIN_SIZE, cube_mob::MAX_SIZE);
        let changed = {
            let mut data = self.entity_data.lock();
            if *data.id_size.get() == size {
                false
            } else {
                data.id_size.set(size);
                true
            }
        };
        if changed {
            self.refresh_dimensions();
        }

        {
            let mut attributes = self.attributes().lock();
            attributes.set_base_value(
                vanilla_attributes::MAX_HEALTH,
                cube_mob::health_for_size(size),
            );
            attributes.set_base_value(
                vanilla_attributes::MOVEMENT_SPEED,
                cube_mob::movement_speed_for_size(size),
            );
            attributes.set_base_value(
                vanilla_attributes::ATTACK_DAMAGE,
                cube_mob::slime_attack_damage(size),
            );
        }

        if reset_health {
            self.set_health(self.get_max_health());
        }
    }

    /// Vanilla `Slime.isTiny()`.
    #[must_use]
    pub fn is_tiny(&self) -> bool {
        self.size() <= cube_mob::MIN_SIZE
    }

    /// Vanilla `Slime.canDealDamage()`: tiny slimes never hurt on contact.
    #[must_use]
    pub fn can_deal_damage(&self) -> bool {
        !self.is_tiny() && !self.is_no_ai()
    }

    /// Vanilla `Slime.getAttackDamage()`.
    #[must_use]
    pub fn attack_damage(&self) -> f64 {
        cube_mob::slime_attack_damage(self.size())
    }

    /// Rolls vanilla's natural-spawn size and applies it.
    ///
    /// `special_multiplier` is vanilla's regional-difficulty factor; Steel does
    /// not track per-chunk difficulty yet, so callers pass `1.0`.
    pub fn randomize_size(&self, special_multiplier: f32) {
        self.set_size(cube_mob::random_spawn_size(special_multiplier), true);
    }

    /// Vanilla `getSoundVolume()`: tiny cubes are quieter.
    #[must_use]
    fn sound_volume(&self) -> f32 {
        if self.is_tiny() {
            0.4
        } else {
            1.0
        }
    }

    /// Vanilla `Slime.getSoundPitch()`.
    #[must_use]
    fn sound_pitch(&self) -> f32 {
        if self.is_tiny() {
            (rand::random::<f32>() - rand::random::<f32>()) * 0.2 + 1.0
        } else {
            1.0
        }
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        sync_dirty_mob_effects(&self.living_base, &self.entity_data);
    }

    fn jump_sound(&self) -> SoundEventRef {
        if self.is_tiny() {
            &sound_events::ENTITY_SLIME_JUMP_SMALL
        } else {
            &sound_events::ENTITY_SLIME_JUMP
        }
    }

    fn squish_sound(&self) -> SoundEventRef {
        if self.is_tiny() {
            &sound_events::ENTITY_SLIME_SQUISH_SMALL
        } else {
            &sound_events::ENTITY_SLIME_SQUISH
        }
    }

    /// Vanilla `AbstractCubeMob.jumpFromGround`, as inherited by `Slime`.
    ///
    /// Only `MagmaCube` adds a size-based boost; a slime jumps normally.
    fn do_jump(&self) {
        self.default_jump_from_ground();
        self.mark_velocity_sync();
        self.play_sound(self.jump_sound(), self.sound_volume(), self.sound_pitch());
    }

    fn split_on_death(&self) {
        let width = self.bounding_box().width();
        let _ = cube_mob::split_on_death(self, self.size(), width, &|child, size| {
            if let Some(slime) = child.as_entity_event_source().downcast_ref::<Self>() {
                slime.set_size(size, true);
            }
        });
    }

    /// Vanilla `AbstractCubeMob.dealDamage`, reached through `playerTouch`.
    fn deal_touch_damage(self: &Arc<Self>, target: &dyn Entity) {
        let Some(target_living) = target.as_living_entity() else {
            return;
        };
        if !self.can_deal_damage()
            || !Entity::is_alive(self)
            || self.is_allied_to(target)
            || !self.is_within_melee_attack_range(target_living)
            || !self.has_line_of_sight(target)
        {
            return;
        }
        let Some(world) = self.level() else {
            return;
        };
        let shared: SharedEntity = Arc::clone(self);
        let source = self.mob_attack_damage_source(&ItemStack::empty(), &shared);
        if target.hurt(&world, &source, self.attack_damage() as f32) {
            // Vanilla draws the attack pitch from the cube's own random.
            let pitch = (rand::random::<f32>() - rand::random::<f32>()) * 0.2 + 1.0;
            self.play_sound(&sound_events::ENTITY_SLIME_ATTACK, 1.0, pitch);
        }
    }
}

impl DefaultEntityPush for SlimeEntity {}

impl Entity for SlimeEntity {
    /// Vanilla `AbstractCubeMob` is not pushable and is never a hard collision.
    fn is_hard_collision_relevant(&self) -> bool {
        false
    }

    fn is_pushable(&self) -> bool {
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
        let scale = self.size() as f32 * LivingEntity::get_scale(self);
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

    fn play_step_sound(&self, _pos: BlockPos, _block_state: BlockStateId) {
        self.play_sound(self.squish_sound(), self.sound_volume(), 1.0);
    }

    fn player_touch(self: Arc<Self>, player: &Arc<Player>) {
        self.deal_touch_damage(player.as_ref() as &dyn Entity);
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        // Vanilla stores `Size` zero-based, so a size-one slime is saved as `0`.
        nbt.insert("Size", self.size() - 1);
        self.save_mob(nbt);
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        if let Some(size) = nbt.int_value("Size") {
            // `reset_health = false` keeps the saved health, as vanilla does.
            self.set_size(size.wrapping_add(1), false);
        }
        self.load_mob(nbt);
    }
}

impl LivingEntity for SlimeEntity {
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
        Some(if self.is_tiny() {
            &sound_events::ENTITY_SLIME_HURT_SMALL
        } else {
            &sound_events::ENTITY_SLIME_HURT
        })
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(if self.is_tiny() {
            &sound_events::ENTITY_SLIME_DEATH_SMALL
        } else {
            &sound_events::ENTITY_SLIME_DEATH
        })
    }

    /// Vanilla `Slime.getBaseExperienceReward` is the raw size.
    fn base_experience_reward(&self) -> i32 {
        self.size()
    }

    fn on_death(&self, _source: &DamageSource) {
        self.split_on_death();
    }

    fn should_drop_experience(&self) -> bool {
        Monster::should_drop_experience_monster(self)
    }

    fn should_drop_loot(&self, world: &World) -> bool {
        Monster::should_drop_loot_monster(self, world)
    }

    fn server_ai_step(&self, entity: &SharedEntity) {
        Mob::mob_server_ai_step(self, entity);
    }

    fn ai_step(&self, entity: &SharedEntity) -> Option<MoveResult> {
        Monster::monster_ai_step(self, entity)
    }
}

impl Mob for SlimeEntity {
    fn mob_base(&self) -> &MobBase {
        &self.mob_base
    }

    fn tick_goal_selectors(&self, entity: &SharedEntity) {
        PathfinderMob::tick_pathfinder_goal_selectors(self, entity);
    }

    fn tick_path_navigation(&self) {
        PathfinderMob::tick_pathfinder_path_navigation(self);
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        spawn_reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        // Vanilla only rolls a random size for natural spawns; commands and
        // spawners keep the size the caller set.
        if spawn_reason == EntitySpawnReason::Natural {
            self.randomize_size(1.0);
        }
        self.finalize_spawn_mob_base(world, spawn_reason, group_data)
    }

    fn custom_server_ai_step(&self) {
        // Vanilla's `SlimeKeepOnJumpingGoal` only acts on the ground and never
        // while the cube is riding something.
        if !self.on_ground() || self.is_passenger() {
            return;
        }
        if self.jump_delay.fetch_sub(1, Ordering::Relaxed) > 0 {
            return;
        }
        // Vanilla halves the wait while the cube has a target.
        let aggressive = self.target().is_some();
        self.jump_delay
            .store(cube_mob::next_jump_delay(aggressive), Ordering::Relaxed);
        self.do_jump();
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }
}

impl Monster for SlimeEntity {}

impl PathfinderMob for SlimeEntity {}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::sync::{Arc, Weak};

    use glam::DVec3;
    use simdnbt::borrow::read_compound;
    use simdnbt::owned::NbtCompound;
    use steel_registry::{
        init_vanilla_registry, vanilla_attributes, vanilla_damage_types, vanilla_entities,
    };
    use steel_utils::{ChunkPos, Downcast as _, WorldAabb};

    use super::SlimeEntity;
    use crate::bootstrap::init_globals;
    use crate::entity::damage::DamageSource;
    use crate::entity::{Entity, LivingEntity, SharedEntity, next_entity_id};
    use crate::test_support::{fresh_test_world, insert_ready_full_chunk};

    fn slime() -> SlimeEntity {
        init_vanilla_registry();
        SlimeEntity::new(&vanilla_entities::SLIME, 1, DVec3::ZERO, Weak::new())
    }

    #[test]
    fn size_drives_health_speed_damage_and_hitbox() {
        let slime = slime();
        assert_eq!(slime.size(), 1);
        assert!(slime.is_tiny());
        assert!(!slime.can_deal_damage());
        assert!((slime.get_max_health() - 1.0).abs() < 1e-4);

        slime.set_size(4, true);
        assert_eq!(slime.size(), 4);
        assert!(!slime.is_tiny());
        assert!(slime.can_deal_damage());
        assert!((slime.get_max_health() - 16.0).abs() < 1e-4);
        assert!((slime.attack_damage() - 4.0).abs() < 1e-4);
        let speed = slime
            .attributes()
            .lock()
            .get_value(vanilla_attributes::MOVEMENT_SPEED)
            .expect("slimes always have a movement speed");
        assert!((speed - 0.6).abs() < 1e-4);
        // Vanilla `EntityTypes.sized(0.52F, 0.52F)` scaled by the cube size.
        assert!((slime.bounding_box().height() - 2.08).abs() < 1e-4);
        // Vanilla `Slime.getBaseExperienceReward` is the raw size.
        assert_eq!(slime.base_experience_reward(), 4);
    }

    #[test]
    fn size_is_clamped_to_the_vanilla_range() {
        let slime = slime();
        slime.set_size(0, true);
        assert_eq!(slime.size(), 1);
        slime.set_size(i32::MAX, true);
        assert_eq!(slime.size(), 127);
    }

    #[test]
    fn dying_slime_splits_into_smaller_slimes_in_the_world() {
        init_globals();
        let world_fixture = fresh_test_world("slime_split");
        let world = &world_fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));

        let parent: SharedEntity = Arc::new(SlimeEntity::new(
            &vanilla_entities::SLIME,
            next_entity_id(),
            DVec3::new(8.0, 65.0, 8.0),
            Arc::downgrade(world),
        ));
        world
            .try_add_entity(Arc::clone(&parent))
            .expect("test slime should attach to the loaded chunk");
        let slime = parent
            .downcast_ref::<SlimeEntity>()
            .expect("slime is concrete");
        slime.set_size(4, true);
        LivingEntity::die(slime, &DamageSource::environment(&vanilla_damage_types::GENERIC));

        let split_box = WorldAabb::new(4.0, 60.0, 4.0, 12.0, 70.0, 12.0);
        let children = world.get_entities_in_aabb_matching(&split_box, |entity| {
            entity.entity_type() == &vanilla_entities::SLIME && entity.id() != parent.id()
        });
        assert!(
            (2..=4).contains(&children.len()),
            "vanilla spawns two to four children, got {}",
            children.len()
        );
        for child in &children {
            let child = child
                .downcast_ref::<SlimeEntity>()
                .expect("split children are slimes");
            assert_eq!(child.size(), 2, "children are half the parent's size");
            assert!((child.get_health() - 4.0).abs() < 1e-4);
            // Vanilla `setUpSplitCube` snaps children half a block up.
            assert!((child.position().y - 65.5).abs() < 1e-4);
        }

        // A size-one slime has nothing to split into.
        let tiny: SharedEntity = Arc::new(SlimeEntity::new(
            &vanilla_entities::SLIME,
            next_entity_id(),
            DVec3::new(8.0, 65.0, 8.0),
            Arc::downgrade(world),
        ));
        world
            .try_add_entity(Arc::clone(&tiny))
            .expect("tiny test slime should attach to the loaded chunk");
        let tiny_slime = tiny.downcast_ref::<SlimeEntity>().expect("slime is concrete");
        LivingEntity::die(tiny_slime, &DamageSource::environment(&vanilla_damage_types::GENERIC));
        let after = world.get_entities_in_aabb_matching(&split_box, |entity| {
            entity.entity_type() == &vanilla_entities::SLIME
                && entity.id() != parent.id()
                && entity.id() != tiny.id()
        });
        assert_eq!(after.len(), children.len(), "size-one slimes do not split");
    }

    #[test]
    fn saved_size_is_zero_based_and_keeps_health() {
        let slime = slime();
        slime.set_size(4, true);
        slime.set_health(7.0);

        let mut nbt = NbtCompound::new();
        slime.save_additional(&mut nbt);
        assert_eq!(nbt.int_value("Size"), Some(3), "vanilla saves size - 1");

        let loaded = SlimeEntity::new(&vanilla_entities::SLIME, 2, DVec3::ZERO, Weak::new());
        let mut bytes = Vec::new();
        nbt.write(&mut bytes);
        let borrowed = read_compound(&mut Cursor::new(bytes.as_slice()))
            .unwrap_or_else(|error| panic!("reborrow failed: {error}"));
        loaded.load_additional((&borrowed).into());

        assert_eq!(loaded.size(), 4);
        assert!((loaded.get_health() - 7.0).abs() < 1e-4);
    }
}
