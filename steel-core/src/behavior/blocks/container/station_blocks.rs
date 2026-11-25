//! Behavior for the crafting stations whose input slots live in the menu.
//!
//! Loom, cartography table, smithing table, and stonecutter have no block
//! entity: vanilla opens a menu via `SimpleMenuProvider` backed by the block
//! position and level. Only `LoomBlock` and `StonecutterBlock` extend
//! `HorizontalDirectionalBlock`; `CartographyTableBlock` and
//! `SmithingTableBlock` have a single default block state.

use std::sync::Arc;

use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::blocks::properties::BlockStateProperties;
use steel_registry::vanilla_custom_stats;
use steel_utils::{BlockPos, BlockStateId, translations};
use text_components::TextComponent;

use crate::behavior::InventoryAccess;
use crate::behavior::block::BlockBehavior;
use crate::behavior::context::{BlockHitResult, BlockPlaceContext, InteractionResult};
use crate::inventory::menu::kinds::{cartography_table, loom, smithing, stonecutter};
use crate::player::Player;
use crate::world::World;

/// Vanilla `LoomBlock`.
#[block_behavior]
pub struct LoomBlock {
    block: BlockRef,
}

impl LoomBlock {
    /// Creates a new loom block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for LoomBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state().set_value(
            &BlockStateProperties::HORIZONTAL_FACING,
            context.horizontal_direction().opposite(),
        ))
    }

    fn use_without_item(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &Player,
        _hit_result: &BlockHitResult,
        _inv: &mut InventoryAccess,
    ) -> InteractionResult {
        let inventory = Arc::clone(&player.inventory);
        let menu_world = Arc::clone(world);
        player.open_menu(
            TextComponent::from(translations::CONTAINER_LOOM.msg()),
            move |context| loom(inventory, context.container_id, pos, &menu_world),
        );
        player.award_custom_stat(&vanilla_custom_stats::INTERACT_WITH_LOOM);
        InteractionResult::Success
    }
}

/// Vanilla `CartographyTableBlock`.
#[block_behavior]
pub struct CartographyTableBlock {
    block: BlockRef,
}

impl CartographyTableBlock {
    /// Creates a new cartography table block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for CartographyTableBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    fn use_without_item(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &Player,
        _hit_result: &BlockHitResult,
        _inv: &mut InventoryAccess,
    ) -> InteractionResult {
        let inventory = Arc::clone(&player.inventory);
        let menu_world = Arc::clone(world);
        player.open_menu(
            TextComponent::from(translations::CONTAINER_CARTOGRAPHY_TABLE.msg()),
            move |context| cartography_table(inventory, context.container_id, pos, &menu_world),
        );
        player.award_custom_stat(&vanilla_custom_stats::INTERACT_WITH_CARTOGRAPHY_TABLE);
        InteractionResult::Success
    }
}

/// Vanilla `SmithingTableBlock`.
#[block_behavior]
pub struct SmithingTableBlock {
    block: BlockRef,
}

impl SmithingTableBlock {
    /// Creates a new smithing table block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for SmithingTableBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    fn use_without_item(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &Player,
        _hit_result: &BlockHitResult,
        _inv: &mut InventoryAccess,
    ) -> InteractionResult {
        let inventory = Arc::clone(&player.inventory);
        let menu_world = Arc::clone(world);
        player.open_menu(
            TextComponent::from(translations::CONTAINER_UPGRADE.msg()),
            move |context| smithing(inventory, context.container_id, pos, &menu_world),
        );
        player.award_custom_stat(&vanilla_custom_stats::INTERACT_WITH_SMITHING_TABLE);
        InteractionResult::Success
    }
}

/// Vanilla `StonecutterBlock`.
#[block_behavior]
pub struct StonecutterBlock {
    block: BlockRef,
}

impl StonecutterBlock {
    /// Creates a new stonecutter block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for StonecutterBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state().set_value(
            &BlockStateProperties::HORIZONTAL_FACING,
            context.horizontal_direction().opposite(),
        ))
    }

    fn use_without_item(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &Player,
        _hit_result: &BlockHitResult,
        _inv: &mut InventoryAccess,
    ) -> InteractionResult {
        let inventory = Arc::clone(&player.inventory);
        let menu_world = Arc::clone(world);
        player.open_menu(
            TextComponent::from(translations::CONTAINER_STONECUTTER.msg()),
            move |context| stonecutter(inventory, context.container_id, pos, &menu_world),
        );
        player.award_custom_stat(&vanilla_custom_stats::INTERACT_WITH_STONECUTTER);
        InteractionResult::Success
    }
}
