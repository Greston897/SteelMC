use rand::{Rng, RngExt as _};
use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::vanilla_blocks;
use steel_utils::{BlockPos, BlockStateId, Direction, types::UpdateFlags};

use super::snowy_block::{snowy_placement_state, update_snowy_shape};
use super::spreading_grass_block::SpreadingGrassBlock;
use crate::behavior::block::BlockBehavior;
use crate::behavior::blocks::vegetation::bonemealable::{BonemealAction, Bonemealable};
use crate::behavior::context::BlockPlaceContext;
use crate::world::{LevelReader, ScheduledTickAccess, World};
use std::sync::Arc;

/// Behavior for grass blocks.
#[block_behavior]
pub struct GrassBlock {
    block: BlockRef,
}

impl GrassBlock {
    /// Creates a new grass block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for GrassBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(snowy_placement_state(self.block, context))
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        _world: &dyn ScheduledTickAccess,
        _pos: BlockPos,
        direction: Direction,
        _neighbor_pos: BlockPos,
        neighbor_state: BlockStateId,
    ) -> BlockStateId {
        update_snowy_shape(state, direction, neighbor_state)
    }

    fn as_bonemealable(&self) -> Option<&dyn Bonemealable> {
        Some(self)
    }

    fn random_tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        SpreadingGrassBlock::random_tick(
            &vanilla_blocks::GRASS_BLOCK,
            &vanilla_blocks::DIRT,
            state,
            world,
            pos,
        );
    }
}


/// Vanilla `GrassBlock` is a `BonemealableBlock` using `NeighborSpreader`: bonemeal
/// copies this block onto dirt neighbours that pass vanilla's spreadability test.
impl Bonemealable for GrassBlock {
    fn is_valid_bonemeal_target(
        &self,
        _state: BlockStateId,
        world: &dyn LevelReader,
        pos: BlockPos,
    ) -> bool {
        // Vanilla requires a light level of at least 9 and an air block above.
        if world.get_block_state(pos.above()).get_light_dampening() != 0 {
            return false;
        }
        crate::behavior::blocks::vegetation::bonemealable::has_spreadable_neighbor_pos(
            world,
            pos,
            self.block.default_state(),
        )
    }

    fn perform_bonemeal(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        rng: &mut dyn rand::Rng,
        pos: BlockPos,
    ) {
        // Vanilla `NeighborSpreader`: up to eight spread attempts per bonemeal use.
        let target = self.block.default_state();
        for _ in 0..8 {
            let mut attempt = pos;
            for _ in 0..3 {
                attempt = attempt.offset(
                    rng.random_range(-1..=1),
                    rng.random_range(-1..=1),
                    rng.random_range(-1..=1),
                );
                if attempt == pos {
                    continue;
                }
                let state = world.get_block_state(attempt);
                if state.get_block() != &vanilla_blocks::DIRT {
                    continue;
                }
                if world.get_block_state(attempt.above()).get_light_dampening() != 0 {
                    continue;
                }
                world.set_block(attempt, target, UpdateFlags::UPDATE_ALL);
            }
        }
    }

    fn bonemeal_action_type(&self) -> BonemealAction {
        BonemealAction::NeighborSpreader
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::blocks::block_state_ext::BlockStateExt;
    use steel_registry::blocks::properties::BlockStateProperties;
    use steel_registry::{init_vanilla_registry, vanilla_blocks};
    use steel_utils::{BlockPos, Direction};

    use super::*;
    use crate::behavior::init_behaviors;
    use crate::test_support::TestLevel;

    #[test]
    fn grass_block_updates_snowy_state() {
        init_vanilla_registry();
        init_behaviors();

        let level = TestLevel::default();
        let pos = BlockPos::new(0, 64, 0);
        let behavior = GrassBlock::new(&vanilla_blocks::GRASS_BLOCK);

        let non_snowy = vanilla_blocks::GRASS_BLOCK.default_state();
        let snowy = behavior.update_shape(
            non_snowy,
            &level,
            pos,
            Direction::Up,
            pos.above(),
            vanilla_blocks::SNOW.default_state(),
        );
        assert!(snowy.get_value(&BlockStateProperties::SNOWY));

        let cleared = behavior.update_shape(
            snowy,
            &level,
            pos,
            Direction::Up,
            pos.above(),
            vanilla_blocks::AIR.default_state(),
        );
        assert!(!cleared.get_value(&BlockStateProperties::SNOWY));
    }
}
