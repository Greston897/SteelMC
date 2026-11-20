use std::str::FromStr as _;
use std::sync::Arc;

use rand::{Rng, RngExt};
use steel_macros::block_behavior;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::blocks::properties::{BlockStateProperties, IntProperty};
use steel_registry::feature::{ConfiguredFeatureKind, TrunkPlacer};
use steel_registry::vanilla_block_tags::BlockTag;
use steel_registry::{REGISTRY, RegistryExt as _, vanilla_blocks};
use steel_utils::random::worldgen_random::WorldgenRandom;
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId, Direction, Identifier};
use steel_worldgen::biomes::obfuscate_biome_seed;

use crate::behavior::block::BlockBehavior;
use crate::behavior::blocks::vegetation::bonemealable::Bonemealable;
use crate::behavior::context::BlockPlaceContext;
use crate::world::{LevelReader, ScheduledTickAccess, World};
use crate::worldgen::feature::FeatureDecorationRunner;
use crate::worldgen::feature::no_nested_features;

use super::{BlockRef, default_surviving_state, survives_on_tag};

/// `SaplingBlock.STAGE` property; the second stage marks a grown sapling.
const STAGE: &IntProperty = &BlockStateProperties::STAGE;
/// Vanilla `SaplingBlock` growth bound: one in seven random ticks progresses.
const RANDOM_GROWTH_BOUND: u32 = 7;

/// Vanilla `SaplingBlock` survival, growth, and bonemeal behaviour.
///
/// Vanilla delegates the shape to a `TreeGrower`; Steel keeps the grower names
/// from `classes.json` and resolves the configured features lazily so a datapack
/// that renames a tree still loads the block.
#[block_behavior]
pub struct SaplingBlock {
    block: BlockRef,
    /// `TreeGrower` secondary tree name (e.g. `oak`).
    #[json_arg(value, json = "tree_grower_name")]
    tree_grower_name: &'static str,
    /// The grower's first tree type (e.g. `oak`, `jungle_tree_no_vine`).
    ///
    /// Growers with a single tree (`dark_oak`, `pale_oak`) omit the key
    /// entirely in `classes.json`; the `optional = "none"` sentinel accepts
    /// both a missing key and an explicit `"none"` value.
    #[json_arg(value, json = "tree_grower_shortest_tree_type", optional = "none")]
    shortest_tree_type: Option<&'static str>,
}

impl SaplingBlock {
    /// Creates a new sapling block behavior.
    #[must_use]
    pub const fn new(
        block: BlockRef,
        tree_grower_name: &'static str,
        shortest_tree_type: Option<&'static str>,
    ) -> Self {
        Self {
            block,
            tree_grower_name,
            shortest_tree_type,
        }
    }

