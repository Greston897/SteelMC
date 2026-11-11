//! Vanilla `net.minecraft.world.entity.monster.cubemob.MagmaCube`.
//!
//! Fire-immune cube mob sharing the slime family sizing with `Slime`, but with
//! its own damage, armor, jump and sound formulas: it hurts even at size one, has
//! three armor per size, hops higher the bigger it is, rises quickly out of lava,
//! and waits four times as long between hops.
//!
//! In 26.3 `MagmaCube` no longer extends `Slime`; both extend `AbstractCubeMob`.
//! Steel keeps the shared formulas in [`super::cube_mob`].
//!
//! Deviations (documented in `PARITY-26.3-RU.md`):
//! * natural spawning (nether wastes, basalt deltas, fortress, bastion, soul sand
//!   valley) is not wired: the entity type is registered without a spawn rule and
//!   only spawners and commands create it;
//! * the hopping goals are collapsed into `custom_server_ai_step` instead of a
//!   `MagmaCubeKeepOnJumpingGoal` / `SlimeRandomDirectionGoal` pair;
//! * `AbstractCubeMob.push` contact damage against iron golems is not wired,
//!   because Steel has no entity-push hook a mob can react to. The player path
//!   (`playerTouch`) is wired through [`Entity::player_touch`];
//! * lava particles and the squish animation are client-side in vanilla.

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
use steel_registry::vanilla_entity_data::MagmaCubeEntityData;
use steel_registry::vanilla_entities;
use steel_registry::{sound_events, vanilla_attributes, vanilla_fluid_tags};
use steel_utils::locks::SyncMutex;
use steel_utils::nbt::NbtValueInput as _;
use steel_utils::{BlockPos, BlockStateId, Downcast as _, DowncastType, DowncastTypeKey, Identifier};

use super::cube_mob;
use crate::entity::ai::goal::{
    FloatGoal, HurtByTargetGoal, LookAtPlayerGoal, NearestAttackableTargetGoal,
    RandomLookAroundGoal, WaterAvoidingRandomStrollGoal,
};
use crate::entity::damage::DamageSource;
use crate::entity::{
    DefaultEntityPush, Entity, EntityBase, EntityBaseLoad, EntityEventSource,
    EntityMovementEmission, EntityPose,
    EntitySpawnReason, EntitySyncedData, LivingEntity, LivingEntityBase, Mob, MobBase, Monster,
    PathfinderMob, SharedEntity, SpawnGroupData, sync_dirty_mob_effects,
};
use crate::physics::MoveResult;
use crate::player::Player;
use crate::world::World;

const DEFAULT_STEP_HEIGHT: f32 = 0.6;

/// Vanilla `MagmaCube.getJumpDelay` multiplies the slime delay by four.
const JUMP_DELAY_MULTIPLIER: i32 = 4;

/// Vanilla `MagmaCube` jump boost per size step.
const JUMP_BOOST_PER_SIZE: f32 = 0.1;

/// Vanilla `MagmaCube` lava jump base velocity.
const LAVA_JUMP_BASE: f32 = 0.22;

/// Vanilla `MagmaCube` lava jump boost per size step.
const LAVA_JUMP_PER_SIZE: f32 = 0.05;

/// Vanilla `MagmaCube`.
#[entity_behavior(class = "MagmaCube")]
pub struct MagmaCubeEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    entity_data: SyncMutex<MagmaCubeEntityData>,
    jump_delay: AtomicI32,
}

// SAFETY: This key is owned by Steel and uniquely identifies `MagmaCubeEntity`.
unsafe impl DowncastType for MagmaCubeEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/magma_cube");
}

impl MagmaCubeEntity {
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

        let mut entity_data = MagmaCubeEntityData::new();
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

