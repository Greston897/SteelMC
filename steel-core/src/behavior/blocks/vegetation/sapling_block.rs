use std::str::FromStr as _;
use std::sync::Arc;

use rand::{Rng, RngExt};
use steel_macros::block_behavior;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::blocks::properties::{BlockStateProperties, IntProperty};
use steel_registry::feature::ConfiguredFeatureKind;
use steel_registry::vanilla_block_tags::BlockTag;
use steel_registry::{REGISTRY, RegistryExt as _, vanilla_blocks};
use steel_utils::random::worldgen_random::WorldgenRandom;
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId, Identifier};
use steel_worldgen::biomes::obfuscate_biome_seed;

use crate::behavior::block::BlockBehavior;
use crate::behavior::blocks::vegetation::bonemealable::Bonemealable;
use crate::behavior::context::BlockPlaceContext;
use crate::world::{LevelAccessor, LevelReader, World};
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

    /// Vanilla `TreeGrower.getConfiguredFeature`: the secondary tree wins at 10%
    /// when the grower has one, otherwise the primary tree is always used.
    fn feature_for(&self, rng: &mut dyn Rng) -> Option<&'static str> {
        match self.shortest_tree_type {
            // Vanilla picks the secondary tree 10% of the time.
            Some(primary) if primary != self.tree_grower_name => {
                if rng.random::<f32>() < 0.1 {
                    Some(self.tree_grower_name)
                } else {
                    Some(primary)
                }
            }
            // Single-tree growers always use their only tree.
            Some(primary) => Some(primary),
            None => Some(self.tree_grower_name),
        }
    }

    /// Resolves a grower name to its `minecraft:<name>` configured feature.
    fn configured_feature(
        name: &str,
    ) -> Option<&'static steel_registry::feature::ConfiguredFeature> {
        let key = Identifier::from_str(&format!("minecraft:{name}")).ok()?;
        REGISTRY.configured_features.by_key(&key)
    }

    /// Vanilla `SaplingBlock.advanceTree`: place the tree feature, replacing the
    /// sapling; restore it when the feature could not find room.
    fn grow_tree(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
        rng: &mut dyn Rng,
    ) -> bool {
        let Some(name) = self.feature_for(rng) else {
            return false;
        };
        let Some(feature) = Self::configured_feature(name) else {
            return false;
        };
        let ConfiguredFeatureKind::Tree(config) = &feature.kind else {
            return false;
        };

        let replacement = vanilla_blocks::AIR.default_state();
        world.set_block(pos, replacement, UpdateFlags::UPDATE_NONE);

        let mut worldgen_random = WorldgenRandom::from_seed(rng.random());
        let mut level = Arc::clone(world);
        let placed = FeatureDecorationRunner::place_tree_feature(
            &mut level,
            &REGISTRY,
            &mut worldgen_random,
            config,
            pos,
            obfuscate_biome_seed(world.seed()),
            no_nested_features,
        );

        if placed {
            // Vanilla plays the block-change sound through the feature placement.
            world.send_block_updated(pos);
        } else {
            world.set_block(pos, state, UpdateFlags::UPDATE_NONE);
        }
        placed
    }
}

impl BlockBehavior for SaplingBlock {
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
        if world.max_local_raw_brightness(pos, world.sky_darkening()) < 9 {
            return;
        }
        let mut rng = rand::rng();
        self.grow_tree(world, pos, state, &mut rng);
    }

    fn tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        // Vanilla `SaplingBlock.tick` only runs the post-placement STAGE check.
        if state.get_value(STAGE) == 0 {
            world.set_block(pos, state.set_value(STAGE, 1), UpdateFlags::UPDATE_CLIENTS);
        }
    }

    fn as_bonemealable(&self) -> Option<&dyn Bonemealable> {
        Some(self)
    }
}

impl Bonemealable for SaplingBlock {
    fn is_valid_bonemeal_target(
        &self,
        _state: BlockStateId,
        _world: &dyn LevelReader,
        _pos: BlockPos,
    ) -> bool {
        true
    }

    fn perform_bonemeal(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        rng: &mut dyn Rng,
        pos: BlockPos,
    ) {
        self.grow_tree(world, pos, state, rng);
    }
}
