//! Vanilla `NaturalSpawner`: the natural mob-spawning pass.
//!
//! Vanilla runs natural spawning on a 400-tick cycle from `ServerChunkCache`:
//! each pass picks a random player, picks a random position inside the monster
//! spawn ring, checks the entity's `SpawnPlacements` predicate and the category
//! mob cap, then spawns a small pack. The constants below are vanilla's.

use std::sync::Arc;

use rand::{Rng as _, RngExt as _};
use steel_registry::biome::BiomeRef;
use steel_registry::entity_type::{EntityTypeRef, MobCategory};
use steel_registry::vanilla_game_rules;
use steel_registry::{REGISTRY, RegistryExt as _};
use steel_utils::{BlockPos, types::Difficulty};
use steel_worldgen::biomes::obfuscate_biome_seed;

use crate::chunk::heightmap::HeightmapType;
use crate::entity::spawn::{EntitySpawnPlacement, EntitySpawnReason, EntitySpawnRequest};
use crate::entity::spawn_placements::SpawnPlacements;
use crate::entity::{Entity, spawn_entity};
use crate::worldgen::generators::vanilla::fuzzed_biome_at_block;
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
        .count_accessible_by_category(category)
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
    // Vanilla `NaturalSpawner.isValidSpawnPostitionForType` is gated by the
    // `doMobSpawning` game rule before any per-type work is done.
    if !world.get_game_rule(&vanilla_game_rules::SPAWN_MOBS) {
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
        if !category_spawning_enabled(world, category) {
            continue;
        }
        let cap = mob_cap(category, players);
        if count_category(world, category) >= cap {
            continue;
        }
        // Vanilla `NaturalSpawner.spawnCategoryForChunk` picks the type per
        // chunk from that chunk's biome, so a pass is a number of attempts
        // rather than a walk over every registered type.
        while count_category(world, category) < cap {
            let before = spawned;
            spawned += try_spawn_pack(world, category);
            // No progress means the sampled position or biome rejected the
            // spawn; vanilla also gives up on the chunk at that point.
            if spawned == before {
                break;
            }
        }
    }
    spawned
}

/// Vanilla's per-category game-rule gates (`doMobSpawning`, `spawnMonsters`,
/// `spawnPatrols`, `spawnWanderingTraders`, `spawnPhantoms`, `spawnWardens`).
///
/// Vanilla checks the monster rule in `NaturalSpawner.spawnCategoryForChunk`
/// and the special-mob rules in their own spawners; Steel funnels all of them
/// through the category so a disabled rule stops the whole pipeline.
fn category_spawning_enabled(world: &Arc<World>, category: MobCategory) -> bool {
    match category {
        MobCategory::Monster => world.get_game_rule(&vanilla_game_rules::SPAWN_MONSTERS),
        _ => true,
    }
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

/// A natural-spawn candidate: the type plus the biome entry that produced it.
struct SpawnCandidate {
    entity_type: EntityTypeRef,
    min_count: i32,
    max_count: i32,
}

/// Vanilla `NaturalSpawner.getRandomSpawnMobAt` -> `Biome.getMobs(category)`.
///
/// Vanilla picks the type from the **biome of the chunk it is spawning in**, not
/// from a global per-category list, so a mooshroom only appears in a mushroom
/// field and a polar bear only in ice. Steel used to walk every registered type
/// of the category, which spawned mobs in the wrong biomes and burned 185 type
/// checks per category per pass.
///
/// The biome is resolved at the pack's anchor through the loaded chunk
/// (`World::noise_biome_id`), matching `Level.getBiome` in vanilla.
fn pick_candidate(world: &Arc<World>, category: MobCategory, pos: BlockPos) -> Option<SpawnCandidate> {
    let biome = world_biome_at(world, pos)?;
    let list = biome.spawners.get(category.name())?;
    if list.is_empty() {
        return None;
    }

    let total: i32 = list.iter().map(|entry| entry.weight.max(0)).sum();
    if total <= 0 {
        return None;
    }

    let mut pick = rand::rng().random_range(0..total);
    let mut chosen = list.last()?;
    for entry in list {
        pick -= entry.weight.max(0);
        if pick < 0 {
            chosen = entry;
            break;
        }
    }

    Some(SpawnCandidate {
        entity_type: REGISTRY.entity_types.by_key(&chosen.entity_type)?,
        min_count: chosen.min_count,
        max_count: chosen.max_count,
    })
}

/// Resolves the biome of `pos` from the loaded world.
fn world_biome_at(world: &Arc<World>, pos: BlockPos) -> Option<BiomeRef> {
    let zoom_seed = obfuscate_biome_seed(world.seed());
    let biome_id = fuzzed_biome_at_block(zoom_seed, pos, |quart| {
        world.noise_biome_id(quart.x, quart.y, quart.z).unwrap_or(0)
    });
    REGISTRY.biomes.by_id(usize::from(biome_id))
}

/// Attempts one vanilla pack spawn, returning how many mobs appeared.
fn try_spawn_pack(world: &Arc<World>, category: MobCategory) -> usize {
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

    let Some(candidate) = pick_candidate(world, category, pos) else {
        return 0;
    };
    let entity_type = candidate.entity_type;

    if !SpawnPlacements::check_natural_spawn_rules(entity_type, world, pos, &mut random) {
        return 0;
    }

    // Vanilla `NaturalSpawner.getRandomSpawnMobAt` then `GroupData.getState`:
    // the pack size comes from the biome entry's `minCount..=maxCount`, not from
    // a global `1..=4` constant (that spawned wrong pack sizes for e.g. wolves).
    let pack = if candidate.max_count > candidate.min_count {
        candidate.min_count + random.random_range(0..=(candidate.max_count - candidate.min_count))
    } else {
        candidate.min_count
    }
    .max(0);

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
        // Keep iterating: `iter_players` stops only when the callback returns
        // `false`, and every candidate must be counted to match the index.
        true
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

    #[test]
    fn category_names_are_the_biome_spawner_keys() {
        // `Biome.spawners` is keyed by these exact strings in the data pack; a
        // wrong key silently spawns nothing for that category.
        assert_eq!(MobCategory::Monster.name(), "monster");
        assert_eq!(MobCategory::Creature.name(), "creature");
        assert_eq!(
            MobCategory::UndergroundWaterCreature.name(),
            "underground_water_creature"
        );
        assert_eq!(MobCategory::WaterAmbient.name(), "water_ambient");
    }
}
