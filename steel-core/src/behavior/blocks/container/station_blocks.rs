//! Behavior for the crafting stations that only open a menu.
//!
//! Loom, cartography table, smithing table and stonecutter have no block entity:
//! vanilla opens a menu whose input slots live in the menu itself. The block
//! class therefore only overrides `useWithoutItem`, and the menu is resolved from
//! the player's inventory exactly like the vanilla `SimpleMenuProvider`.

use std::sync::Arc;

use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_registry::vanilla_custom_stats;
use steel_utils::{BlockPos, BlockStateId};
use text_components::TextComponent;

use crate::behavior::InventoryAccess;
use crate::behavior::block::BlockBehavior;
use crate::behavior::context::{BlockHitResult, BlockPlaceContext, InteractionResult};
use crate::inventory::menu::kinds::{cartography_table, loom, smithing, stonecutter};
use crate::player::Player;
use crate::world::World;

macro_rules! station_block {
    ($name:ident, $menu:path, $title:expr, $stat:expr) => {
        #[block_behavior]
        pub struct $name {
            block: BlockRef,
        }

        impl $name {
            /// Creates a new station block behavior.
            #[must_use]
            pub const fn new(block: BlockRef) -> Self {
                Self { block }
            }
        }

        impl BlockBehavior for $name {
            fn get_state_for_placement(
                &self,
                context: &BlockPlaceContext<'_>,
            ) -> Option<BlockStateId> {
                // Vanilla `HorizontalDirectionalBlock.getStateForPlacement`.
                Some(self.block.default_state().set_value(
                    &steel_registry::blocks::properties::BlockStateProperties::HORIZONTAL_FACING,
                    context.horizontal_direction().opposite(),
                ))
            }

            fn use_without_item(
                &self,
                _state: BlockStateId,
                _world: &Arc<World>,
                _pos: BlockPos,
                player: &Player,
                _hit_result: &BlockHitResult,
                _inv: &mut InventoryAccess,
            ) -> InteractionResult {
                let inventory = Arc::clone(&player.inventory);
                player.open_menu(TextComponent::plain($title), move |context| {
                    $menu(inventory, context.container_id)
                });
                player.award_custom_stat($stat);
                InteractionResult::Success
            }
        }
    };
}

station_block!(
    LoomBlock,
    loom,
    "Loom",
    &vanilla_custom_stats::INTERACT_WITH_LOOM
);
station_block!(
    CartographyTableBlock,
    cartography_table,
    "Cartography Table",
    &vanilla_custom_stats::INTERACT_WITH_CARTOGRAPHY_TABLE
);
station_block!(
    SmithingTableBlock,
    smithing,
    "Smithing Table",
    &vanilla_custom_stats::INTERACT_WITH_SMITHING_TABLE
);
station_block!(
    StonecutterBlock,
    stonecutter,
    "Stonecutter",
    &vanilla_custom_stats::INTERACT_WITH_STONECUTTER
);
