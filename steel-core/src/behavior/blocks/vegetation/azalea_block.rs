use std::sync::Arc;

use rand::{Rng, RngExt as _};
use steel_macros::block_behavior;
use steel_registry::{
    blocks::{BlockRef, block_state_ext::BlockStateExt},
    vanilla_block_tags::BlockTag,
};
use steel_utils::{BlockPos, BlockStateId, Direction};

use crate::{
    behavior::{
        BlockBehavior, BlockPlaceContext,
        blocks::vegetation::{
            Vegetation,
            bonemealable::Bonemealable,
            default_surviving_state,
            sapling_block::SaplingBlock,
            vegetation_block::{survival_update_shape, vegetation_can_survive},
        },
    },
    world::{LevelReader, ScheduledTickAccess, World},
};

/// Behavior for azalea blocks.
#[block_behavior]
pub struct AzaleaBlock {
    block: BlockRef,
}

impl AzaleaBlock {
    /// Creates a new azalea block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for AzaleaBlock {
    fn as_bonemealable(&self) -> Option<&dyn Bonemealable> {
        Some(self)
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        _direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        survival_update_shape(self, state, world, pos)
    }

    fn can_survive(&self, state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        vegetation_can_survive(self, state, world, pos)
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        default_surviving_state(self.block, self, context)
    }
}

impl Vegetation for AzaleaBlock {
    fn may_place_on(&self, state: BlockStateId, _world: &dyn LevelReader, _pos: BlockPos) -> bool {
        state.get_block().has_tag(&BlockTag::SUPPORTS_AZALEA)
    }
}

impl Bonemealable for AzaleaBlock {
    fn is_valid_bonemeal_target(
        &self,
        _state: BlockStateId,
        world: &dyn LevelReader,
        pos: BlockPos,
    ) -> bool {
        let min_height = SaplingBlock::minimum_tree_height("azalea_tree");
        !world.is_outside_build_height(pos.y() + min_height + 2)
            && world
                .get_block_state(pos.above())
                .get_fluid_state()
                .is_empty()
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
        SaplingBlock::new(self.block, "azalea_tree", None).grow_tree(world, pos, state, rng);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::behavior::BLOCK_BEHAVIORS;
    use crate::bootstrap::init_globals;
    use crate::test_support::{fresh_test_world, insert_ready_full_chunk};
    use steel_registry::vanilla_blocks;
    use steel_utils::{ChunkPos, types::UpdateFlags};

    #[test]
    fn both_azaleas_register_bonemeal_and_reject_fluid_above_or_build_limit() {
        init_globals();
        let fixture = fresh_test_world("azalea_bonemeal_target");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let pos = BlockPos::new(8, 64, 8);
        for block in [&vanilla_blocks::AZALEA, &vanilla_blocks::FLOWERING_AZALEA] {
            let behavior = BLOCK_BEHAVIORS.get_behavior(block);
            let grower = behavior
                .as_bonemealable()
                .expect("azalea must be connected to bonemeal");
            let state = block.default_state();
            world.set_block(
                pos.above(),
                vanilla_blocks::AIR.default_state(),
                UpdateFlags::UPDATE_NONE,
            );
            assert!(grower.is_valid_bonemeal_target(state, world.as_ref(), pos));
            world.set_block(
                pos.above(),
                vanilla_blocks::WATER.default_state(),
                UpdateFlags::UPDATE_NONE,
            );
            assert!(!grower.is_valid_bonemeal_target(state, world.as_ref(), pos));
            assert!(!grower.is_valid_bonemeal_target(
                state,
                world.as_ref(),
                BlockPos::new(8, world.min_y() + world.height() - 2, 8)
            ));
        }
    }

    #[test]
    fn azalea_bonemeal_places_configured_tree_and_rooted_dirt() {
        use rand::{SeedableRng as _, rngs::StdRng};
        init_globals();
        let fixture = fresh_test_world("azalea_bonemeal_tree");
        let world = &fixture.world;
        for x in -1..=1 {
            for z in -1..=1 {
                insert_ready_full_chunk(world, ChunkPos::new(x, z));
            }
        }
        let pos = BlockPos::new(8, 64, 8);
        for x in 0..16 {
            for z in 0..16 {
                world.set_block(
                    BlockPos::new(x, 63, z),
                    vanilla_blocks::GRASS_BLOCK.default_state(),
                    UpdateFlags::UPDATE_NONE,
                );
            }
        }
        let state = vanilla_blocks::FLOWERING_AZALEA.default_state();
        world.set_block(pos, state, UpdateFlags::UPDATE_NONE);
        let behavior = BLOCK_BEHAVIORS.get_behavior(&vanilla_blocks::FLOWERING_AZALEA);
        let grower = behavior
            .as_bonemealable()
            .expect("flowering azalea bonemeal registration");
        let mut rng = StdRng::seed_from_u64(263);
        grower.perform_bonemeal(state, world, &mut rng, pos);
        assert_eq!(
            world.get_block_state(pos).get_block(),
            &vanilla_blocks::OAK_LOG
        );
        assert_eq!(
            world.get_block_state(pos.below()).get_block(),
            &vanilla_blocks::ROOTED_DIRT
        );
    }
}
