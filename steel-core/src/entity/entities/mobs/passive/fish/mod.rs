//! Shared scaffolding for vanilla water animals and fish.
//!
//! Vanilla models this family with four layers that Steel had no equivalent
//! for yet:
//!
//! * `WaterAnimal` — air/drowning handling (`MAX_AIR = 300`, `DROWN` damage),
//!   the `[seaLevel - 13, seaLevel]` spawn window and the water path-node malus,
//! * `FishEntity` (`AbstractFish`) — land flopping, bucket bookkeeping
//!   (`FROM_BUCKET`), `PathType::Water` malus and the fish spawn limit,
//! * `SchoolingFish` (`AbstractSchoolingFish`) — leader/group bookkeeping
//!   shared by Cod/Salmon/TropicalFish,
//! * `AgeableWaterCreature` — deliberately *not* modelled yet; `Squid` keeps
//!   its own copy of the ambient-sound interval until the layer exists.
//!
//! The free functions below are the layer methods that can be expressed
//! without knowing the concrete entity type; each fish entity forwards its
//! trait methods to them, mirroring how the vanilla hierarchy splits the work.

#![allow(
    dead_code,
    reason = "the water-mob pack lands incrementally; helpers are consumed as each fish entity arrives"
)]
pub mod cod;
pub mod salmon;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use glam::DVec3;
use rand::Rng;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::fluid::is_water_fluid;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_damage_types;
use steel_utils::BlockPos;

use crate::entity::damage::DamageSource;
use crate::entity::{Entity, EntitySpawnReason, LivingEntity, SharedEntity};
use crate::physics::{MoveResult, MoverType};
use crate::world::LevelReader;

/// Vanilla `WaterAnimal.MAX_AIR`.
pub const WATER_ANIMAL_MAX_AIR: i32 = 300;
/// Vanilla `WaterAnimal.SPAWN_DEPTH_OFFSET` — the fish spawn window is
/// `[seaLevel - 13, seaLevel]`.
pub const WATER_ANIMAL_SPAWN_DEPTH_OFFSET: i32 = 13;
/// Vanilla `WaterAnimal.DROWN_DAMAGE` (`2.0`).
pub const WATER_ANIMAL_DROWN_DAMAGE: f32 = 2.0;
/// Vanilla `WaterAnimal` ambient sound interval (`getAmbientSoundInterval`).
pub const WATER_ANIMAL_AMBIENT_SOUND_INTERVAL: i32 = 120;
/// Vanilla `WaterAnimal` experience drop: `1 + random(3)`.
pub const WATER_ANIMAL_EXPERIENCE_MIN: i32 = 1;
/// Vanilla `WaterAnimal` experience drop upper bound (exclusive).
pub const WATER_ANIMAL_EXPERIENCE_MAX: i32 = 4;

/// Vanilla `FishEntity` spawn cap (`getLimitPerChunk`).
pub const FISH_LIMIT_PER_CHUNK: i32 = 8;
/// Vanilla `FishEntity.MAX_HEALTH`.
pub const FISH_MAX_HEALTH: f64 = 3.0;
/// Vanilla `FishEntity` movement speed (from `createFishAttributes`).
pub const FISH_MOVEMENT_SPEED: f64 = 0.7;
/// Vanilla `FishEntity` panic speed modifier.
pub const FISH_PANIC_SPEED: f64 = 1.25;
/// Vanilla `FishEntity` flee speed modifier.
pub const FISH_FLEE_SPEED: f64 = 1.6;
/// Vanilla `FishEntity` flee distance (`8.0F`).
pub const FISH_FLEE_DISTANCE: f32 = 8.0;
/// Vanilla `FishEntity` flee y-range (`1.6`).
pub const FISH_FLEE_Y_RANGE: f64 = 1.6;
/// Vanilla `FishEntity` flee path distance (`1.4`).
pub const FISH_FLEE_PATH_DISTANCE: f64 = 1.4;
/// Vanilla `SwimToRandomPlaceGoal` speed modifier.
pub const FISH_SWIM_TO_RANDOM_PLACE_SPEED: f64 = 1.0;
/// Vanilla `SwimToRandomPlaceGoal` interval.
pub const FISH_SWIM_TO_RANDOM_PLACE_INTERVAL: i32 = 40;
/// Vanilla fish water drag after a self-movement step.
pub const FISH_WATER_DRAG: f64 = 0.9;
/// Vanilla fish acceleration while swimming (`updateVelocity(0.01F, input)`).
pub const FISH_WATER_ACCELERATION: f32 = 0.01;
/// Vanilla `FishEntity` flop impulse.
pub const FISH_FLOP_IMPULSE: f64 = 0.4;
/// Vanilla `FishEntity` flop horizontal impulse bounds.
pub const FISH_FLOP_HORIZONTAL: f64 = 0.05;
/// Vanilla fish downward bias while the navigation is idle (`-0.005`).
pub const FISH_WATER_SINKING_BIAS: f64 = 0.005;
/// Vanilla `FishEntity` `FROM_BUCKET` synced-data default.
pub const FISH_FROM_BUCKET_DEFAULT: bool = false;
/// Vanilla `SchoolingFish` leader proximity (squared distance `121.0`).
pub const SCHOOLING_CLOSE_ENOUGH_SQUARED: f64 = 121.0;
/// Vanilla `SchoolingFish` group-refresh radius (`8.0`).
pub const SCHOOLING_GROUP_RADIUS: f64 = 8.0;
/// Vanilla `SchoolingFish` group-refresh chance (`random.nextInt(200) == 1`).
pub const SCHOOLING_REFRESH_CHANCE: i32 = 200;
/// Vanilla `SchoolingFish` follow-leader navigation speed.
pub const SCHOOLING_FOLLOW_SPEED: f64 = 1.0;
/// Vanilla `SchoolingFish` group-refresh interval (every tick, guarded by the
/// `random.nextInt(200) == 1` roll).
pub const SCHOOLING_GROUP_SIZE_DEFAULT: i32 = 1;

