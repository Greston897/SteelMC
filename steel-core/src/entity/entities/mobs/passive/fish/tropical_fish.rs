//! Vanilla `TropicalFishEntity`.
//!
//! `TropicalFish extends AbstractSchoolingFish` like cod and salmon, so the
//! shared `WaterAnimal` + `FishEntity` layers come from [`super`]. On top of
//! them it tracks one packed variant int (`DATA_ID_TYPE_VARIANT`): bit 0..8 hold
//! the [`TropicalFishPattern`] packed id (base size + pattern index), bits 16..24
//! the base colour and bits 24..32 the pattern colour. The same packing is
//! reused for the `Variant` entity NBT key and the `BucketVariantTag` bucket key.
//!
//! Deviations (deliberate, documented in `PARITY-26.3-RU.md`):
//! * vanilla rolls a spawn variant out of a fixed list of 22 "common" packed
//!   variants (`TropicalFish.COMMON_VARIANTS`). That list is not part of the
//!   extracted data, so Steel rolls a uniform pattern and two uniform colours;
//!   the resulting packed value keeps the vanilla layout, only the distribution
//!   differs.
//! * `AbstractSchoolingFish` leader/group bookkeeping is not wired yet (same as
//!   cod and salmon); tropical fish use the default `getMaxGroupSize() == 1`,
//!   so no cod-shaped group refresh is skipped.
//! * Java edition exposes a single hitbox for both pattern bases (0.5 x 0.4 per
//!   the extracted `entities.json` and the wiki hitbox table), so the large
//!   patterns do not grow the box.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::{borrow, owned};
use steel_macros::entity_behavior;
use steel_registry::DyeColor;
use steel_registry::entity_data::EntityPose;
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::entity_variant::TropicalFishPattern;
use steel_registry::items::ItemRef;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::sound_events;
use steel_registry::vanilla_attributes;
use steel_registry::vanilla_entity_data::TropicalFishEntityData;
use steel_registry::vanilla_items;
use steel_utils::locks::SyncMutex;
use steel_utils::nbt::NbtValueInput as _;
use steel_utils::{DowncastType, DowncastTypeKey};

use super::{
    FISH_LIMIT_PER_CHUNK, FISH_MAX_HEALTH, FISH_MOVEMENT_SPEED, FISH_PANIC_SPEED,
    FISH_SWIM_TO_RANDOM_PLACE_INTERVAL, FISH_SWIM_TO_RANDOM_PLACE_SPEED,
    WATER_ANIMAL_AMBIENT_SOUND_INTERVAL, WATER_ANIMAL_EXPERIENCE_MAX, WATER_ANIMAL_EXPERIENCE_MIN,
    check_water_animal_spawn_rules, flop, tick_breathing, travel_in_water,
};
use crate::entity::ai::goal::{PanicGoal, RandomSwimmingGoal};
use crate::entity::ai::path::PathType;
use crate::entity::bucketable::{self, Bucketable};
use crate::entity::damage::DamageSource;
use crate::entity::registry::SpawnRule;
use crate::entity::{
    Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, EntitySpawnReason, EntitySyncedData,
    LivingEntity, LivingEntityBase, Mob, MobBase, PathfinderMob, SharedEntity, SpawnGroupData,
};
use crate::physics::MoveResult;
use crate::world::World;

const TROPICAL_FISH_FLOP_SOUND: SoundEventRef = &sound_events::ENTITY_TROPICAL_FISH_FLOP;

/// Bit shift of the base colour inside the packed vanilla variant.
pub const BASE_COLOR_SHIFT: u32 = 16;
/// Bit shift of the pattern colour inside the packed vanilla variant.
pub const PATTERN_COLOR_SHIFT: u32 = 24;
/// Mask of one packed colour byte.
pub const COLOR_MASK: i32 = 0xFF;
/// Mask of the packed pattern id (`TropicalFishPattern` packed ids stay below it).
pub const PATTERN_MASK: i32 = 0xFFFF;

/// Vanilla `TropicalFish.DEFAULT_VARIANT` base colour (`kob`, white, white).
pub const DEFAULT_BASE_COLOR: DyeColor = DyeColor::White;
/// Vanilla `TropicalFish.DEFAULT_VARIANT` pattern colour.
pub const DEFAULT_PATTERN_COLOR: DyeColor = DyeColor::White;
/// Vanilla `TropicalFish.DEFAULT_VARIANT` pattern.
pub const DEFAULT_PATTERN: TropicalFishPattern = TropicalFishPattern::Kob;

