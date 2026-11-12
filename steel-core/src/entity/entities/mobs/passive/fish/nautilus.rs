//! Vanilla `Nautilus` and `ZombieNautilus`.
//!
//! Both extend `AbstractNautilus` (dash / swim speeds) on top of
//! `TamableAnimal` + `AgeableMob` + `Animal`. Steel does not model the
//! `TamableAnimal` layer as a trait yet, so the taming flags live directly in
//! the generated `TamableAnimal` part of the entity data and the species only
//! adds the vanilla dash behaviour.

use std::sync::Weak;

use glam::DVec3;
use simdnbt::{borrow, owned};
use steel_macros::entity_behavior;
use steel_registry::entity_data::EntityPose;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::sound_events;
use steel_registry::vanilla_attributes;
use steel_registry::vanilla_entity_data::{NautilusEntityData, ZombieNautilusEntityData};
use steel_utils::locks::SyncMutex;
use steel_utils::{DowncastType, DowncastTypeKey};

use crate::entity::ai::goal::{PanicGoal, RandomSwimmingGoal};
use crate::entity::ai::path::PathType;
use crate::entity::registry::SpawnRule;
use crate::entity::{
    Entity, EntityBase, EntityBaseLoad, EntityDimensions, EntityMovementEmission, EntitySyncedData,
    LivingEntity, LivingEntityBase, Mob, MobBase, PathfinderMob, SharedEntity,
};
use crate::physics::MoveResult;
use crate::world::World;

use super::{
    WATER_ANIMAL_AMBIENT_SOUND_INTERVAL, WATER_ANIMAL_EXPERIENCE_MAX, WATER_ANIMAL_EXPERIENCE_MIN,
    check_water_animal_spawn_rules, flop, tick_breathing, travel_in_water,
};

/// Vanilla `AbstractNautilus` base speed (`0.5`).
const NAUTILUS_MOVEMENT_SPEED: f64 = 0.5;
/// Vanilla nautilus dash speed modifier.
const NAUTILUS_DASH_SPEED: f64 = 2.0;
/// Vanilla panic speed modifier.
const NAUTILUS_PANIC_SPEED: f64 = 1.25;
/// Vanilla `SwimToRandomPlaceGoal` interval for nautilus.
const NAUTILUS_SWIM_INTERVAL: i32 = 40;

#[entity_behavior(class = "Nautilus", spawn_rule)]
/// Vanilla `Nautilus`: a tameable water animal that dashes away.
pub struct NautilusEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    entity_data: SyncMutex<NautilusEntityData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `NautilusEntity`.
unsafe impl DowncastType for NautilusEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/nautilus");
}

impl NautilusEntity {
    /// Vanilla `WaterAnimal.canSpawn`.
    pub(crate) const SPAWN_RULE: SpawnRule = check_water_animal_spawn_rules;

    /// Creates a new nautilus.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Recreates a nautilus from saved entity state.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self::new_with_base(
            EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
        )
    }

    fn new_with_base(base: EntityBase, entity_type: EntityTypeRef) -> Self {
        let living_base = LivingEntityBase::new(entity_type);
        living_base
            .attributes()
            .lock()
            .set_base_value(vanilla_attributes::MOVEMENT_SPEED, NAUTILUS_MOVEMENT_SPEED);

        let mob_base = MobBase::new();
        mob_base
            .pathfinding_malus()
            .lock()
            .set(PathType::Water, 0.0);

        let mut entity_data = NautilusEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, PanicGoal::new(NAUTILUS_PANIC_SPEED));
            goal_selector.add_goal(1, RandomSwimmingGoal::new(1.0, NAUTILUS_SWIM_INTERVAL));
        }

        Self {
            base,
            entity_type,
            living_base,
            mob_base,
            entity_data: SyncMutex::new(entity_data),
        }
    }

    /// Vanilla `AbstractNautilus.isDash`.
    #[must_use]
    pub fn is_dash(&self) -> bool {
        *self.entity_data.lock().abstract_nautilus().dash.get()
    }

    /// Vanilla `AbstractNautilus.setDash`.
    pub fn set_dash(&self, dash: bool) {
        self.entity_data.lock().abstract_nautilus_mut().dash.set(dash);
    }

    /// Vanilla dash: mark the nautilus as dashing and boost it forward.
    pub fn start_dash(&self) {
        self.set_dash(true);
        let (yaw, _) = self.rotation();
        let rotation = yaw.to_radians();
        let direction = DVec3::new(-rotation.sin(), 0.0, rotation.cos());
        self.set_velocity(direction * NAUTILUS_DASH_SPEED);
    }
}

