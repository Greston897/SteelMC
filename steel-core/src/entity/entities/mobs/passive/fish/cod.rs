//! Vanilla `CodEntity`.
//!
//! `Cod extends AbstractSchoolingFish` in vanilla, but with the default
//! `getMaxGroupSize() == 1` — a cod never recruits followers, so the schooling
//! bookkeeping is only the "I can follow a leader" half of the algorithm and the
//! group refresh is skipped entirely. That is why this entity only carries the
//! `WaterAnimal` + `FishEntity` layers.

use std::sync::Weak;

use glam::DVec3;
use simdnbt::{borrow, owned};
use steel_macros::entity_behavior;
use steel_registry::entity_data::EntityPose;
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::items::ItemRef;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::sound_events;
use steel_registry::vanilla_attributes;
use steel_registry::vanilla_entity_data::CodEntityData;
use steel_registry::vanilla_items;
use steel_utils::locks::SyncMutex;
use steel_utils::{DowncastType, DowncastTypeKey};

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

use super::{
    FISH_LIMIT_PER_CHUNK, FISH_MAX_HEALTH, FISH_MOVEMENT_SPEED, FISH_PANIC_SPEED,
    FISH_SWIM_TO_RANDOM_PLACE_INTERVAL, FISH_SWIM_TO_RANDOM_PLACE_SPEED,
    WATER_ANIMAL_AMBIENT_SOUND_INTERVAL, WATER_ANIMAL_EXPERIENCE_MAX, WATER_ANIMAL_EXPERIENCE_MIN,
    check_water_animal_spawn_rules, flop, tick_breathing, travel_in_water,
};

const COD_FLOP_SOUND: SoundEventRef = &sound_events::ENTITY_COD_FLOP;

#[entity_behavior(class = "Cod", spawn_rule)]
/// Vanilla Cod entity
pub struct CodEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    entity_data: SyncMutex<CodEntityData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `CodEntity`.
unsafe impl DowncastType for CodEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/cod");
}

impl CodEntity {
    /// Vanilla `WaterAnimal.canSpawn`.
    pub(crate) const SPAWN_RULE: SpawnRule = check_water_animal_spawn_rules;

    /// Creates a new Cod entity
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Recreates a cod from saved entity state
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
            .set_base_value(vanilla_attributes::MAX_HEALTH, FISH_MAX_HEALTH);
        living_base
            .attributes()
            .lock()
            .set_base_value(vanilla_attributes::MOVEMENT_SPEED, FISH_MOVEMENT_SPEED);

        let mob_base = MobBase::new();
        // Vanilla `WaterAnimal` gives water path nodes a malus of `0.0`.
        mob_base
            .pathfinding_malus()
            .lock()
            .set(PathType::Water, 0.0);

        let mut entity_data = CodEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            // Vanilla goal priorities of `FishEntity`.
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

    /// Vanilla `FishEntity` spawn cap per chunk.
    #[must_use]
    pub const fn limit_per_chunk() -> i32 {
        FISH_LIMIT_PER_CHUNK
    }

    fn handle_air_supply(&self, pre_tick_air_supply: i32) {
        tick_breathing(self, pre_tick_air_supply);
    }

    fn tick_flopping(&self) {
        if !self.is_in_water() && self.on_ground() && self.vertical_collision() {
            flop(self, COD_FLOP_SOUND);
        }
    }
}

impl Entity for CodEntity {
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

impl LivingEntity for CodEntity {
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
        Some(&sound_events::ENTITY_COD_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_COD_DEATH)
    }

    fn base_experience_reward(&self) -> i32 {
        rand::random_range(WATER_ANIMAL_EXPERIENCE_MIN..WATER_ANIMAL_EXPERIENCE_MAX)
    }
}

impl Mob for CodEntity {
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
        Some(&sound_events::ENTITY_COD_AMBIENT)
    }

    fn ambient_sound_interval(&self) -> i32 {
        WATER_ANIMAL_AMBIENT_SOUND_INTERVAL
    }
}

impl PathfinderMob for CodEntity {}

impl Bucketable for CodEntity {
    fn is_from_bucket(&self) -> bool {
        *self.entity_data.lock().abstract_fish().from_bucket.get()
    }

    fn set_from_bucket(&self, from_bucket: bool) {
        self.entity_data
            .lock()
            .abstract_fish_mut()
            .from_bucket
            .set(from_bucket);
    }

    fn bucket_item(&self) -> ItemRef {
        &vanilla_items::COD_BUCKET
    }
}
