//! Vanilla `net.minecraft.world.entity.monster.Creeper`.
//!
//! The creeper is a melee-approach hostile mob that never actually swings: once
//! it keeps its target in sight within seven blocks it swells (`swell_dir = 1`,
//! `entity.creeper.primed` on the transition), stops navigating, and detonates
//! after `MAX_SWELL` ticks with a `3.0`-block `MOB` explosion (doubled when
//! powered). Losing the target or the line of sight deflates it again.
//!
//! Deviations (deliberate, documented in `PARITY-26.3-RU.md`):
//! * the vanilla `SwellGoal` is folded into the entity: the swell counter is
//!   advanced from `ai_step` and navigation is cancelled from
//!   `custom_server_ai_step`, which runs after the goal selectors — the visible
//!   behaviour (approach, stop, swell, deflate, explode) matches vanilla;
//! * natural spawning is not wired yet (`SpawnRule` needs `&World`), so the mob
//!   is only reachable through spawners/commands for now;
//! * `is_ignited` is synchronized but no igniter path sets it yet, and there is
//!   no lightning hook in the entity trait, so charged creepers cannot yet be
//!   produced by a strike (the `set_powered` API is in place for it).

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_entity_data::CreeperEntityData;
use steel_registry::{sound_events, vanilla_attributes, vanilla_entities};
use steel_utils::locks::SyncMutex;
use steel_utils::{DowncastType, DowncastTypeKey};

use crate::entity::ai::goal::{
    AvoidEntityGoal, FloatGoal, HurtByTargetGoal, LookAtPlayerGoal, MeleeAttackGoal,
    NearestAttackableTargetGoal, RandomLookAroundGoal, WaterAvoidingRandomStrollGoal,
};
use crate::entity::damage::DamageSource;
use crate::entity::{
    DefaultEntityPush, Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, EntityPose,
    EntitySyncedData, LivingEntity, LivingEntityBase, Mob, MobBase, Monster, PathfinderMob,
    RemovalReason, SharedEntity, sync_dirty_mob_effects,
};
use crate::physics::MoveResult;
use crate::world::{
    DefaultExplosionDamageCalculator, ExplosionInteraction, ExplosionOptions, World,
};

const DEFAULT_STEP_HEIGHT: f32 = 0.6;
/// Vanilla `Creeper.MAX_SWELL` (`30` ticks, i.e. 1.5 s).
pub const MAX_SWELL: i32 = 30;
/// Vanilla `Creeper` explosion radius (`3.0`).
pub const EXPLOSION_RADIUS: f32 = 3.0;
/// Vanilla powered-creeper radius multiplier (`2.0`).
pub const POWERED_RADIUS_MULTIPLIER: f32 = 2.0;
/// Vanilla `SwellGoal` keeps swelling while the target is within seven blocks
/// (`distanceToSqr <= 49.0`).
pub const SWELL_DISTANCE_SQR: f64 = 49.0;
/// Vanilla `SwellGoal` only *starts* within three blocks (`distanceToSqr < 9.0`).
pub const SWELL_START_DISTANCE_SQR: f64 = 9.0;
/// Vanilla `Creeper` spawns the explosion at `getY(0.0625)`.
pub const EXPLOSION_HEIGHT_FRACTION: f64 = 0.0625;
/// Vanilla cat and ocelot avoidance distance.
const FELINE_AVOID_DISTANCE: f32 = 6.0;
/// Vanilla cat and ocelot avoidance walk speed.
const FELINE_AVOID_WALK_SPEED: f64 = 1.0;
/// Vanilla cat and ocelot avoidance sprint speed.
const FELINE_AVOID_SPRINT_SPEED: f64 = 1.2;
/// Vanilla `Creeper` melee-approach speed modifier.
const APPROACH_SPEED_MODIFIER: f64 = 1.0;
/// Vanilla `Creeper` stroll speed modifier (`0.8`).
const STROLL_SPEED_MODIFIER: f64 = 0.8;
/// Vanilla `LookAtPlayerGoal` search distance.
const LOOK_AT_PLAYER_DISTANCE: f64 = 8.0;

