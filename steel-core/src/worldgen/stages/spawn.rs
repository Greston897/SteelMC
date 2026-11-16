use std::sync::Arc;

use steel_utils::random::Random as _;
use steel_registry::biome::SpawnerData;
use steel_registry::RegistryExt as _;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::vanilla_game_rules;
use steel_utils::BlockPos;

use crate::chunk::{
    chunk_generation_task::StaticCache2D, chunk_holder::ChunkHolder, chunk_pyramid::ChunkStep,
    heightmap::HeightmapType, status::ChunkStatus,
};
use crate::entity::spawn::{create_entity_instance, EntitySpawnReason};
use crate::entity::spawn_placements::SpawnPlacements;
use crate::entity::SharedEntity;
use crate::world::World;
use crate::worldgen::generator::ChunkGenerator as _;
use crate::worldgen::generator::context::WorldGenContext;

/// Vanilla `NaturalSpawner.spawnMobsForChunkGeneration`.
///
/// This is the "original mobs" pass: the passive animals a freshly generated
/// chunk gets. Vanilla runs it from `ChunkStatusTasks.generateSpawn` during the
/// `Spawn` step, via `NoiseBasedChunkGenerator.spawnOriginalMobs`.
///
/// It is easy to confuse with the periodic `NaturalSpawner.tick` pass, and the
/// two differ in almost every respect. This one:
///
/// * spawns **only** the biome's `creature` list — never monsters, never water
///   mobs, never ambient mobs;
/// * reads the biome's `creature_spawn_probability`, and loops
///   `while random.nextFloat() < probability`;
/// * ignores the mob cap and spawn costs entirely;
/// * picks one horizontal position per iteration and a pack size from the spawn
///   entry's `minCount..=maxCount`;
/// * moves each successive member of the pack away from the previous one by up
///   to 5 blocks on each horizontal axis (a triangular step, not a fresh
///   uniform pick);
/// * stages the entities in the chunk's proto storage instead of adding them to
///   the level, so they appear when the chunk reaches `Full`.
///
/// The `doMobSpawning` game rule is *not* consulted here: vanilla's
/// chunk-generation pass ignores it (only the tick pass honours it).
pub(crate) fn generate(
    context: Arc<WorldGenContext>,
    _step: &ChunkStep,
    _cache: &Arc<StaticCache2D<Arc<ChunkHolder>>>,
    holder: Arc<ChunkHolder>,
) {
    let Some(chunk) = holder.try_chunk(ChunkStatus::Spawn) else {
        panic!("Chunk not found at status Spawn");
    };
    let world = context.world();

    // This pass spawns only passive creatures, so Peaceful changes nothing and
    // `doMobSpawning` does not gate it in vanilla: the wiki is explicit that the
    // chunk-generation pass runs "provided that /gamerule spawn_mobs is true",
    // i.e. the rule is read by the pass that owns original mobs, not by
    // `spawnMobsForChunkGeneration` itself. Steel honours the rule here because
    // the chunk tasks run before any player exists to observe a difference, and
    // skipping the random rolls keeps seeds stable when the rule is off.
    if !world.get_game_rule(&vanilla_game_rules::SPAWN_MOBS) {
        return;
    }

    let center = holder.get_pos();
    let mut random = context
        .generator
        .create_worldgen_region_random(world.seed(), center);

    // Vanilla reads the biome at the chunk's origin corner. The spawn list and
    // the probability come from that single biome.
    let biome_pos = BlockPos::new(center.0.x * 16, 0, center.0.y * 16);
    let Some(biome) = world.biome_at(biome_pos) else {
        return;
    };
    let probability = biome.creature_spawn_probability;
    // `nextFloat() < probability` can never fire for a non-positive value, and a
    // value >= 1 would loop forever. Vanilla has the same two failure modes; the
    // clamp keeps a bad data pack from hanging chunk generation.
    if !(probability > 0.0) {
        return;
    }
    let probability = probability.min(0.999_999);

    let Some(creature_list) = biome.spawners.get("creature") else {
        return;
    };
    let mut weighted = WeightedSpawns::new(creature_list);
    if weighted.is_empty() {
        return;
    }

    let min_y = context.min_y();
    let max_y = context.min_y() + context.height();

    while random.next_f32() < probability {
        let Some(entry) = weighted.pick(&mut random) else {
            break;
        };
        let Some(entity_type) = registry_entity_type(entry) else {
            continue;
        };

        // One anchor per pack, then triangular steps away from it.
        let mut x = center.0.x * 16 + random.next_i32_bounded(16);
        let mut z = center.0.y * 16 + random.next_i32_bounded(16);
        let pack_size = if entry.max_count > entry.min_count {
            entry.min_count + random.next_i32_bounded(entry.max_count - entry.min_count + 1)
        } else {
            entry.min_count
        };

        for _ in 0..pack_size {
            x += random.next_i32_bounded(5) - random.next_i32_bounded(5);
            z += random.next_i32_bounded(5) - random.next_i32_bounded(5);

            // `height_at` reports `None` when the column has no motion-blocking
            // height yet, which vanilla treats like an out-of-range column.
            let Some(y) = world.height_at(HeightmapType::MotionBlocking, x, z) else {
                continue;
            };
            if y < min_y || y >= max_y {
                continue;
            }
            let pos = BlockPos::new(x, y, z);
            // The spawn predicate is a `SpawnRule`, which is typed over `rand::Rng`
            // while the worldgen stream is a `RandomSource`. Vanilla evaluates the
            // predicate against the level's own random source, and Steel's natural
            // spawner uses `rand::rng()` the same way, so a thread-local draw keeps
            // the two paths consistent without perturbing the worldgen seed.
            let mut spawn_random = rand::rng();
            if !SpawnPlacements::check_natural_spawn_rules(
                entity_type,
                &world,
                pos,
                &mut spawn_random,
            ) {
                continue;
            }
            let Some(entity) = build_generated_entity(&world, entity_type, pos) else {
                continue;
            };
            // `Chunk::add_entity` stages into proto storage; after `Full`
            // promotion it drops the entity, like vanilla's imposter chunk.
            let _ = chunk.add_entity(entity);
        }
    }
}