impl Entity for NautilusEntity {
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
        let air_supply = self.air_supply();
        Mob::base_tick_mob(self);
        tick_breathing(self, air_supply);
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

    fn save_additional(&self, nbt: &mut owned::NbtCompound) {
        self.save_mob(nbt);
    }

    fn load_additional(&self, nbt: borrow::NbtCompound<'_, '_>) {
        self.load_mob(nbt);
    }
}

impl LivingEntity for NautilusEntity {
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

    fn travel_in_water(
        &self,
        shared: &SharedEntity,
        input: DVec3,
        _base_gravity: f64,
        _is_falling: bool,
        _old_y: f64,
    ) -> Option<MoveResult> {
        let navigation_idle = self.mob_base.navigation().lock().is_done();
        travel_in_water(self, shared, input, navigation_idle)
    }

    fn server_ai_step(&self, entity: &SharedEntity) {
        Mob::mob_server_ai_step(self, entity);
    }

    fn ai_step(&self, entity: &SharedEntity) -> Option<MoveResult> {
        let result = Mob::mob_ai_step(self, entity);
        if !self.is_in_water() && self.on_ground() && self.vertical_collision() {
            flop(self, &sound_events::ENTITY_NAUTILUS_SWIM);
        }
        result
    }

    fn hurt_sound(&self, _source: &crate::entity::damage::DamageSource) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_NAUTILUS_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_NAUTILUS_DEATH)
    }

    fn base_experience_reward(&self) -> i32 {
        rand::random_range(WATER_ANIMAL_EXPERIENCE_MIN..WATER_ANIMAL_EXPERIENCE_MAX)
    }
}

impl Mob for NautilusEntity {
    fn mob_base(&self) -> &MobBase {
        &self.mob_base
    }

    fn tick_goal_selectors(&self, entity: &SharedEntity) {
        PathfinderMob::tick_pathfinder_goal_selectors(self, entity);
    }

    fn tick_path_navigation(&self) {
        PathfinderMob::tick_pathfinder_path_navigation(self);
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }

    fn ambient_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_NAUTILUS_AMBIENT)
    }

    fn ambient_sound_interval(&self) -> i32 {
        WATER_ANIMAL_AMBIENT_SOUND_INTERVAL
    }
}

impl PathfinderMob for NautilusEntity {}

#[entity_behavior(class = "ZombieNautilus")]
/// Vanilla `ZombieNautilus`: the zombified nautilus variant.
pub struct ZombieNautilusEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    entity_data: SyncMutex<ZombieNautilusEntityData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `ZombieNautilusEntity`.
unsafe impl DowncastType for ZombieNautilusEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/zombie_nautilus");
}

