//! Vanilla `PufferfishEntity`.
//!
//! Mechanics ported: the three-state puff machine (`NOT_PUFFED`, `SEMI_PUFFED`,
//! `FULLY_PUFFED`) with vanilla tick thresholds (`>0 → 1`, `>40 → 2` when
//! inflating; `>60` and `>100` when deflating), the hitbox scale
//! (`0.5 / 0.7 / 1.0`), the poison sting on every mob inside
//! `boundingBox.expand(0.3)` and the puff sounds.
//!
//! Deviations (documented):
//! * vanilla `InflateGoal` uses a `NOT_SCARY_FOR_PUFFERFISH` entity-type tag and
//!   `createNonAttackable().ignoreDistanceScalingFactor().ignoreVisibility()`;
//!   Steel approximates it with "a non-spectator, non-creative player within
//!   8 blocks" because the tag pipeline is not wired for this predicate yet.
//! * `playerTouch` (`onPlayerCollision`) is not wired: Steel has no
//!   `Entity.onPlayerCollision` hook for this entity class yet.

use std::sync::Weak;

use glam::DVec3;
use simdnbt::{borrow, owned};
use steel_macros::entity_behavior;
use steel_registry::entity_data::EntityPose;
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::item_stack::ItemStack;
use steel_registry::items::ItemRef;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::sound_events;
use steel_registry::vanilla_attributes;
use steel_registry::vanilla_entity_data::PufferfishEntityData;
use steel_registry::vanilla_items;
use steel_registry::vanilla_mob_effects::POISON;
use steel_utils::locks::SyncMutex;
use steel_utils::{DowncastType, DowncastTypeKey};

use super::{
    FISH_LIMIT_PER_CHUNK, FISH_MAX_HEALTH, FISH_MOVEMENT_SPEED, FISH_PANIC_SPEED,
    FISH_SWIM_TO_RANDOM_PLACE_INTERVAL, FISH_SWIM_TO_RANDOM_PLACE_SPEED,
    WATER_ANIMAL_AMBIENT_SOUND_INTERVAL, WATER_ANIMAL_EXPERIENCE_MAX, WATER_ANIMAL_EXPERIENCE_MIN,
    check_water_animal_spawn_rules, flop, tick_breathing, travel_in_water,
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
use crate::entity::damage::DamageSource;
use crate::entity::living_base::MobEffectInstance;

/// Vanilla `PufferfishEntity.PUFF_STATE_NOT_PUFFED`.
pub const NOT_PUFFED: i32 = 0;
/// Vanilla `PufferfishEntity.PUFF_STATE_SEMI_PUFFED`.
pub const SEMI_PUFFED: i32 = 1;
/// Vanilla `PufferfishEntity.PUFF_STATE_FULLY_PUFFED`.
pub const FULLY_PUFFED: i32 = 2;
/// Ticks before the semi-puffed state becomes fully puffed.
pub const PUFFER_INFLATE_FULL_TICKS: i32 = 40;
/// Deflate ticks before fully puffed drops to semi-puffed.
pub const PUFFER_DEFLATE_SEMI_TICKS: i32 = 60;
/// Deflate ticks before semi-puffed drops to not puffed.
pub const PUFFER_DEFLATE_NONE_TICKS: i32 = 100;
/// Vanilla inflate trigger radius (player search).
pub const PUFFER_INFLATE_RADIUS: f64 = 8.0;
/// Vanilla sting area expansion.
pub const PUFFER_STING_EXPANSION: f64 = 0.3;
/// Vanilla poison duration factor: `60 * puffState` ticks.
pub const PUFFER_POISON_TICKS_PER_STATE: i32 = 60;

const PUFFER_STING_SOUND: SoundEventRef = &sound_events::ENTITY_PUFFER_FISH_STING;
const PUFFER_BLOW_UP_SOUND: SoundEventRef = &sound_events::ENTITY_PUFFER_FISH_BLOW_UP;
const PUFFER_BLOW_OUT_SOUND: SoundEventRef = &sound_events::ENTITY_PUFFER_FISH_BLOW_OUT;
const PUFFER_FLOP_SOUND: SoundEventRef = &sound_events::ENTITY_PUFFER_FISH_FLOP;

/// Vanilla `PufferfishEntity.getPuffState()`.
#[must_use]
pub const fn is_puffed(state: i32) -> bool {
    state > NOT_PUFFED
}

/// Vanilla `PufferfishEntity` hitbox scale per puff state.
#[must_use]
pub const fn puff_scale(state: i32) -> f32 {
    match state {
        FULLY_PUFFED => 1.0,
        SEMI_PUFFED => 0.7,
        _ => 0.5,
    }
}

/// Vanilla sting damage: `1 + puffState`.
#[must_use]
pub const fn sting_damage(state: i32) -> f32 {
    (1 + state) as f32
}

#[entity_behavior(class = "Pufferfish", spawn_rule)]
/// Vanilla Pufferfish entity
pub struct PufferfishEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    entity_data: SyncMutex<PufferfishEntityData>,
    state: SyncMutex<PufferTicks>,
}

