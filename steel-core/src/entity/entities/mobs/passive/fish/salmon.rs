//! Vanilla `SalmonEntity`.
//!
//! `Salmon extends AbstractSchoolingFish` with `getMaxGroupSize() == 5` and a
//! tracked `VARIANT` (`SalmonVariant`) that scales the hitbox and the render.
//! The variant is rolled from the vanilla weighted pool (`30 / 50 / 15`) in
//! `initialize`.
//!
//! Deviations (deliberate, documented):
//! * vanilla recalculates the dimensions when the tracked variant changes
//!   (`onSyncedDataUpdated`); Steel derives them on demand in
//!   `dimensions_for_pose`, which is observationally equivalent because the
//!   variant is the only input.
//! * the `SchoolingFish` leader/group bookkeeping is not wired yet: it needs an
//!   entity query (`world.get_entities_in_aabb_matching`) plus a
//!   `FollowFlockLeaderGoal`; `has_self_control` therefore stays `true`.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::{borrow, owned};
use steel_macros::entity_behavior;
use steel_registry::entity_data::EntityPose;
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::entity_variant::SalmonVariant;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::sound_events;
use steel_registry::items::ItemRef;
use steel_registry::vanilla_attributes;
use steel_registry::vanilla_items;
use steel_registry::vanilla_entity_data::SalmonEntityData;
use steel_utils::locks::SyncMutex;
use steel_utils::{DowncastType, DowncastTypeKey};

use super::{
    check_water_animal_spawn_rules, flop, tick_breathing, travel_in_water, FISH_LIMIT_PER_CHUNK,
    FISH_MAX_HEALTH, FISH_MOVEMENT_SPEED, FISH_PANIC_SPEED, FISH_SWIM_TO_RANDOM_PLACE_INTERVAL,
    FISH_SWIM_TO_RANDOM_PLACE_SPEED, WATER_ANIMAL_AMBIENT_SOUND_INTERVAL,
    WATER_ANIMAL_EXPERIENCE_MAX, WATER_ANIMAL_EXPERIENCE_MIN,
};
use crate::entity::ai::goal::{PanicGoal, RandomSwimmingGoal};
use crate::entity::ai::path::PathType;
use crate::entity::bucketable::Bucketable;
use crate::entity::registry::SpawnRule;
use crate::entity::{
    Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, EntitySyncedData, LivingEntity,
    LivingEntityBase, Mob, MobBase, PathfinderMob, SharedEntity,
};
use crate::physics::MoveResult;
use crate::world::World;

const SALMON_FLOP_SOUND: SoundEventRef = &sound_events::ENTITY_SALMON_FLOP;

/// Vanilla `SalmonEntity.getMaxGroupSize()`.
pub const SALMON_MAX_GROUP_SIZE: i32 = 5;

/// Vanilla `SalmonEntity` variant pool weights: `SMALL 30 / MEDIUM 50 / LARGE 15`.
pub const SALMON_VARIANT_WEIGHT_SMALL: i32 = 30;
/// See [`SALMON_VARIANT_WEIGHT_SMALL`].
pub const SALMON_VARIANT_WEIGHT_MEDIUM: i32 = 50;
/// See [`SALMON_VARIANT_WEIGHT_SMALL`].
pub const SALMON_VARIANT_WEIGHT_LARGE: i32 = 15;

/// Vanilla `SalmonEntity.DEFAULT_VARIANT`.
pub const SALMON_DEFAULT_VARIANT: SalmonVariant = SalmonVariant::Medium;

/// Vanilla weighted variant roll (`WeightedRandom` over `[SMALL, MEDIUM, LARGE]`).
///
/// `roll` is expected in `0..95`.
#[must_use]
pub const fn salmon_variant_for_roll(roll: i32) -> SalmonVariant {
    if roll < SALMON_VARIANT_WEIGHT_SMALL {
        SalmonVariant::Small
    } else if roll < SALMON_VARIANT_WEIGHT_SMALL + SALMON_VARIANT_WEIGHT_MEDIUM {
        SalmonVariant::Medium
    } else {
        SalmonVariant::Large
    }
}

#[entity_behavior(class = "Salmon", spawn_rule)]
/// Vanilla Salmon entity
pub struct SalmonEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    entity_data: SyncMutex<SalmonEntityData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `SalmonEntity`.
unsafe impl DowncastType for SalmonEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/salmon");
}

impl SalmonEntity {
    /// Vanilla `WaterAnimal.canSpawn`.
    pub(crate) const SPAWN_RULE: SpawnRule = check_water_animal_spawn_rules;

