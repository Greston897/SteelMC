use std::sync::Arc;

use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::blocks::properties::{
    BlockStateProperties, BoolProperty, Direction, EnumProperty,
};
use steel_registry::level_events;
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId};

use crate::behavior::block::schedule_water_tick_if_waterlogged;
use crate::behavior::blocks::redstone::{MAX_REDSTONE_SIGNAL, MIN_REDSTONE_SIGNAL};
use crate::behavior::{BlockBehavior, BlockPlaceContext};
use crate::entity::ai::path::PathComputationType;
use crate::world::{LevelReader, ScheduledTickAccess, SignalQueryContext, World};

/// Lightning rod behavior.
#[block_behavior]
pub struct LightningRodBlock {
    block: BlockRef,
}

const WATERLOGGED: &BoolProperty = &BlockStateProperties::WATERLOGGED;
const FACING: &EnumProperty<Direction> = &BlockStateProperties::FACING;
const POWERED: &BoolProperty = &BlockStateProperties::POWERED;

const ACTIVATION_TICKS: i32 = 8;

impl LightningRodBlock {
    /// Creates a lightning rod behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
    /// Returns the block this behavior is attached to.
    #[must_use]
    pub const fn block_ref(&self) -> BlockRef {
        self.block
    }

    fn update_neighbors(block: BlockRef, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        let front = state.get_value(FACING).opposite();
        // Experimental redstone orientations are intentionally omitted.
        world.update_neighbors_at(pos.relative(front), block);
    }
    /// Vanilla `LightningRodBlock.onLightningStrike`: powers the rod and
    /// schedules its deactivation.
    ///
    /// Named apart from the `BlockBehavior::on_lightning_strike` hook so the
    /// call from `WeatheringLightningRodBlock` cannot resolve to the trait
    /// method, which takes `&self`.
    pub(crate) fn strike(block: BlockRef, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        world.set_block(pos, state.set_value(POWERED, true), UpdateFlags::UPDATE_ALL);
        Self::update_neighbors(block, state, world, pos);
        world.schedule_block_tick_default(pos, block, ACTIVATION_TICKS);
        world.level_event(
            level_events::PARTICLES_ELECTRIC_SPARK,
            pos,
            state.get_value(FACING).get_axis().ordinal(),
            None,
        );
    }
}

impl BlockBehavior for LightningRodBlock {
    fn on_lightning_strike(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        Self::strike(self.block, state, world, pos);
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(
            self.block
                .default_state()
                .set_value(FACING, context.clicked_face())
                .set_value(WATERLOGGED, context.is_water_source()),
        )
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
        schedule_water_tick_if_waterlogged(state, world, pos);

        state
    }

    fn get_own_signal(
        &self,
        state: BlockStateId,
        _world: &dyn LevelReader,
        _pos: BlockPos,
        _context: SignalQueryContext,
    ) -> i32 {
        if state.get_value(POWERED) {
            return MAX_REDSTONE_SIGNAL;
        }
        MIN_REDSTONE_SIGNAL
    }

    fn get_direct_signal(
        &self,
        state: BlockStateId,
        _world: &dyn LevelReader,
        _pos: BlockPos,
        direction: Direction,
        _context: SignalQueryContext,
    ) -> i32 {
        if state.get_value(POWERED) && state.get_value(FACING) == direction {
            return MAX_REDSTONE_SIGNAL;
        }
        MIN_REDSTONE_SIGNAL
    }

    fn tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        world.set_block(
            pos,
            state.set_value(POWERED, false),
            UpdateFlags::UPDATE_ALL,
        );
        Self::update_neighbors(self.block, state, world, pos);
    }

    fn affect_neighbors_after_removal(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _moved_by_piston: bool,
    ) {
        if state.get_value(POWERED) {
            Self::update_neighbors(self.block, state, world, pos);
        }
    }

    fn on_place(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        old_state: BlockStateId,
        _moved_by_piston: bool,
    ) {
        if state.get_block() != old_state.get_block()
            && state.get_value(POWERED)
            && !world.has_scheduled_block_tick(pos, self.block)
        {
            world.schedule_block_tick_default(pos, self.block, ACTIVATION_TICKS);
        }
    }

    fn is_signal_source(&self, _state: BlockStateId, _context: SignalQueryContext) -> bool {
        true
    }

    fn is_pathfindable(
        &self,
        _state: BlockStateId,
        _computation_type: PathComputationType,
    ) -> bool {
        false
    }

    // `animateTick` emits client-local particles only.
}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_blocks};
    use steel_utils::ChunkPos;

    use super::*;
    use crate::behavior::init_behaviors;
    use crate::test_support::{fresh_test_world, insert_ready_full_chunk};

    #[test]
    fn waxed_rod_strike_powers_and_then_unpowers_the_rod() {
        init_vanilla_registry();
        init_behaviors();
        let fixture = fresh_test_world("waxed_rod_strike");
        let world = &fixture.world;
        let pos = BlockPos::new(8, 64, 8);
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let state = vanilla_blocks::WAXED_LIGHTNING_ROD.default_state();
        assert!(world.set_block(pos, state, UpdateFlags::UPDATE_ALL));
        let behavior = LightningRodBlock::new(&vanilla_blocks::WAXED_LIGHTNING_ROD);
        behavior.on_lightning_strike(state, world, pos);
        let powered = world.get_block_state(pos);
        assert!(powered.get_value(POWERED));
        assert!(world.has_scheduled_block_tick(pos, &vanilla_blocks::WAXED_LIGHTNING_ROD));
        behavior.tick(powered, world, pos);
        assert!(!world.get_block_state(pos).get_value(POWERED));
    }
}