        let cube = Self {
            base,
            entity_type,
            living_base,
            mob_base,
            entity_data: SyncMutex::new(entity_data),
            jump_delay: AtomicI32::new(
                cube_mob::initial_jump_delay() * JUMP_DELAY_MULTIPLIER,
            ),
        };
        cube.set_size(cube_mob::MIN_SIZE, true);
        cube
    }

    /// Vanilla cube size, always within `1..=127`.
    #[must_use]
    pub fn size(&self) -> i32 {
        *self.entity_data.lock().abstract_cube_mob().id_size.get()
    }

    /// Sets the synced size and every value vanilla derives from it.
    ///
    /// `reset_health` mirrors vanilla's `setSize(size, resetHealth)`: it is
    /// `false` while loading from disk so a saved cube keeps its health.
    pub fn set_size(&self, size: i32, reset_health: bool) {
        // Vanilla uses the unclamped argument for armor, so a negative or huge
        // requested size still yields the armor its `setSize` argument implies.
        let armor = cube_mob::magma_cube_armor(size);
        let size = size.clamp(cube_mob::MIN_SIZE, cube_mob::MAX_SIZE);
        let changed = {
            let mut data = self.entity_data.lock();
            if *data.abstract_cube_mob().id_size.get() == size {
                false
            } else {
                data.abstract_cube_mob_mut().id_size.set(size);
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
                cube_mob::magma_cube_attack_damage(size),
            );
            attributes.set_base_value(vanilla_attributes::ARMOR, armor);
        }

        if reset_health {
            self.set_health(self.get_max_health());
        }
    }

    /// Vanilla `AbstractCubeMob.isTiny()`.
    #[must_use]
    pub fn is_tiny(&self) -> bool {
        self.size() <= cube_mob::MIN_SIZE
    }

    /// Vanilla `MagmaCube.canDealDamage`: unlike a slime, even size one hurts.
    #[must_use]
    pub fn can_deal_damage(&self) -> bool {
        !self.is_no_ai()
    }

    /// Vanilla `MagmaCube.getAttackDamage()`.
    #[must_use]
    pub fn attack_damage(&self) -> f64 {
        cube_mob::magma_cube_attack_damage(self.size())
    }

    /// Rolls vanilla's natural-spawn size and applies it.
    pub fn randomize_size(&self, special_multiplier: f32) {
        self.set_size(cube_mob::random_spawn_size(special_multiplier), true);
    }

    /// Vanilla `getSoundVolume()`, inherited from `AbstractCubeMob`.
    #[must_use]
    fn sound_volume(&self) -> f32 {
        if self.is_tiny() {
            0.4
        } else {
            1.0
        }
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        sync_dirty_mob_effects(&self.living_base, &self.entity_data);
    }

    fn jump_sound(&self) -> SoundEventRef {
        &sound_events::ENTITY_MAGMA_CUBE_JUMP
    }

    fn squish_sound(&self) -> SoundEventRef {
        if self.is_tiny() {
            &sound_events::ENTITY_MAGMA_CUBE_SQUISH_SMALL
        } else {
            &sound_events::ENTITY_MAGMA_CUBE_SQUISH
        }
    }

    /// Vanilla `MagmaCube.jumpFromGround`: the size-scaled boost.
    fn do_jump(&self) {
        self.default_jump_from_ground();
        let boosted = self.velocity().y + f64::from(self.size() as f32 * JUMP_BOOST_PER_SIZE);
        self.set_velocity(DVec3::new(self.velocity().x, boosted, self.velocity().z));
        self.mark_velocity_sync();
        self.play_sound(self.jump_sound(), self.sound_volume(), 1.0);
    }

    /// Vanilla `MagmaCube.jumpInLiquid`: a strong lava rise.
    fn jump_in_lava(&self) {
        let rise = LAVA_JUMP_BASE + self.size() as f32 * LAVA_JUMP_PER_SIZE;
        let velocity = self.velocity();
        self.set_velocity(DVec3::new(velocity.x, f64::from(rise), velocity.z));
        self.mark_velocity_sync();
    }

    fn split_on_death(&self) {
        let width = self.bounding_box().width();
        let _ = cube_mob::split_on_death(self, self.size(), width, &|child, size| {
            if let Some(cube) = child.as_entity_event_source().downcast_ref::<Self>() {
                cube.set_size(size, true);
            }
        });
    }

    /// Vanilla `AbstractCubeMob.dealDamage`, reached through `playerTouch`.
    fn deal_touch_damage(self: &Arc<Self>, target: &dyn Entity) {
        let Some(target_living) = target.as_living_entity() else {
            return;
        };
        if !self.can_deal_damage()
            || !Entity::is_alive(&**self)
            || self.is_allied_to(target)
            || !self.is_within_melee_attack_range(target_living)
            || !self.has_line_of_sight(target)
        {
            return;
        }
        let Some(world) = self.level() else {
            return;
        };
        let shared = Arc::clone(self).into_shared_entity();
        let source = self.mob_attack_damage_source(&ItemStack::empty(), &shared);
        if target.hurt(&world, &source, self.attack_damage() as f32) {
            let pitch = (rand::random::<f32>() - rand::random::<f32>()) * 0.2 + 1.0;
            self.play_sound(&sound_events::ENTITY_SLIME_ATTACK, 1.0, pitch);
        }
    }
}

impl DefaultEntityPush for MagmaCubeEntity {}

impl Entity for MagmaCubeEntity {
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
        // Vanilla stores `Size` zero-based, so a size-one cube is saved as `0`.
        nbt.insert("Size", self.size() - 1);
        self.save_mob(nbt);
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        if let Some(size) = nbt.int_value("Size") {
            self.set_size(size.wrapping_add(1), false);
        }
        self.load_mob(nbt);
    }
}