/// Vanilla `TropicalFish.packVariant`: pattern id, base colour, pattern colour.
#[must_use]
pub const fn pack_variant(
    pattern: TropicalFishPattern,
    base_color: DyeColor,
    pattern_color: DyeColor,
) -> i32 {
    pattern.id()
        | (base_color.id() << BASE_COLOR_SHIFT)
        | (pattern_color.id() << PATTERN_COLOR_SHIFT)
}

/// Vanilla `TropicalFish.getPattern(int)`.
#[must_use]
pub const fn pattern_for_variant(packed: i32) -> TropicalFishPattern {
    TropicalFishPattern::by_id(packed & PATTERN_MASK)
}

/// Vanilla `TropicalFish.getBaseColor(int)`.
#[must_use]
pub const fn base_color_for_variant(packed: i32) -> DyeColor {
    DyeColor::by_id((packed >> BASE_COLOR_SHIFT) & COLOR_MASK)
}

/// Vanilla `TropicalFish.getPatternColor(int)`.
#[must_use]
pub const fn pattern_color_for_variant(packed: i32) -> DyeColor {
    DyeColor::by_id((packed >> PATTERN_COLOR_SHIFT) & COLOR_MASK)
}

/// Vanilla `TropicalFishEntity`.
#[entity_behavior(class = "TropicalFish", spawn_rule)]
pub struct TropicalFishEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    entity_data: SyncMutex<TropicalFishEntityData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `TropicalFishEntity`.
unsafe impl DowncastType for TropicalFishEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/tropical_fish");
}

impl TropicalFishEntity {
    /// Vanilla `WaterAnimal.canSpawn`.
    pub(crate) const SPAWN_RULE: SpawnRule = check_water_animal_spawn_rules;

    /// Creates a new tropical fish entity.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Recreates a tropical fish from saved entity state.
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

        let mut entity_data = TropicalFishEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);
        // Vanilla `TropicalFish.defineSynchedData` starts from the default variant.
        entity_data
            .tropical_fish_mut()
            .id_type_variant
            .set(default_variant());

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

    /// Returns the packed vanilla variant int (`DATA_ID_TYPE_VARIANT`).
    #[must_use]
    pub fn packed_variant(&self) -> i32 {
        *self.entity_data.lock().tropical_fish().id_type_variant.get()
    }

    /// Sets the packed vanilla variant int.
    pub fn set_packed_variant(&self, packed: i32) {
        self.entity_data
            .lock()
            .tropical_fish_mut()
            .id_type_variant
            .set(packed);
    }

    /// Vanilla `TropicalFish.getPattern()`.
    #[must_use]
    pub fn pattern(&self) -> TropicalFishPattern {
        pattern_for_variant(self.packed_variant())
    }

    /// Vanilla `TropicalFish.getBaseColor()`.
    #[must_use]
    pub fn base_color(&self) -> DyeColor {
        base_color_for_variant(self.packed_variant())
    }

    /// Vanilla `TropicalFish.getPatternColor()`.
    #[must_use]
    pub fn pattern_color(&self) -> DyeColor {
        pattern_color_for_variant(self.packed_variant())
    }

    /// Vanilla `TropicalFish.setPackedVariant(pattern, base, patternColor)`.
    pub fn set_variant(
        &self,
        pattern: TropicalFishPattern,
        base_color: DyeColor,
        pattern_color: DyeColor,
    ) {
        self.set_packed_variant(pack_variant(pattern, base_color, pattern_color));
    }

    /// Vanilla `TropicalFish.getMaxGroupSize()` inherited from `AbstractSchoolingFish`.
    #[must_use]
    pub const fn max_group_size() -> i32 {
        super::SCHOOLING_GROUP_SIZE_DEFAULT
    }

    /// Vanilla `FishEntity.getLimitPerChunk()` inherited by tropical fish.
    #[must_use]
    pub const fn limit_per_chunk() -> i32 {
        FISH_LIMIT_PER_CHUNK
    }

    /// Rolls a spawn variant.
    ///
    /// Vanilla picks from `TropicalFish.COMMON_VARIANTS`; that list is not part
    /// of the extracted data, so Steel rolls the pattern and both colours
    /// uniformly instead (documented deviation).
    fn roll_spawn_variant(&self) {
        let pattern = TropicalFishPattern::VALUES
            .get(rand::random_range(0..TropicalFishPattern::VALUES.len()))
            .copied()
            .unwrap_or_default();
        let base_color = DyeColor::by_id(rand::random_range(0..16));
        let pattern_color = DyeColor::by_id(rand::random_range(0..16));
        self.set_variant(pattern, base_color, pattern_color);
    }

    fn handle_air_supply(&self, pre_tick_air_supply: i32) {
        tick_breathing(self, pre_tick_air_supply);
    }

    fn tick_flopping(&self) {
        if !self.is_in_water() && self.on_ground() && self.vertical_collision() {
            flop(self, TROPICAL_FISH_FLOP_SOUND);
        }
    }
}

