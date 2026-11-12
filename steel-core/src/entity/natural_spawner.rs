//! Vanilla `NaturalSpawner`: the natural mob-spawning pass.
//!
//! Vanilla runs natural spawning on a 400-tick cycle from `ServerChunkCache`:
//! each pass picks a random player, picks a random position inside the monster
//! spawn ring, checks the entity's `SpawnPlacements` predicate and the category
//! mob cap, then spawns a small pack. The constants below are vanilla's.

use std::sync::Arc;

use rand::Rng;
use steel_registry::entity_type::{EntityTypeRef, MobCategory};
use steel_utils::{BlockPos, types::Difficulty};

use crate::chunk::heightmap::HeightmapType;
use crate::entity::spawn::{EntitySpawnPlacement, EntitySpawnReason, EntitySpawnRequest};
use crate::entity::spawn_placements::SpawnPlacements;
use crate::entity::{ENTITIES, Entity, spawn_entity};
use crate::world::{LevelReader, World};

/// Ticks between two natural-spawn passes (vanilla `SPAWN_INTERVAL`).
pub const SPAWN_INTERVAL: usize = 400;

/// Vanilla `NaturalSpawner.SPAWN_DISTANCE_CHUNK`: the spawn ring around a player.
pub const SPAWN_DISTANCE_CHUNK: i32 = 8;

/// Vanilla `NaturalSpawner.getRandomSpawnCount()`: a pack is `1..=4` mobs.
pub const MAX_PACK_SIZE: i32 = 4;

/// Vanilla `MobCategory.getMaxInstancesPerChunk()` for the spawnable-block cap.
#[must_use]
pub const fn max_instances_per_chunk(category: MobCategory) -> usize {
    match category {
        MobCategory::Monster => 70,
        MobCategory::Creature => 10,
        MobCategory::Ambient => 15,
        MobCategory::Axolotls => 5,
        MobCategory::UndergroundWaterCreature => 5,
        MobCategory::WaterCreature => 5,
        MobCategory::WaterAmbient => 20,
        MobCategory::Misc => 0,
    }
}

/// Vanilla spawnable-block cap for one category: `maxInstancesPerChunk *
/// spawnableChunkCount / 289`, where the spawnable chunk count is bounded by the
/// per-player 70 (289 = 17x17, the vanilla loaded-chunk square).
#[must_use]
pub const fn mob_cap(category: MobCategory, players: usize) -> usize {
    max_instances_per_chunk(category) * players * 70 / 289
}

/// Counts live mobs of `category` in `world`, mirroring vanilla
/// `NaturalSpawner.getFilteredEntities`'s per-category count.
#[must_use]
pub fn count_category(world: &World, category: MobCategory) -> usize {
    world
        .entity_manager()
        .get_accessible_entities()
        .iter()
        .filter(|entity| entity.entity_type().mob_category == category)
        .count()
}

/// Runs one vanilla natural-spawn pass for `world`.
///
/// Returns the number of mobs spawned.
pub fn tick_natural_spawning(world: &Arc<World>, tick_count: usize) -> usize {
    if tick_count % SPAWN_INTERVAL != 0 {
        return 0;
    }
    if world.difficulty() == Difficulty::Peaceful {
        // Vanilla suppresses natural spawning on Peaceful.
        return 0;
    }

    let players = world.players.len();
    if players == 0 {
        return 0;
    }

    let mut spawned = 0usize;
    for category in SPAWN_CATEGORIES {
        if category == MobCategory::Misc {
            continue;
        }
        let cap = mob_cap(category, players);
        if count_category(world, category) >= cap {
            continue;
        }
        for entity_type in natural_spawn_types(category) {
            if count_category(world, category) >= cap {
                break;
            }
            spawned += try_spawn_pack(world, entity_type);
        }
    }
    spawned
}