impl LivingEntity for MagmaCubeEntity {
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
            &sound_events::ENTITY_MAGMA_CUBE_HURT_SMALL
        } else {
            &sound_events::ENTITY_MAGMA_CUBE_HURT
        })
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(if self.is_tiny() {
            &sound_events::ENTITY_MAGMA_CUBE_DEATH_SMALL
        } else {
            &sound_events::ENTITY_MAGMA_CUBE_DEATH
        })
    }

    /// Vanilla `AbstractCubeMob.getBaseExperienceReward` is the raw size.
    fn base_experience_reward(&self) -> i32 {
        self.size()
    }

    fn on_death(&self, _source: &DamageSource) {
        self.split_on_death();
    }

    /// Vanilla `MagmaCube.jumpInLiquid`: lava gives a much stronger rise than
    /// the shared water jump.
    fn jump_in_liquid(&self, fluid_tag: &Identifier) {
        if fluid_tag == &vanilla_fluid_tags::FluidTag::LAVA {
            self.jump_in_lava();
        } else {
            self.set_velocity(self.velocity() + DVec3::new(0.0, f64::from(0.04_f32), 0.0));
        }
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

impl Mob for MagmaCubeEntity {
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
        if spawn_reason == EntitySpawnReason::Natural {
            self.randomize_size(1.0);
        }
        self.finalize_spawn_mob_base(world, spawn_reason, group_data)
    }

    fn custom_server_ai_step(&self) {
        if !self.on_ground() || self.is_passenger() {
            return;
        }
        if self.jump_delay.fetch_sub(1, Ordering::Relaxed) > 0 {
            return;
        }
        let aggressive = self.target().is_some();
        self.jump_delay.store(
            cube_mob::next_jump_delay(aggressive) * JUMP_DELAY_MULTIPLIER,
            Ordering::Relaxed,
        );
        self.do_jump();
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }
}

impl Monster for MagmaCubeEntity {}

impl PathfinderMob for MagmaCubeEntity {}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::sync::Weak;

    use glam::DVec3;
    use simdnbt::borrow::read_compound;
    use simdnbt::owned::NbtCompound;
    use steel_registry::{init_vanilla_registry, vanilla_attributes, vanilla_entities};
    use steel_utils::nbt::NbtValueInput as _;

    use super::MagmaCubeEntity;
    use crate::entity::{Entity, LivingEntity};

    fn cube() -> MagmaCubeEntity {
        init_vanilla_registry();
        MagmaCubeEntity::new(&vanilla_entities::MAGMA_CUBE, 1, DVec3::ZERO, Weak::new())
    }

    #[test]
    fn size_drives_health_armor_damage_and_fire_immunity() {
        let cube = cube();
        assert_eq!(cube.size(), 1);
        // Unlike a slime, even a size-one magma cube hurts.
        assert!(cube.can_deal_damage());
        assert!(cube.fire_immune());
        assert!((cube.attack_damage() - 3.0).abs() < 1e-4);

        cube.set_size(4, true);
        assert_eq!(cube.size(), 4);
        assert!((cube.get_max_health() - 16.0).abs() < 1e-4);
        assert!((cube.attack_damage() - 6.0).abs() < 1e-4);
        let armor = cube
            .attributes()
            .lock()
            .get_value(vanilla_attributes::ARMOR)
            .expect("magma cubes always have armor");
        assert!((armor - 12.0).abs() < 1e-4);
        assert!((cube.base_experience_reward() as f32 - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn size_is_clamped_to_the_vanilla_range() {
        let cube = cube();
        cube.set_size(0, true);
        assert_eq!(cube.size(), 1);
        cube.set_size(i32::MAX, true);
        assert_eq!(cube.size(), 127);
    }

    #[test]
    fn saved_size_is_zero_based_and_keeps_health() {
        let cube = cube();
        cube.set_size(3, true);
        cube.set_health(5.0);

        let mut nbt = NbtCompound::new();
        cube.save_additional(&mut nbt);
        assert_eq!(nbt.int_value("Size"), Some(2), "vanilla saves size - 1");

        let loaded =
            MagmaCubeEntity::new(&vanilla_entities::MAGMA_CUBE, 2, DVec3::ZERO, Weak::new());
        let mut bytes = Vec::new();
        nbt.write(&mut bytes);
        let borrowed = read_compound(&mut Cursor::new(bytes.as_slice()))
            .unwrap_or_else(|error| panic!("reborrow failed: {error}"));
        loaded.load_additional((&borrowed).into());

        assert_eq!(loaded.size(), 3);
        assert!((loaded.get_health() - 5.0).abs() < 1e-4);
    }
}