/// Vanilla `PufferfishEntity` non-synced inflate/deflate counters.
pub struct PufferTicks {
    inflate_ticks: i32,
    deflate_ticks: i32,
}

// SAFETY: This key is owned by Steel and uniquely identifies `PufferfishEntity`.
unsafe impl DowncastType for PufferfishEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/pufferfish");
}

impl PufferfishEntity {
    /// Vanilla `WaterAnimal.canSpawn`.
    pub(crate) const SPAWN_RULE: SpawnRule = check_water_animal_spawn_rules;

    /// Creates a new Pufferfish entity
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Recreates a pufferfish from saved entity state
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self::new_with_base(
            EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
        )
    }

    /// Vanilla `getPuffState()`.
    #[must_use]
    pub fn puff_state(&self) -> i32 {
        *self.entity_data.lock().pufferfish().puff_state.get()
    }

    /// Vanilla `setPuffState(int)`.
    pub fn set_puff_state(&self, state: i32) {
        self.entity_data
            .lock()
            .pufferfish_mut()
            .puff_state
            .set(state);
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

        let mut entity_data = PufferfishEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);
        entity_data.pufferfish_mut().puff_state.set(NOT_PUFFED);

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
            state: SyncMutex::new(PufferTicks {
                inflate_ticks: 0,
                deflate_ticks: 0,
            }),
        }
    }

    fn handle_air_supply(&self, pre_tick_air_supply: i32) {
        tick_breathing(self, pre_tick_air_supply);
    }

    fn tick_flopping(&self) {
        if !self.is_in_water() && self.on_ground() && self.vertical_collision() {
            flop(self, PUFFER_FLOP_SOUND);
        }
    }

    /// Vanilla `InflateGoal.canUse` approximation: a threatening player nearby.
    fn should_inflate(&self) -> bool {
        let Some(world) = self.level() else {
            return false;
        };
        let search = self.base.bounding_box().inflate(PUFFER_INFLATE_RADIUS);
        world.has_entity_in_aabb_matching(&search, |entity| {
            entity
                .as_player()
                .is_some_and(|player| !player.is_spectator() && !player.has_infinite_materials())
        })
    }

    /// Vanilla `PufferfishEntity.tick`.
    fn tick_puff_state(&self) {
        let inflating = self.should_inflate();
        let mut state = self.state.lock();
        let mut puff = self.puff_state();

        if inflating {
            state.inflate_ticks += 1;
            state.deflate_ticks = 0;
        } else {
            state.deflate_ticks += 1;
        }

        if state.inflate_ticks > 0 {
            if puff == NOT_PUFFED {
                puff = SEMI_PUFFED;
                self.set_puff_state(puff);
                self.make_sound(Some(PUFFER_BLOW_UP_SOUND));
            } else if puff == SEMI_PUFFED && state.inflate_ticks > PUFFER_INFLATE_FULL_TICKS {
                puff = FULLY_PUFFED;
                self.set_puff_state(puff);
                self.make_sound(Some(PUFFER_BLOW_UP_SOUND));
            }
            state.deflate_ticks = 0;
        } else if puff != NOT_PUFFED
            && (state.deflate_ticks > PUFFER_DEFLATE_SEMI_TICKS
                || state.deflate_ticks > PUFFER_DEFLATE_NONE_TICKS)
        {
            puff -= 1;
            self.set_puff_state(puff);
            self.make_sound(Some(PUFFER_BLOW_OUT_SOUND));
            if puff == NOT_PUFFED {
                state.inflate_ticks = 0;
            }
        }
    }

    /// Vanilla `PufferfishEntity.tickMovement` poison sting.
    fn tick_sting(&self, shared: &SharedEntity) {
        let state = self.puff_state();
        if !is_puffed(state) {
            return;
        }
        let Some(world) = self.level() else {
            return;
        };
        let search = self.base.bounding_box().inflate(PUFFER_STING_EXPANSION);
        let victims = world.get_entities_in_aabb_matching(&search, |entity| {
            entity.is_alive() && entity.as_living_entity().is_some()
        });
        if victims.is_empty() {
            return;
        }

        let source = Mob::mob_attack_damage_source(self, &ItemStack::empty(), shared);
        for victim in victims {
            if victim.hurt(&world, &source, sting_damage(state)) {
                self.make_sound(Some(PUFFER_STING_SOUND));
                if let Some(living) = victim.as_living_entity() {
                    living.add_mob_effect(
                        MobEffectInstance::with_duration(
                            POISON,
                            PUFFER_POISON_TICKS_PER_STATE * state,
                            0,
                        ),
                    );
                }
            }
        }
    }
}