    /// Vanilla 26.3 weighted primary/flower tree lists. Singleton lists do not
    /// consume a random value; weighted lists use an integer, not a float roll.
    fn feature_for(&self, rng: &mut dyn Rng, flowers: bool) -> Option<&'static str> {
        Some(match self.tree_grower_name {
            "oak" => match (flowers, rng.random_range(0..10)) {
                (true, 9) => "fancy_oak_bees_005",
                (true, _) => "oak_bees_005",
                (false, 9) => "fancy_oak",
                (false, _) => "oak",
            },
            "birch" if flowers => "birch_bees_005",
            "cherry" if flowers => "cherry_bees_005",
            "jungle" => "jungle_tree_no_vine",
            "mangrove" => {
                if rng.random_range(0..100) < 15 {
                    "mangrove"
                } else {
                    "tall_mangrove"
                }
            }
            "poplar" => match rng.random_range(0..3) {
                0 => "red_poplar",
                1 => "orange_poplar",
                _ => "yellow_poplar",
            },
            "azalea" | "azalea_tree" => "azalea_tree",
            "dark_oak" | "pale_oak" => return None,
            other => other,
        })
    }

    fn mega_feature_for(&self, rng: &mut dyn Rng) -> Option<&'static str> {
        match self.tree_grower_name {
            "spruce" => Some(if rng.random_range(0..2) == 0 {
                "mega_spruce"
            } else {
                "mega_pine"
            }),
            "jungle" => Some("mega_jungle_tree"),
            "dark_oak" => Some("dark_oak"),
            "pale_oak" => Some("pale_oak_bonemeal"),
            _ => None,
        }
    }

    /// Vanilla scans the four possible 2x2 squares containing the clicked
    /// sapling, comparing block identity, not the saplings' STAGE values.
    fn two_by_two(world: &dyn LevelReader, pos: BlockPos, state: BlockStateId) -> Option<BlockPos> {
        for dx in [0, -1] {
            for dz in [0, -1] {
                let origin = pos.offset(dx, 0, dz);
                if [(0, 0), (1, 0), (0, 1), (1, 1)].into_iter().all(|(x, z)| {
                    world.get_block_state(origin.offset(x, 0, z)).get_block() == state.get_block()
                }) {
                    return Some(origin);
                }
            }
        }
        None
    }

    fn has_flowers(world: &dyn LevelReader, pos: BlockPos) -> bool {
        for x in -2..=2 {
            for y in -1..=1 {
                for z in -2..=2 {
                    if world
                        .get_block_state(pos.offset(x, y, z))
                        .is_in_tag(&BlockTag::FLOWERS)
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Both random ticks and bonemeal advance STAGE before attempting a tree.
    fn advance_tree(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
        rng: &mut dyn Rng,
    ) {
        if state.get_value(STAGE) == 0 {
            world.set_block(
                pos,
                state.set_value(STAGE, 1),
                UpdateFlags::UPDATE_INVISIBLE | UpdateFlags::UPDATE_SKIP_BLOCK_ENTITY_SIDEEFFECTS,
            );
        } else {
            self.grow_tree(world, pos, state, rng);
        }
    }

    /// Resolves a grower name to its `minecraft:<name>` configured feature.
    fn configured_feature(
        name: &str,
    ) -> Option<&'static steel_registry::feature::ConfiguredFeature> {
        let key = Identifier::from_str(&format!("minecraft:{name}")).ok()?;
        REGISTRY.configured_features.by_key(&key)
    }

    /// Vanilla `TreeGrower.getMinimumHeight`: base trunk height of the
    /// shortest configured tree, or zero when no tree feature is registered.
    pub(super) fn minimum_tree_height(name: &str) -> i32 {
        let Some(feature) = Self::configured_feature(name) else {
            return 0;
        };
        let ConfiguredFeatureKind::Tree(config) = &feature.kind else {
            return 0;
        };
        match &config.trunk_placer {
            TrunkPlacer::Straight(base)
            | TrunkPlacer::Giant(base)
            | TrunkPlacer::Fancy(base)
            | TrunkPlacer::Forking(base)
            | TrunkPlacer::DarkOak(base)
            | TrunkPlacer::MegaJungle(base) => base.base_height,
            TrunkPlacer::Bending(base) => base.base_height,
            TrunkPlacer::UpwardsBranching(base) => base.base_height,
            TrunkPlacer::Cherry(base) => base.base_height,
            TrunkPlacer::Poplar(base) => base.base_height,
        }
    }

    /// Vanilla `SaplingBlock.advanceTree`: place the tree feature, replacing the
    /// sapling; restore it when the feature could not find room.
    pub(super) fn grow_tree(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
        rng: &mut dyn Rng,
    ) -> bool {
        // Vanilla attempts a configured mega feature first. A failed mega
        // placement restores all four original states and never tries a small tree.
        if let Some(name) = self.mega_feature_for(rng)
            && Self::configured_feature(name).is_some()
            && let Some(origin) = Self::two_by_two(world.as_ref(), pos, state)
        {
            let saplings = [(0, 0), (1, 0), (0, 1), (1, 1)].map(|(x, z)| {
                let p = origin.offset(x, 0, z);
                (p, world.get_block_state(p))
            });
            return Self::place_tree(world, origin, name, &saplings, rng);
        }
        let Some(name) = self.feature_for(rng, Self::has_flowers(world.as_ref(), pos)) else {
            return false;
        };
        Self::place_tree(world, pos, name, &[(pos, state)], rng)
    }

    fn place_tree(
        world: &Arc<World>,
        origin: BlockPos,
        name: &str,
        saplings: &[(BlockPos, BlockStateId)],
        rng: &mut dyn Rng,
    ) -> bool {
        let Some(feature) = Self::configured_feature(name) else {
            return false;
        };
        let ConfiguredFeatureKind::Tree(config) = &feature.kind else {
            return false;
        };
        let remove_flags = UpdateFlags::UPDATE_CLIENTS
            | UpdateFlags::UPDATE_KNOWN_SHAPE
            | UpdateFlags::UPDATE_SUPPRESS_DROPS
            | UpdateFlags::UPDATE_SKIP_BLOCK_ENTITY_SIDEEFFECTS
            | UpdateFlags::UPDATE_SKIP_ON_PLACE;
        for &(pos, original) in saplings {
            let replacement = crate::fluid::fluid_state_to_block(original.get_fluid_state());
            world.set_block(pos, replacement, remove_flags);
        }
        let mut worldgen_random = WorldgenRandom::from_seed(rng.random());
        let mut level = Arc::clone(world);
        let placed = FeatureDecorationRunner::place_tree_feature(
            &mut level,
            &REGISTRY,
            &mut worldgen_random,
            config,
            origin,
            obfuscate_biome_seed(world.seed()),
            no_nested_features,
        );
        if !placed {
            for &(pos, original) in saplings {
                world.set_block(
                    pos,
                    original,
                    UpdateFlags::UPDATE_INVISIBLE
                        | UpdateFlags::UPDATE_SKIP_BLOCK_ENTITY_SIDEEFFECTS,
                );
            }
        }
        placed
    }
}

impl BlockBehavior for SaplingBlock {
    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        _direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        super::vegetation_block::survival_update_shape(self, state, world, pos)
    }

    fn can_survive(&self, _state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        survives_on_tag(world, pos, &BlockTag::SUPPORTS_VEGETATION)
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        default_surviving_state(self.block, self, context)
    }

    fn random_tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        if rand::random_range(0..RANDOM_GROWTH_BOUND) != 0 {
            return;
        }
        // Vanilla only advances light-blocking (fully lit) saplings.
        if world.max_local_raw_brightness(pos.above(), world.sky_darkening()) < 9 {
            return;
        }
        let mut rng = rand::rng();
        self.advance_tree(world, pos, state, &mut rng);
    }

    fn as_bonemealable(&self) -> Option<&dyn Bonemealable> {
        Some(self)
    }
}