/// Vanilla `Creeper`.
#[entity_behavior(class = "Creeper")]
pub struct CreeperEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    entity_data: SyncMutex<CreeperEntityData>,
    /// Vanilla `Creeper.swell`, the exploded-in ticks counter.
    swell: AtomicI32,
}

// SAFETY: This key is owned by Steel and uniquely identifies `CreeperEntity`.
unsafe impl DowncastType for CreeperEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/creeper");
}

impl CreeperEntity {
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

        let mut entity_data = CreeperEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, FloatGoal::new(&mob_base));
            goal_selector.add_goal(
                3,
                AvoidEntityGoal::with_selector(
                    FELINE_AVOID_DISTANCE,
                    FELINE_AVOID_WALK_SPEED,
                    FELINE_AVOID_SPRINT_SPEED,
                    |entity, _| {
                        entity.entity_type() == &vanilla_entities::CAT
                            || entity.entity_type() == &vanilla_entities::OCELOT
                    },
                ),
            );
            goal_selector.add_goal(4, MeleeAttackGoal::new(APPROACH_SPEED_MODIFIER, false));
            goal_selector.add_goal(5, WaterAvoidingRandomStrollGoal::new(STROLL_SPEED_MODIFIER));
            goal_selector.add_goal(6, LookAtPlayerGoal::new(LOOK_AT_PLAYER_DISTANCE));
            goal_selector.add_goal(6, RandomLookAroundGoal::new());

            let mut target_selector = mob_base.target_selector().lock();
            target_selector.add_goal(
                1,
                NearestAttackableTargetGoal::new_for_players(true, |_, _| true),
            );
            target_selector.add_goal(2, HurtByTargetGoal::new());
        }

        Self {
            base,
            entity_type,
            living_base,
            mob_base,
            entity_data: SyncMutex::new(entity_data),
            swell: AtomicI32::new(0),
        }
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        sync_dirty_mob_effects(&self.living_base, &self.entity_data);
    }

    /// Vanilla `Creeper.getSwellDir`: `1` while swelling, `-1` while deflating.
    #[must_use]
    pub fn swell_dir(&self) -> i32 {
        *self.entity_data.lock().swell_dir.get()
    }

    /// Sets the swell direction (vanilla `Creeper.setSwellDir`).
    pub fn set_swell_dir(&self, dir: i32) {
        self.entity_data.lock().swell_dir.set(dir);
    }

    /// Vanilla `Creeper.getSwell`: the current fuse progress in ticks.
    #[must_use]
    pub fn swell(&self) -> i32 {
        self.swell.load(Ordering::Relaxed)
    }

    /// Sets the fuse progress, clamped to `[-1, MAX_SWELL]`.
    ///
    /// Vanilla only ever stores `0..=MAX_SWELL`, but `setSwell(-1)` is legal.
    pub fn set_swell(&self, swell: i32) {
        self.swell
            .store(swell.clamp(-1, MAX_SWELL), Ordering::Relaxed);
    }

    /// Vanilla `Creeper.isPowered`.
    #[must_use]
    pub fn is_powered(&self) -> bool {
        *self.entity_data.lock().is_powered.get()
    }

    /// Sets the powered flag (vanilla `Creeper.setPowered`).
    pub fn set_powered(&self, powered: bool) {
        self.entity_data.lock().is_powered.set(powered);
    }

    /// Vanilla `Creeper.isIgnited`: the synced short-fuse flag.
    #[must_use]
    pub fn is_ignited(&self) -> bool {
        *self.entity_data.lock().is_ignited.get()
    }

    /// Sets the synced ignited flag (vanilla `Creeper.setIgnited`).
    pub fn set_ignited(&self, ignited: bool) {
        self.entity_data.lock().is_ignited.set(ignited);
    }

    /// Vanilla `Creeper.getExplosionPower`: `3.0`, doubled while powered.
    #[must_use]
    pub fn explosion_power(&self) -> f32 {
        Self::power_for_powered(self.is_powered())
    }

    /// Pure form of the vanilla explosion radius, usable without an entity.
    #[must_use]
    pub fn power_for_powered(powered: bool) -> f32 {
        if powered {
            EXPLOSION_RADIUS * POWERED_RADIUS_MULTIPLIER
        } else {
            EXPLOSION_RADIUS
        }
    }

    /// Vanilla `Creeper` fuse step: `swell += dir`, never dropping below zero.
    #[must_use]
    pub fn next_swell(current: i32, dir: i32) -> i32 {
        (current + dir).max(0)
    }

    /// Runs the vanilla `tick` fuse logic and detonates on completion.
    fn tick_swell(&self, entity: &SharedEntity) {
        if !LivingEntity::is_alive(self) {
            return;
        }

        // Vanilla `SwellGoal`: the fuse only *starts* within three blocks
        // (`canUse`), but keeps burning out to seven blocks while the target
        // stays visible (`canContinueToUse` + `tick`).
        let already_swelling = self.swell_dir() > 0;
        let dir = match self.target() {
            Some(target)
                if self.distance_to_sqr(target.position()) <= SWELL_DISTANCE_SQR
                    && self.has_line_of_sight_cached(target.as_ref())
                    && (already_swelling
                        || self.distance_to_sqr(target.position()) < SWELL_START_DISTANCE_SQR) =>
            {
                1
            }
            _ => -1,
        };
        self.set_swell_dir(dir);

        let was_idle = self.swell() == 0;
        if dir > 0 && was_idle {
            // Vanilla plays the primer exactly on the idle -> swelling edge.
            self.play_sound(&sound_events::ENTITY_CREEPER_PRIMED, 1.0, 0.5);
        }

        let swell = Self::next_swell(self.swell(), dir);
        self.set_swell(swell);
        if swell >= MAX_SWELL {
            self.explode(entity);
        }
    }

    /// Vanilla `Creeper.explodeCreeper`: a `MOB` explosion at `getY(0.0625)`.
    fn explode(&self, entity: &SharedEntity) {
        let Some(world) = self.level() else {
            return;
        };
        let position = self.position();
        let center = DVec3::new(
            position.x,
            position.y + EXPLOSION_HEIGHT_FRACTION,
            position.z,
        );

        let mut options =
            ExplosionOptions::new(center, self.explosion_power(), ExplosionInteraction::Mob);
        options.source = Some(Arc::<dyn Entity>::clone(entity));
        options.immutable_block_calculator = Some(&DefaultExplosionDamageCalculator);
        world.explode(options);

        self.set_removed(RemovalReason::Killed);
    }
}