/// Vanilla `WaterAnimal.canSpawn`.
///
/// The check is purely positional: the entity has to spawn between
/// `seaLevel - 13` and `seaLevel` inside water.
pub fn check_water_animal_spawn_rules(
    _entity_type: EntityTypeRef,
    level: &dyn LevelReader,
    _spawn_reason: EntitySpawnReason,
    pos: BlockPos,
    _random: &mut dyn Rng,
) -> bool {
    is_within_spawn_depth(level.sea_level(), pos.y())
        && is_water_fluid(level.get_block_state(pos.below()).get_fluid_state().fluid_id)
        && level.get_block_state(pos.above()).get_block() == &steel_registry::vanilla_blocks::WATER
}

/// Vanilla `WaterAnimal.canSpawn` depth window: `[seaLevel - 13, seaLevel]`.
#[must_use]
pub const fn is_within_spawn_depth(sea_level: i32, y: i32) -> bool {
    y >= sea_level - WATER_ANIMAL_SPAWN_DEPTH_OFFSET && y <= sea_level
}

/// Vanilla `WaterAnimal.baseTick` breathing step.
///
/// `pre_tick_air_supply` has to be captured *before* the mob base tick runs,
/// exactly like vanilla snapshots `getAir()` before calling `super.baseTick()`.
pub fn tick_breathing<E>(entity: &E, pre_tick_air_supply: i32)
where
    E: LivingEntity + ?Sized,
{
    if LivingEntity::is_alive(entity) && !entity.is_in_water() {
        entity.set_air_supply(pre_tick_air_supply - 1);
        if entity.should_take_drowning_damage() {
            entity.set_air_supply(0);
            if let Some(world) = entity.level() {
                entity.hurt(
                    &world,
                    &DamageSource::environment(&vanilla_damage_types::DROWN),
                    WATER_ANIMAL_DROWN_DAMAGE,
                );
            }
        }
    } else {
        entity.set_air_supply(WATER_ANIMAL_MAX_AIR);
    }
}

/// Vanilla `FishEntity.tickMovement` land-flop.
///
/// Applies the random horizontal impulse plus the fixed `0.4` vertical
/// impulse, clears the on-ground flag and plays the flop sound.
pub fn flop<E>(entity: &E, flop_sound: SoundEventRef)
where
    E: Entity + ?Sized,
{
    entity.set_velocity(flop_velocity(
        rand::random_range(-FISH_FLOP_HORIZONTAL..FISH_FLOP_HORIZONTAL),
        rand::random_range(-FISH_FLOP_HORIZONTAL..FISH_FLOP_HORIZONTAL),
    ));
    entity.set_on_ground(false);
    entity.make_sound(Some(flop_sound));
}

/// Vanilla `FishEntity.tickMovement` flop impulse: `(x, 0.4, z)` with the
/// random horizontal components supplied by the caller.
#[must_use]
pub const fn flop_velocity(horizontal_x: f64, horizontal_z: f64) -> DVec3 {
    DVec3::new(horizontal_x, FISH_FLOP_IMPULSE, horizontal_z)
}

/// Vanilla `FishEntity.travelInWater` drag step: `velocity * 0.9`, minus
/// `0.005` on the Y axis while the navigation is idle.
#[must_use]
pub fn fish_water_drag(velocity: DVec3, navigation_idle: bool) -> DVec3 {
    let mut movement = velocity * FISH_WATER_DRAG;
    if navigation_idle {
        movement.y -= FISH_WATER_SINKING_BIAS;
    }
    movement
}

/// Vanilla `FishEntity.travelInWater`.
///
/// Fish do not use the generic `LivingEntity.travelInWater`: they accelerate
/// with a fixed `0.01` factor, ignore buoyancy/dolphin's-grace and only apply
/// the `0.9` drag, plus a small downward bias while the navigation is idle.
pub fn travel_in_water(
    entity: &impl LivingEntity,
    shared: &SharedEntity,
    input: DVec3,
    navigation_idle: bool,
) -> Option<MoveResult> {
    entity.move_relative(f64::from(FISH_WATER_ACCELERATION), input);
    let result = Arc::clone(shared).move_entity(MoverType::SelfMovement, entity.velocity())?;

    entity.set_velocity(fish_water_drag(entity.velocity(), navigation_idle));

    Some(result)
}