impl Bonemealable for SaplingBlock {
    fn is_valid_bonemeal_target(
        &self,
        state: BlockStateId,
        world: &dyn LevelReader,
        pos: BlockPos,
    ) -> bool {
        if matches!(self.tree_grower_name, "dark_oak" | "pale_oak")
            && Self::two_by_two(world, pos, state).is_none()
        {
            return false;
        }
        let height = self.shortest_tree_type.map_or(0, Self::minimum_tree_height);
        !world.is_outside_build_height(pos.y() + height)
    }

    fn is_bonemeal_success(
        &self,
        _state: BlockStateId,
        _world: &Arc<World>,
        rng: &mut dyn Rng,
        _pos: BlockPos,
    ) -> bool {
        f64::from(rng.random::<f32>()) < 0.45
    }

    fn perform_bonemeal(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        rng: &mut dyn Rng,
        pos: BlockPos,
    ) {
        self.advance_tree(world, pos, state, rng);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{fresh_test_world, insert_ready_full_chunk};
    use rand::{SeedableRng as _, rngs::StdRng};
    use steel_utils::ChunkPos;

    #[test]
    fn bonemeal_advances_stage_and_scheduled_tick_does_not() {
        crate::bootstrap::init_globals();
        let fixture = fresh_test_world("sapling_stage_parity");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let pos = BlockPos::new(8, 64, 8);
        let state = vanilla_blocks::OAK_SAPLING.default_state();
        let sapling = SaplingBlock::new(&vanilla_blocks::OAK_SAPLING, "oak", Some("oak"));
        world.set_block(
            pos.below(),
            vanilla_blocks::DIRT.default_state(),
            UpdateFlags::UPDATE_NONE,
        );
        world.set_block(pos, state, UpdateFlags::UPDATE_NONE);
        sapling.tick(state, world, pos);
        assert_eq!(world.get_block_state(pos).get_value(STAGE), 0);
        sapling.perform_bonemeal(state, world, &mut StdRng::seed_from_u64(263), pos);
        assert_eq!(
            world.get_block_state(pos).get_block(),
            &vanilla_blocks::OAK_SAPLING
        );
        assert_eq!(world.get_block_state(pos).get_value(STAGE), 1);
        assert!(!sapling.is_valid_bonemeal_target(
            state,
            world.as_ref(),
            BlockPos::new(8, world.min_y() + world.height(), 8)
        ));
    }

    #[test]
    fn primary_tree_lists_and_flower_variants_match_vanilla() {
        crate::bootstrap::init_globals();
        let mut rng = StdRng::seed_from_u64(263);
        for (grower, expected) in [
            ("jungle", "jungle_tree_no_vine"),
            ("azalea", "azalea_tree"),
            ("birch", "birch_bees_005"),
            ("cherry", "cherry_bees_005"),
        ] {
            let tree = SaplingBlock::new(&vanilla_blocks::OAK_SAPLING, grower, None);
            assert_eq!(tree.feature_for(&mut rng, true), Some(expected));
        }
        for grower in ["dark_oak", "pale_oak"] {
            let tree = SaplingBlock::new(&vanilla_blocks::OAK_SAPLING, grower, None);
            assert_eq!(tree.feature_for(&mut rng, false), None);
            assert!(tree.mega_feature_for(&mut rng).is_some());
        }
        let tree = SaplingBlock::new(&vanilla_blocks::OAK_SAPLING, "oak", Some("oak"));
        let mut normal = 0;
        let mut fancy = 0;
        for _ in 0..1000 {
            match tree.feature_for(&mut rng, true) {
                Some("oak_bees_005") => normal += 1,
                Some("fancy_oak_bees_005") => fancy += 1,
                other => panic!("unexpected oak feature: {other:?}"),
            }
        }
        assert!(normal > fancy && fancy > 0);
    }

    #[test]
    fn mega_only_target_requires_square_and_failed_growth_restores_each_stage() {
        crate::bootstrap::init_globals();
        let fixture = fresh_test_world("mega_sapling_restore");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let origin = BlockPos::new(8, 64, 8);
        let state = vanilla_blocks::DARK_OAK_SAPLING.default_state();
        let tree = SaplingBlock::new(&vanilla_blocks::DARK_OAK_SAPLING, "dark_oak", None);
        world.set_block(origin, state, UpdateFlags::UPDATE_NONE);
        assert!(!tree.is_valid_bonemeal_target(state, world.as_ref(), origin));
        let positions = [(0, 0), (1, 0), (0, 1), (1, 1)].map(|(x, z)| origin.offset(x, 0, z));
        for (index, pos) in positions.iter().enumerate() {
            world.set_block(
                pos.below(),
                vanilla_blocks::DIRT.default_state(),
                UpdateFlags::UPDATE_NONE,
            );
            world.set_block(
                *pos,
                state.set_value(STAGE, if index == 0 { 0 } else { 1 }),
                UpdateFlags::UPDATE_NONE,
            );
            world.set_block(
                pos.above(),
                vanilla_blocks::STONE.default_state(),
                UpdateFlags::UPDATE_NONE,
            );
        }
        for pos in positions {
            assert_eq!(
                SaplingBlock::two_by_two(world.as_ref(), pos, state),
                Some(origin)
            );
            assert!(tree.is_valid_bonemeal_target(state, world.as_ref(), pos));
        }
        let originals = positions.map(|pos| world.get_block_state(pos));
        assert!(!tree.grow_tree(world, origin, state, &mut StdRng::seed_from_u64(263)));
        for (pos, original) in positions.into_iter().zip(originals) {
            assert_eq!(world.get_block_state(pos), original);
        }
    }

    #[test]
    fn sapling_loses_support_on_neighbor_update() {
        use crate::test_support::TestLevel;
        crate::bootstrap::init_globals();
        let tree = SaplingBlock::new(&vanilla_blocks::OAK_SAPLING, "oak", Some("oak"));
        let state = vanilla_blocks::OAK_SAPLING.default_state();
        for direction in [Direction::Down, Direction::Up, Direction::North] {
            assert!(
                tree.update_shape(
                    state,
                    &TestLevel::default(),
                    BlockPos::ZERO,
                    direction,
                    BlockPos::ZERO.below(),
                    vanilla_blocks::AIR.default_state()
                )
                .is_air()
            );
        }
    }
}