impl DefaultEntityPush for CreeperEntity {}

impl Entity for CreeperEntity {
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

impl LivingEntity for CreeperEntity {
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
        Some(&sound_events::ENTITY_CREEPER_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_CREEPER_DEATH)
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
        self.tick_swell(entity);
        if self.is_removed() {
            return None;
        }
        Monster::monster_ai_step(self, entity)
    }
}

impl Mob for CreeperEntity {
    fn mob_base(&self) -> &MobBase {
        &self.mob_base
    }

    fn tick_goal_selectors(&self, entity: &SharedEntity) {
        PathfinderMob::tick_pathfinder_goal_selectors(self, entity);
    }

    fn tick_path_navigation(&self) {
        PathfinderMob::tick_pathfinder_path_navigation(self);
    }

    /// Vanilla `SwellGoal.start`/`tick` stop the navigation so the creeper
    /// stands still while the fuse burns. This hook runs after the goal
    /// selectors and the navigation tick, so it always wins for that tick.
    fn custom_server_ai_step(&self) {
        if self.swell_dir() > 0 {
            self.mob_base().navigation().lock().stop();
        }
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }
}

impl Monster for CreeperEntity {}

impl PathfinderMob for CreeperEntity {}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::sync::Weak;

    use glam::DVec3;
    use simdnbt::borrow::read_compound;
    use simdnbt::owned::NbtCompound;
    use steel_registry::{init_vanilla_registry, vanilla_entities};

    use super::{
        CreeperEntity, EXPLOSION_RADIUS, MAX_SWELL, POWERED_RADIUS_MULTIPLIER, SWELL_DISTANCE_SQR,
        SWELL_START_DISTANCE_SQR,
    };
    use crate::entity::{Entity, LivingEntity};

    #[test]
    fn creeper_uses_registry_attributes_and_round_trips_nbt() {
        init_vanilla_registry();
        let creeper = CreeperEntity::new(&vanilla_entities::CREEPER, 1, DVec3::ZERO, Weak::new());
        assert!((creeper.get_max_health() - 20.0).abs() < 1e-4);
        assert!(!creeper.is_powered());
        assert!(!creeper.is_ignited());
        assert_eq!(creeper.swell(), 0);

        let mut nbt = NbtCompound::new();
        creeper.save_additional(&mut nbt);

        let loaded = CreeperEntity::new(&vanilla_entities::CREEPER, 2, DVec3::ZERO, Weak::new());
        let mut bytes = Vec::new();
        nbt.write(&mut bytes);
        let borrowed = read_compound(&mut Cursor::new(&bytes))
            .unwrap_or_else(|error| panic!("reborrow failed: {error}"));
        loaded.load_additional((&borrowed).into());
        assert!(loaded.get_health() > 0.0);
    }

    #[test]
    fn fuse_accumulates_and_deflates_without_going_negative() {
        // Swelling advances one tick per tick and stops at the last tick before
        // detonation; deflating never drops below zero.
        assert_eq!(CreeperEntity::next_swell(0, 1), 1);
        assert_eq!(CreeperEntity::next_swell(MAX_SWELL - 1, 1), MAX_SWELL);
        assert_eq!(CreeperEntity::next_swell(5, -1), 4);
        assert_eq!(CreeperEntity::next_swell(0, -1), 0);
    }

    #[test]
    fn explosion_power_doubles_when_powered() {
        assert!((CreeperEntity::power_for_powered(false) - EXPLOSION_RADIUS).abs() < f32::EPSILON);
        assert!(
            (CreeperEntity::power_for_powered(true) - EXPLOSION_RADIUS * POWERED_RADIUS_MULTIPLIER)
                .abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn swell_window_matches_vanilla_swell_goal() {
        assert!((SWELL_DISTANCE_SQR - 49.0).abs() < f64::EPSILON, "7 blocks");
        assert!(
            (SWELL_START_DISTANCE_SQR - 9.0).abs() < f64::EPSILON,
            "3 blocks"
        );
        assert_eq!(MAX_SWELL, 30);
        assert_eq!(
            CreeperEntity::next_swell(MAX_SWELL - 1, 1),
            MAX_SWELL,
            "the fuse detonates on the 30th tick"
        );
    }

    #[test]
    fn synced_flags_round_trip_through_the_entity_data_layer() {
        init_vanilla_registry();
        let creeper = CreeperEntity::new(&vanilla_entities::CREEPER, 3, DVec3::ZERO, Weak::new());

        creeper.set_powered(true);
        assert!(creeper.is_powered());
        assert!(
            (creeper.explosion_power() - EXPLOSION_RADIUS * POWERED_RADIUS_MULTIPLIER).abs()
                < f32::EPSILON
        );

        creeper.set_ignited(true);
        assert!(creeper.is_ignited());

        creeper.set_swell_dir(1);
        assert_eq!(creeper.swell_dir(), 1);

        creeper.set_swell(-5);
        assert_eq!(creeper.swell(), -1, "clamped to the vanilla floor");
        creeper.set_swell(MAX_SWELL + 10);
        assert_eq!(creeper.swell(), MAX_SWELL, "clamped to the vanilla cap");
    }
}