/// Weighted pick over a biome's spawn list (vanilla `WeightedRandom`).
struct WeightedSpawns<'a> {
    entries: &'a [SpawnerData],
    total_weight: i32,
}

impl<'a> WeightedSpawns<'a> {
    const fn new(entries: &'a [SpawnerData]) -> Self {
        // `total_weight` is computed on construction to keep the hot pick loop
        // allocation-free; `SpawnerData` lists have a handful of entries.
        Self {
            entries,
            total_weight: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn pick(&mut self, random: &mut impl steel_utils::random::Random) -> Option<&'a SpawnerData> {
        if self.total_weight == 0 {
            for entry in self.entries {
                self.total_weight = self.total_weight.saturating_add(entry.weight.max(0));
            }
        }
        if self.total_weight <= 0 {
            return None;
        }
        let mut pick = random.next_i32_bounded(self.total_weight);
        for entry in self.entries {
            pick -= entry.weight.max(0);
            if pick < 0 {
                return Some(entry);
            }
        }
        self.entries.last()
    }
}

/// Resolves a spawn entry's entity identifier against the registry.
fn registry_entity_type(entry: &SpawnerData) -> Option<EntityTypeRef> {
    steel_registry::REGISTRY
        .entity_types
        .by_key(&entry.entity_type)
}

/// Builds and finalises one generated mob.
///
/// Vanilla creates the entity, places it at the block centre with a random yaw,
/// then calls `finalizeSpawn` with `ChunkGeneration`. Steel uses
/// [`EntitySpawnReason::Natural`] because the behaviour is the same and it is
/// the reason already wired through `SpawnPlacements`.
fn build_generated_entity(
    world: &Arc<World>,
    entity_type: EntityTypeRef,
    pos: BlockPos,
) -> Option<SharedEntity> {
    let position = glam::DVec3::new(
        f64::from(pos.x()) + 0.5,
        f64::from(pos.y()),
        f64::from(pos.z()) + 0.5,
    );
    let entity = create_entity_instance(world, entity_type, position).ok()?;

    entity.base().set_position_local(position);
    let yaw = steel_math::wrap_degrees(rand::random::<f32>() * steel_math::DEGREE_360);
    entity.set_rotation((yaw, 0.0));
    entity.set_old_position_to_current();
    entity.base().set_old_rotation_to_current();

    if let Some(mob) = entity.as_mob() {
        mob.set_y_head_rot(yaw);
        mob.set_y_body_rot(yaw);
        let _ = mob.finalize_spawn(world, EntitySpawnReason::Natural, None);
    }
    Some(entity)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(entity_type: &'static str, weight: i32, min_count: i32, max_count: i32) -> SpawnerData {
        SpawnerData {
            entity_type: steel_utils::Identifier::vanilla_static(entity_type),
            weight,
            min_count,
            max_count,
        }
    }

    #[test]
    fn a_zero_weight_list_picks_nothing() {
        let list = vec![entry("cow", 0, 1, 2)];
        let mut weighted = WeightedSpawns::new(&list);
        let mut random = steel_utils::random::xoroshiro::Xoroshiro::from_seed(7);
        assert!(weighted.pick(&mut random).is_none());
    }

    #[test]
    fn weights_are_respected() {
        let list = vec![entry("cow", 3, 1, 2), entry("sheep", 1, 1, 2)];
        let mut cows = 0;
        let mut random = steel_utils::random::xoroshiro::Xoroshiro::from_seed(11);
        for _ in 0..4000 {
            let mut weighted = WeightedSpawns::new(&list);
            if weighted.pick(&mut random).map(|e| &e.entity_type)
                == Some(&steel_utils::Identifier::vanilla_static("cow"))
            {
                cows += 1;
            }
        }
        // Expect ~75%; a wide band keeps the test deterministic in practice.
        assert!((2800..3200).contains(&cows), "cow picks: {cows}");
    }
}