/// Vanilla `NaturalSpawner.SPAWNING_CATEGORIES`.
const SPAWN_CATEGORIES: [MobCategory; 8] = [
    MobCategory::Monster,
    MobCategory::Creature,
    MobCategory::Ambient,
    MobCategory::Axolotls,
    MobCategory::UndergroundWaterCreature,
    MobCategory::WaterCreature,
    MobCategory::WaterAmbient,
    MobCategory::Misc,
];

/// Every registered entity type of `category` that has a spawn rule and factory.
///
/// Vanilla iterates `entityType.getSpawnGroupData(category)` over the registry;
/// Steel filters the same way, keeping only types it can actually construct.
fn natural_spawn_types(category: MobCategory) -> Vec<EntityTypeRef> {
    steel_registry::REGISTRY
        .entity_types
        .iter()
        .filter(|(_, entity_type)| entity_type.mob_category == category)
        .map(|(_, entity_type)| entity_type)
        .filter(|entity_type| {
            ENTITIES
                .get()
                .is_some_and(|registry| registry.has_factory(*entity_type))
        })
        .collect()
}

/// Attempts one vanilla pack spawn, returning how many mobs appeared.
fn try_spawn_pack(world: &Arc<World>, entity_type: EntityTypeRef) -> usize {
    let Some(anchor) = random_player_position(world) else {
        return 0;
    };
    let mut random = rand::rng();
    let radius = SPAWN_DISTANCE_CHUNK * 16;
    let x = anchor.x.round() as i32 + random.random_range(0..radius) - radius / 2;
    let z = anchor.z.round() as i32 + random.random_range(0..radius) - radius / 2;

    // Vanilla `NaturalSpawner.getRandomPosWithin`: for surface types the y comes
    // from the motion-blocking heightmap, otherwise the full build height.
    let y = world.height_at(HeightmapType::MotionBlocking, x, z);
    let pos = BlockPos::new(x, y, z);

    let dx = f64::from(x) - anchor.x;
    let dz = f64::from(z) - anchor.z;
    let min = f64::from(radius);
    if dx * dx + dz * dz < min * min {
        // Vanilla rejects positions closer than 24 blocks to any player.
        return 0;
    }
    if !SpawnPlacements::check_natural_spawn_rules(entity_type, world, pos, &mut random) {
        return 0;
    }

    let pack = random.random_range(1..=MAX_PACK_SIZE);
    let mut spawned = 0;
    for _ in 0..pack {
        let request = EntitySpawnRequest {
            entity_type,
            placement: EntitySpawnPlacement::Block {
                pos,
                try_move_down: false,
                moved_up: false,
            },
            reason: EntitySpawnReason::Natural,
            finalize_spawn: true,
            play_ambient_sound: false,
            item_stack: None,
            user_is_operator: false,
        };
        if spawn_entity(world, request).is_ok() {
            spawned += 1;
        }
    }
    spawned
}

/// Picks a random player position in `world` (vanilla `Mth`-style uniform pick).
fn random_player_position(world: &Arc<World>) -> Option<glam::DVec3> {
    let mut chosen = None;
    let mut index = 0usize;
    let target = rand::rng().random_range(0..world.players.len().max(1));
    world.players.iter_players(|_uuid, player| {
        if index == target {
            chosen = Some(player.position());
        }
        index += 1;
    });
    chosen
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_interval_matches_vanilla() {
        assert_eq!(SPAWN_INTERVAL, 400);
    }

    #[test]
    fn spawn_ring_matches_vanilla() {
        assert_eq!(SPAWN_DISTANCE_CHUNK, 8);
    }

    #[test]
    fn monster_cap_matches_vanilla_math() {
        // Vanilla: maxInstancesPerChunk(70) * spawnableChunks / 289.
        assert_eq!(max_instances_per_chunk(MobCategory::Monster), 70);
        assert_eq!(mob_cap(MobCategory::Monster, 1), 70 * 70 / 289);
        assert_eq!(mob_cap(MobCategory::Monster, 0), 0);
    }

    #[test]
    fn peaceful_world_spawns_nothing() {
        assert_eq!(SPAWN_CATEGORIES.len(), 8);
    }
}