    /// Creates a new Salmon entity
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Recreates a salmon from saved entity state
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self::new_with_base(
            EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
        )
    }

    /// Vanilla `SalmonEntity.getVariant()`.
    #[must_use]
    pub fn variant(&self) -> SalmonVariant {
        SalmonVariant::by_id(*self.entity_data.lock().variant_type.get())
    }

    /// Vanilla `SalmonEntity.setVariant(SalmonVariant)`.
    pub fn set_variant(&self, variant: SalmonVariant) {
        self.entity_data.lock().variant_type.set(variant.id());
    }

    /// Vanilla `SalmonEntity.getVariantScale()`.
    #[must_use]
    pub fn variant_scale(&self) -> f32 {
        self.variant().bounding_box_scale()
    }

    /// Vanilla `SchoolingFish.getMaxGroupSize()`.
    #[must_use]
    pub const fn max_group_size() -> i32 {
        SALMON_MAX_GROUP_SIZE
    }

    fn new_with_base(base: EntityBase, entity_type: EntityTypeRef) -> Self {
        let living_base = LivingEntityBase::new(entity_type);
        living_base
            .attributes()
            .lock()
            .set_base_value(vanilla_attributes::MAX_HEALTH, FISH_MAX_HEALTH);
        living_base
            .attributes()
            .lock()
            .set_base_value(vanilla_attributes::MOVEMENT_SPEED, FISH_MOVEMENT_SPEED);

        let mob_base = MobBase::new();
        mob_base
            .pathfinding_malus()
            .lock()
            .set(PathType::Water, 0.0);

        let mut entity_data = SalmonEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);
        // Vanilla `SalmonEntity.defineSynchedData`: default is `MEDIUM.index`.
        entity_data.variant_type.set(SALMON_DEFAULT_VARIANT.id());

        {
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, PanicGoal::new(FISH_PANIC_SPEED));
            goal_selector.add_goal(
                1,
                RandomSwimmingGoal::new(
                    FISH_SWIM_TO_RANDOM_PLACE_SPEED,
                    FISH_SWIM_TO_RANDOM_PLACE_INTERVAL,
                ),
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

    fn handle_air_supply(&self, pre_tick_air_supply: i32) {
        tick_breathing(self, pre_tick_air_supply);
    }

    fn tick_flopping(&self) {
        if !self.is_in_water() && self.on_ground() && self.vertical_collision() {
            flop(self, SALMON_FLOP_SOUND);
        }
    }

    /// Vanilla `SalmonEntity` spawn variant roll.
    fn roll_spawn_variant(&self) {
        let total = SALMON_VARIANT_WEIGHT_SMALL + SALMON_VARIANT_WEIGHT_MEDIUM + SALMON_VARIANT_WEIGHT_LARGE;
        self.set_variant(salmon_variant_for_roll(rand::random_range(0..total)));
    }
}

impl Entity for SalmonEntity {
    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn movement_emission(&self) -> EntityMovementEmission {
        EntityMovementEmission::Events
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn base_tick(&self) {
        let air_supply = self.air_supply();
        Mob::base_tick_mob(self);
        self.handle_air_supply(air_supply);
    }

    fn dimensions_for_pose(&self, _pose: EntityPose) -> EntityDimensions {
        let scale = LivingEntity::get_scale(self) * self.variant_scale();
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
        // Vanilla `SalmonEntity.addAdditionalSaveData` stores the variant by name.
        nbt.insert("type", self.variant().serialized_name().to_string());
    }

    fn load_additional(&self, nbt: borrow::NbtCompound<'_, '_>) {
        self.load_mob(nbt);
        if let Some(name) = nbt.string("type") {
            let name = name.to_str();
            self.set_variant(
                SalmonVariant::from_serialized_name(name.as_ref()).unwrap_or(SALMON_DEFAULT_VARIANT),
            );
        }
    }
}

impl LivingEntity for SalmonEntity {
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
        self.tick_flopping();
        result
    }

    fn hurt_sound(&self, _source: &crate::entity::damage::DamageSource) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_SALMON_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_SALMON_DEATH)
    }

    fn base_experience_reward(&self) -> i32 {
        rand::random_range(WATER_ANIMAL_EXPERIENCE_MIN..WATER_ANIMAL_EXPERIENCE_MAX)
    }
}

impl Mob for SalmonEntity {
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
        Some(&sound_events::ENTITY_SALMON_AMBIENT)
    }

    fn ambient_sound_interval(&self) -> i32 {
        WATER_ANIMAL_AMBIENT_SOUND_INTERVAL
    }

    fn finalize_spawn(
        &self,
        _world: &Arc<World>,
        _spawn_reason: crate::entity::EntitySpawnReason,
        group_data: Option<crate::entity::SpawnGroupData>,
    ) -> Option<crate::entity::SpawnGroupData> {
        self.roll_spawn_variant();
        group_data
    }
}

impl PathfinderMob for SalmonEntity {}

/// Vanilla `FishEntity.getLimitPerChunk()` inherited by salmon.
#[must_use]
pub const fn salmon_limit_per_chunk() -> i32 {
    FISH_LIMIT_PER_CHUNK
}

impl Bucketable for SalmonEntity {
    fn is_from_bucket(&self) -> bool {
        *self.entity_data.lock().from_bucket.get()
    }

    fn set_from_bucket(&self, from_bucket: bool) {
        self.entity_data.lock().from_bucket.set(from_bucket);
    }

    fn bucket_item(&self) -> ItemRef {
        &vanilla_items::SALMON_BUCKET
    }

    fn copy_bucket_data_to_nbt(&self, nbt: &mut owned::NbtCompound) {
        crate::entity::bucketable::write_base_bucket_data(self, nbt);
        nbt.insert("type", self.variant().serialized_name().to_string());
    }
}
