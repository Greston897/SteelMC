use std::sync::Arc;

use steel_registry::RegistryExt as _;
use steel_registry::biome::SpawnerData;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::vanilla_block_tags::BlockTag;
use steel_registry::vanilla_game_rules;
use steel_registry::{REGISTRY, vanilla_fluids};
use steel_utils::BlockPos;
use steel_utils::random::Random as _;

use crate::chunk::{
    Chunk, chunk_generation_task::StaticCache2D, chunk_holder::ChunkHolder,
    chunk_pyramid::ChunkStep, heightmap::HeightmapType, status::ChunkStatus,
};
use crate::entity::SharedEntity;
use crate::entity::spawn::{EntitySpawnReason, create_entity_instance};
use crate::world::World;
use crate::worldgen::generator::ChunkGenerator as _;
use crate::worldgen::generator::context::WorldGenContext;

/// Vanilla `NaturalSpawner.spawnMobsForChunkGeneration`.
///
/// Runs during the `Spawn` step directly on the chunk's `Light`-status proto data
/// (never querying the not-yet-published `Full` chunk map):
/// - reads the biome's `creature` spawner table and `creature_spawn_probability`;
/// - loops `while random.nextFloat() < probability`;
/// - picks a pack anchor inside the chunk and runs up to 4 clamped placement
///   attempts per pack member against the proto heightmap and block states;
/// - stages spawned mobs into the proto chunk's entity storage so they enter the
///   world when the chunk reaches `Full`.
pub(crate) fn generate(
    context: Arc<WorldGenContext>,
    _step: &ChunkStep,
    _cache: &Arc<StaticCache2D<Arc<ChunkHolder>>>,
    holder: Arc<ChunkHolder>,
) {
    // At entry to `Spawn`, the holder's published status is the preceding `Light` step.
    let Some(chunk) = holder.try_chunk(ChunkStatus::Light) else {
        panic!("Chunk not found at status Light for Spawn stage");
    };
    let world = context.world();

    if !world.get_game_rule(&vanilla_game_rules::SPAWN_MOBS) {
        return;
    }

    let center = holder.get_pos();
    let mut random = context
        .generator
        .create_worldgen_region_random(world.seed(), center);

    // Sample the top surface biome in the center of the proto chunk, matching
    // vanilla `NoiseBasedChunkGenerator.spawnOriginalMobs`.
    let min_y = context.min_y();
    let max_y = min_y + context.height();
    let sample_y = chunk
        .generation_height_at(HeightmapType::WorldSurface, 8, 8)
        .saturating_sub(1)
        .clamp(min_y, max_y.saturating_sub(1));
    let Some(biome_id) = proto_biome_id(chunk, min_y, 2, sample_y >> 2, 2) else {
        return;
    };
    let Some(biome) = REGISTRY.biomes.by_id(usize::from(biome_id)) else {
        return;
    };

    let probability = biome.creature_spawn_probability;
    if probability <= 0.0 || probability.is_nan() {
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

    let xo = center.0.x * 16;
    let zo = center.0.y * 16;

    while random.next_f32() < probability {
        let Some(entry) = weighted.pick(&mut random) else {
            break;
        };
        let Some(entity_type) = registry_entity_type(entry) else {
            continue;
        };
        if !entity_type.summonable {
            continue;
        }

        let pack_size = if entry.max_count > entry.min_count {
            entry.min_count + random.next_i32_bounded(entry.max_count - entry.min_count + 1)
        } else {
            entry.min_count
        };
        let start_x = xo + random.next_i32_bounded(16);
        let start_z = zo + random.next_i32_bounded(16);
        let mut x = start_x;
        let mut z = start_z;

        for _ in 0..pack_size {
            let mut spawned = false;
            for _ in 0..4 {
                let lx = (x - xo) as usize;
                let lz = (z - zo) as usize;
                let y = chunk.generation_height_at(HeightmapType::MotionBlockingNoLeaves, lx, lz);
                let pos = BlockPos::new(x, y, z);
                if y > min_y && y < max_y && is_valid_proto_creature_spawn(chunk, pos) {
                    let half_w = f64::from(entity_type.dimensions.width).clamp(0.0, 7.5);
                    let fx = (f64::from(x) + 0.5)
                        .clamp(f64::from(xo) + half_w, f64::from(xo) + 16.0 - half_w);
                    let fz = (f64::from(z) + 0.5)
                        .clamp(f64::from(zo) + half_w, f64::from(zo) + 16.0 - half_w);
                    let yaw = random.next_f32() * steel_math::DEGREE_360;
                    if let Some(entity) =
                        build_generated_entity(&world, entity_type, fx, f64::from(y), fz, yaw)
                    {
                        let _ = chunk.add_entity(entity);
                        spawned = true;
                    }
                }

                x += random.next_i32_bounded(5) - random.next_i32_bounded(5);
                z += random.next_i32_bounded(5) - random.next_i32_bounded(5);
                // Vanilla `NaturalSpawner.java:353-356`: re-roll around `(startX, startZ)`
                // until the candidate lands back inside this chunk.
                while x < xo || x >= xo + 16 || z < zo || z >= zo + 16 {
                    x = start_x + random.next_i32_bounded(5) - random.next_i32_bounded(5);
                    z = start_z + random.next_i32_bounded(5) - random.next_i32_bounded(5);
                }
                if spawned {
                    break;
                }
            }
        }
    }
}

/// Reads the biome palette id at local quart coordinates directly from `chunk`.
fn proto_biome_id(
    chunk: &Chunk,
    min_y: i32,
    local_quart_x: usize,
    quart_y: i32,
    local_quart_z: usize,
) -> Option<u16> {
    let sections = chunk.sections();
    let count = sections.sections.len();
    if count == 0 {
        return None;
    }
    let min_quart_y = min_y >> 2;
    let max_quart_y = min_quart_y + (count as i32) * 4 - 1;
    let clamped_y = quart_y.clamp(min_quart_y, max_quart_y);
    let offset = (clamped_y - min_quart_y) as usize;
    let section = &sections.sections[offset >> 2];
    Some(
        section
            .read()
            .biomes
            .get(local_quart_x & 3, offset & 3, local_quart_z & 3),
    )
}

/// Vanilla `Animal.checkAnimalSpawnRules` + `SpawnPlacementTypes.ON_GROUND`
/// evaluated against the proto chunk's block states.
fn is_valid_proto_creature_spawn(chunk: &Chunk, pos: BlockPos) -> bool {
    let below = chunk.get_block_state(pos.below());
    if !below.get_block().has_tag(&BlockTag::ANIMALS_SPAWNABLE_ON) {
        return false;
    }
    let at = chunk.get_block_state(pos);
    let above = chunk.get_block_state(pos.above());
    at.is_air()
        && at.get_fluid_state().fluid_id == &vanilla_fluids::EMPTY
        && above.get_fluid_state().fluid_id == &vanilla_fluids::EMPTY
}

/// Weighted pick over a biome's spawn list (vanilla `WeightedList`).
struct WeightedSpawns<'a> {
    entries: &'a [SpawnerData],
    total_weight: i32,
}