impl Entity for PufferfishEntity {
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
        self.tick_puff_state();
    }

    fn dimensions_for_pose(&self, _pose: EntityPose) -> EntityDimensions {
        let scale = LivingEntity::get_scale(self) * puff_scale(self.puff_state());
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
        nbt.insert("PuffState", self.puff_state());
    }

    fn load_additional(&self, nbt: borrow::NbtCompound<'_, '_>) {
        self.load_mob(nbt);
        if let Some(state) = nbt.int("PuffState") {
            self.set_puff_state(state.clamp(NOT_PUFFED, FULLY_PUFFED));
        }
    }
}

impl LivingEntity for PufferfishEntity {
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
        self.tick_sting(entity);
        result
    }

    fn hurt_sound(&self, _source: &DamageSource) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_PUFFER_FISH_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_PUFFER_FISH_DEATH)
    }

    fn base_experience_reward(&self) -> i32 {
        rand::random_range(WATER_ANIMAL_EXPERIENCE_MIN..WATER_ANIMAL_EXPERIENCE_MAX)
    }
}

impl Mob for PufferfishEntity {
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

    fn ambient_sound_interval(&self) -> i32 {
        WATER_ANIMAL_AMBIENT_SOUND_INTERVAL
    }
}

impl PathfinderMob for PufferfishEntity {}

/// Vanilla `FishEntity.getLimitPerChunk()` inherited by the pufferfish.
#[must_use]
pub const fn pufferfish_limit_per_chunk() -> i32 {
    FISH_LIMIT_PER_CHUNK
}

impl Bucketable for PufferfishEntity {
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
        &vanilla_items::PUFFERFISH_BUCKET
    }
}

#[cfg(test)]
mod tests {
    use super::{FULLY_PUFFED, NOT_PUFFED, SEMI_PUFFED, is_puffed, puff_scale, sting_damage};

    #[test]
    fn puff_states_are_vanilla_constants() {
        assert_eq!(NOT_PUFFED, 0);
        assert_eq!(SEMI_PUFFED, 1);
        assert_eq!(FULLY_PUFFED, 2);
        assert!(!is_puffed(NOT_PUFFED));
        assert!(is_puffed(SEMI_PUFFED));
        assert!(is_puffed(FULLY_PUFFED));
    }

    #[test]
    fn puff_scales_match_vanilla() {
        assert!((puff_scale(NOT_PUFFED) - 0.5).abs() < f32::EPSILON);
        assert!((puff_scale(SEMI_PUFFED) - 0.7).abs() < f32::EPSILON);
        assert!((puff_scale(FULLY_PUFFED) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn sting_damage_scales_with_puff_state() {
        assert!((sting_damage(NOT_PUFFED) - 1.0).abs() < f32::EPSILON);
        assert!((sting_damage(SEMI_PUFFED) - 2.0).abs() < f32::EPSILON);
        assert!((sting_damage(FULLY_PUFFED) - 3.0).abs() < f32::EPSILON);
    }
}
