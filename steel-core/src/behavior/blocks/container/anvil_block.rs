use std::sync::Arc;

use steel_macros::block_behavior;
use steel_registry::{
    blocks::{
        BlockRef,
        block_state_ext::BlockStateExt,
        properties::{BlockStateProperties, EnumProperty},
    },
    items::item::BlockHitResult,
    vanilla_blocks, vanilla_custom_stats,
};
use steel_utils::{BlockPos, BlockStateId, Direction, translations};
use text_components::TextComponent;

use crate::{
    behavior::{BlockBehavior, BlockPlaceContext, InteractionResult, InventoryAccess, Fallable},
    inventory::menu::kinds::anvil,
    player::Player,
    world::{World, ScheduledTickAccess},
};

/// Behavior for Anvils
#[block_behavior]
pub struct AnvilBlock {
    block: BlockRef,
}

const FACING: &EnumProperty<Direction> = &BlockStateProperties::FACING;
const HORIZONTAL_FACING: &EnumProperty<Direction> = &BlockStateProperties::HORIZONTAL_FACING;

impl AnvilBlock {
    /// Creates a new Anvil Block Behavior
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }

    /// Returns the next damage level block state for anvil `BlockStates`
    #[must_use]
    pub fn damage(state: BlockStateId) -> Option<BlockStateId> {
        let block = state.get_block();
        if block == &vanilla_blocks::ANVIL {
            Some(
                vanilla_blocks::CHIPPED_ANVIL
                    .default_state()
                    .copy_value(FACING, &state),
            )
        } else if block == &vanilla_blocks::CHIPPED_ANVIL {
            Some(
                vanilla_blocks::DAMAGED_ANVIL
                    .default_state()
                    .copy_value(FACING, &state),
            )
        } else {
            None
        }
    }
}

impl Fallable for AnvilBlock {}

impl BlockBehavior for AnvilBlock {
    fn on_place(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _old_state: BlockStateId,
        _moved_by_piston: bool,
    ) {
        crate::behavior::blocks::FallingBlock::new(self.block).on_place(world, pos);
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        ticks: &dyn ScheduledTickAccess,
        pos: BlockPos,
        _direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        crate::behavior::blocks::FallingBlock::new(self.block).update_shape(state, ticks, pos)
    }

    fn tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        if let Some(entity) = crate::behavior::blocks::FallingBlock::tick(state, world, pos) {
            entity.set_hurts_entities(2.0, 40);
        }
    }

    fn as_fallable(&self) -> Option<&dyn Fallable> {
        Some(self)
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state().set_value(
            HORIZONTAL_FACING,
            context.horizontal_direction().rotate_y_clockwise(),
        ))
    }

    fn allows_spectator_menu(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
    ) -> bool {
        // Vanilla `allowsSpectatorMenu` for container blocks.
        let _ = (state, world, pos);
        true
    }

    fn use_without_item(
        &self,
        _state: BlockStateId,
        _world: &Arc<World>,
        pos: steel_utils::BlockPos,
        player: &Player,
        _hit_result: &BlockHitResult,
        _inv: &mut InventoryAccess,
    ) -> InteractionResult {
        let inventory = Arc::clone(&player.inventory);
        player.open_menu(
            TextComponent::translated(translations::CONTAINER_REPAIR.msg()),
            move |context| anvil(inventory, context.container_id, pos, context.world),
        );
        player.award_custom_stat(&vanilla_custom_stats::INTERACT_WITH_ANVIL);
        InteractionResult::Success
    }
}