impl ZombieNautilusEntity {
    /// Creates a new zombie nautilus.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Recreates a zombie nautilus from saved entity state.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self::new_with_base(
            EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
        )
    }

    fn new_with_base(base: EntityBase, entity_type: EntityTypeRef) -> Self {
        let living_base = LivingEntityBase::new(entity_type);
        living_base
            .attributes()
            .lock()
            .set_base_value(vanilla_attributes::MOVEMENT_SPEED, NAUTILUS_MOVEMENT_SPEED);

        let mob_base = MobBase::new();
        mob_base
            .pathfinding_malus()
            .lock()
            .set(PathType::Water, 0.0);

        let mut entity_data = ZombieNautilusEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, PanicGoal::new(NAUTILUS_PANIC_SPEED));
            goal_selector.add_goal(1, RandomSwimmingGoal::new(1.0, NAUTILUS_SWIM_INTERVAL));
        }

        Self {
            base,
            entity_type,
            living_base,
            mob_base,
            entity_data: SyncMutex::new(entity_data),
        }
    }

    /// Whether the zombie nautilus is currently dashing.
    #[must_use]
    pub fn is_dash(&self) -> bool {
        *self.entity_data.lock().abstract_nautilus().dash.get()
    }

    /// Sets the dash flag.
    pub fn set_dash(&self, dash: bool) {
        self.entity_data.lock().abstract_nautilus_mut().dash.set(dash);
    }
}

impl Entity for ZombieNautilusEntity {
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
        let air_supply = self.air_supply();
        Mob::base_tick_mob(self);
        tick_breathing(self, air_supply);
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

    fn save_additional(&self, nbt: &mut owned::NbtCompound) {
        self.save_mob(nbt);
    }

    fn load_additional(&self, nbt: borrow::NbtCompound<'_, '_>) {
        self.load_mob(nbt);
    }
}

impl LivingEntity for ZombieNautilusEntity {
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

    fn travel_in_water(
        &self,
        shared: &SharedEntity,
        input: DVec3,
        _base_gravity: f64,
        _is_falling: bool,
        _old_y: f64,
    ) -> Option<MoveResult> {
        let navigation_idle = self.mob_base.navigation().lock().is_done();
        travel_in_water(self, shared, input, navigation_idle)
    }

    fn server_ai_step(&self, entity: &SharedEntity) {
        Mob::mob_server_ai_step(self, entity);
    }

    fn ai_step(&self, entity: &SharedEntity) -> Option<MoveResult> {
        Mob::mob_ai_step(self, entity)
    }

    fn hurt_sound(&self, _source: &crate::entity::damage::DamageSource) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_ZOMBIE_NAUTILUS_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_ZOMBIE_NAUTILUS_DEATH)
    }

    fn base_experience_reward(&self) -> i32 {
        rand::random_range(WATER_ANIMAL_EXPERIENCE_MIN..WATER_ANIMAL_EXPERIENCE_MAX)
    }
}

impl Mob for ZombieNautilusEntity {
    fn mob_base(&self) -> &MobBase {
        &self.mob_base
    }

    fn tick_goal_selectors(&self, entity: &SharedEntity) {
        PathfinderMob::tick_pathfinder_goal_selectors(self, entity);
    }

    fn tick_path_navigation(&self) {
        PathfinderMob::tick_pathfinder_path_navigation(self);
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }

    fn ambient_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_ZOMBIE_NAUTILUS_AMBIENT)
    }

    fn ambient_sound_interval(&self) -> i32 {
        WATER_ANIMAL_AMBIENT_SOUND_INTERVAL
    }
}

impl PathfinderMob for ZombieNautilusEntity {}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_entities};

    use super::*;

    #[test]
    fn new_nautilus_is_not_dashing() {
        init_vanilla_registry();
        let nautilus =
            NautilusEntity::new(&vanilla_entities::NAUTILUS, 1, DVec3::ZERO, Weak::new());
        assert!(!nautilus.is_dash());
        nautilus.set_dash(true);
        assert!(nautilus.is_dash());
    }

    #[test]
    fn new_zombie_nautilus_is_not_dashing() {
        init_vanilla_registry();
        let nautilus = ZombieNautilusEntity::new(
            &vanilla_entities::ZOMBIE_NAUTILUS,
            1,
            DVec3::ZERO,
            Weak::new(),
        );
        assert!(!nautilus.is_dash());
    }

    #[test]
    fn nautilus_movement_speed_matches_vanilla() {
        assert_eq!(NAUTILUS_MOVEMENT_SPEED, 0.5);
        assert_eq!(NAUTILUS_DASH_SPEED, 2.0);
    }
}