/// The packed vanilla default variant (`kob`, white, white).
#[must_use]
pub const fn default_variant() -> i32 {
    pack_variant(DEFAULT_PATTERN, DEFAULT_BASE_COLOR, DEFAULT_PATTERN_COLOR)
}

impl Entity for TropicalFishEntity {
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
        // Vanilla `TropicalFish.addAdditionalSaveData` stores the packed int.
        nbt.insert("Variant", self.packed_variant());
    }

    fn load_additional(&self, nbt: borrow::NbtCompound<'_, '_>) {
        self.load_mob(nbt);
        if let Some(variant) = nbt.int_value("Variant") {
            self.set_packed_variant(variant);
        }
    }
}

impl LivingEntity for TropicalFishEntity {
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

    fn hurt_sound(&self, _source: &DamageSource) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_TROPICAL_FISH_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_TROPICAL_FISH_DEATH)
    }

    fn base_experience_reward(&self) -> i32 {
        rand::random_range(WATER_ANIMAL_EXPERIENCE_MIN..WATER_ANIMAL_EXPERIENCE_MAX)
    }
}

impl Mob for TropicalFishEntity {
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
        Some(&sound_events::ENTITY_TROPICAL_FISH_AMBIENT)
    }

    fn ambient_sound_interval(&self) -> i32 {
        WATER_ANIMAL_AMBIENT_SOUND_INTERVAL
    }

    fn finalize_spawn(
        &self,
        _world: &Arc<World>,
        _spawn_reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        self.roll_spawn_variant();
        group_data
    }
}

impl PathfinderMob for TropicalFishEntity {}

impl Bucketable for TropicalFishEntity {
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
        &vanilla_items::TROPICAL_FISH_BUCKET
    }

    fn copy_bucket_data_to_nbt(&self, nbt: &mut owned::NbtCompound) {
        bucketable::write_base_bucket_data(self, nbt);
        // Vanilla `TropicalFish.saveToBucketTag`.
        nbt.insert("BucketVariantTag", self.packed_variant());
    }

    fn read_bucket_data_from_nbt(&self, nbt: &owned::NbtCompound) {
        bucketable::read_base_bucket_data(self, nbt);
        if let Some(variant) = nbt.int_value("BucketVariantTag") {
            self.set_packed_variant(variant);
        }
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::entity_variant::TropicalFishBase;

    use super::*;

    #[test]
    fn default_variant_is_kob_white_white() {
        assert_eq!(default_variant(), 0);
        assert_eq!(pattern_for_variant(0), TropicalFishPattern::Kob);
        assert_eq!(base_color_for_variant(0), DyeColor::White);
        assert_eq!(pattern_color_for_variant(0), DyeColor::White);
    }

    #[test]
    fn packing_round_trips_every_pattern_and_colour() {
        for pattern in TropicalFishPattern::VALUES {
            for base in DyeColor::VALUES {
                for pattern_color in [DyeColor::White, DyeColor::Black, DyeColor::Red] {
                    let packed = pack_variant(pattern, base, pattern_color);
                    assert_eq!(pattern_for_variant(packed), pattern);
                    assert_eq!(base_color_for_variant(packed), base);
                    assert_eq!(pattern_color_for_variant(packed), pattern_color);
                }
            }
        }
    }

    #[test]
    fn large_patterns_keep_their_base_size_bit() {
        let packed = pack_variant(
            TropicalFishPattern::Clayfish,
            DyeColor::Blue,
            DyeColor::Yellow,
        );
        assert_eq!(pattern_for_variant(packed), TropicalFishPattern::Clayfish);
        assert_eq!(
            pattern_for_variant(packed).base(),
            TropicalFishBase::Large
        );
        assert_eq!(base_color_for_variant(packed), DyeColor::Blue);
        assert_eq!(pattern_color_for_variant(packed), DyeColor::Yellow);
    }
}
