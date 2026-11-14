use steel_macros::block_behavior;
use steel_math::{DEGREE_180, convert_to_rotation_segment};
use steel_registry::REGISTRY;
use steel_registry::blocks::properties::{
    BlockStateProperties, Direction, EnumProperty, IntProperty,
};
use steel_registry::blocks::{BlockRef, block_state_ext::BlockStateExt};
use steel_registry::vanilla_blocks;
use std::sync::{Arc, Weak};
use steel_utils::{BlockPos, BlockStateId};

use crate::behavior::block::BlockEntityCreation;
use crate::behavior::{BlockBehavior, BlockPlaceContext};
use crate::block_entity::entities::BannerBlockEntity;
use crate::world::World;
use crate::world::{LevelReader, ScheduledTickAccess};

const FACING: &EnumProperty<Direction> = &BlockStateProperties::HORIZONTAL_FACING;

/// Shared behavior for standing banner blocks
#[block_behavior]
pub struct BannerBlock {
    block: BlockRef,
}

const ROTATION_16: &IntProperty = &BlockStateProperties::ROTATION_16;

impl BannerBlock {
    /// Creates a new banner block behavior
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for BannerBlock {
    fn is_possible_to_respawn_in_this(&self, _state: BlockStateId) -> bool {
        true
    }

    fn can_survive(&self, _state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        world.get_block_state(pos.below()).is_solid()
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        if direction == Direction::Down && !self.can_survive(state, world, pos) {
            return REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR);
        }
        state
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state().set_value(
            ROTATION_16,
            convert_to_rotation_segment(context.rotation() + DEGREE_180),
        ))
    /// Vanilla `BannerBlock.newBlockEntity`: every banner owns a block entity to
    /// keep its pattern layers, so the block must create one or the patterns are
    /// lost as soon as the chunk is saved.
    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::Created(Arc::new(BannerBlockEntity::new(level, pos, state)))
    }

    }
}

/// Shared behavior for wall banner blocks
#[block_behavior]
pub struct WallBannerBlock {
    block: BlockRef,
}

impl WallBannerBlock {
    /// Creates a new wall banner block behavior
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for WallBannerBlock {
    fn is_possible_to_respawn_in_this(&self, _state: BlockStateId) -> bool {
        true
    }

    /// Vanilla `BannerBlock.newBlockEntity`: every banner owns a block entity to
    /// keep its pattern layers, so the block must create one or the patterns are
    /// lost as soon as the chunk is saved.
    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::Created(Arc::new(BannerBlockEntity::new(level, pos, state)))
    }

    fn can_survive(&self, state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        let facing = state.get_value(FACING);
        world
            .get_block_state(facing.opposite().relative(pos))
            .is_solid()
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        let facing = state.get_value(FACING);
        if direction == facing.opposite() && !self.can_survive(state, world, pos) {
            return REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR);
        }
        state
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        for direction in context.get_nearest_looking_directions() {
            if !direction.is_horizontal() {
                continue;
            }

            let state = self
                .block
                .default_state()
                .set_value(FACING, direction.opposite());
            if self.can_survive(state, context.world.as_ref(), context.place_pos()) {
                return Some(state);
            }
        }

        None
    }
}