impl<'a> WeightedSpawns<'a> {
    const fn new(entries: &'a [SpawnerData]) -> Self {
        Self {
            entries,
            total_weight: 0,
        }
    }

    const fn is_empty(&self) -> bool {
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
    REGISTRY.entity_types.by_key(&entry.entity_type)
}

/// Builds and finalizes one chunk-generated creature.
fn build_generated_entity(
    world: &Arc<World>,
    entity_type: EntityTypeRef,
    x: f64,
    y: f64,
    z: f64,
    raw_yaw: f32,
) -> Option<SharedEntity> {
    let position = glam::DVec3::new(x, y, z);
    let entity = create_entity_instance(world, entity_type, position).ok()?;

    entity.base().set_position_local(position);
    let yaw = steel_math::wrap_degrees(raw_yaw);
    entity.set_rotation((yaw, 0.0));
    entity.set_old_position_to_current();
    entity.base().set_old_rotation_to_current();

    if let Some(mob) = entity.as_mob() {
        mob.set_y_head_rot(yaw);
        mob.set_y_body_rot(yaw);
        let _ = mob.finalize_spawn(world, EntitySpawnReason::ChunkGeneration, None);
    }
    Some(entity)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(
        entity_type: &'static str,
        weight: i32,
        min_count: i32,
        max_count: i32,
    ) -> SpawnerData {
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
        assert!((2800..3200).contains(&cows), "cow picks: {cows}");
    }
}
