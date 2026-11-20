//! Crafting table menu (3x3 grid).
//!
//! Slot layout (46 total):
//! - Slot 0: Result
//! - Slots 1-9: 3x3 grid
//! - Slots 10-36: Main inventory (27)
//! - Slots 37-45: Hotbar (9)

use std::sync::Arc;

use crate::inventory::container::{CraftingContainer, ResultContainer};
use crate::inventory::prelude::*;
use crate::inventory::slots::CraftingHandler;
use crate::player::player_inventory::PlayerInventory;
use crate::player::recipe_placement::{PostPlaceAction, StackedItemContents, plan_placement};
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_blocks;
use steel_registry::vanilla_menu_types;
use steel_utils::BlockPos;
use steel_utils::locks::IntoShared;
use steel_utils::locks::Shared;
use crate::player::recipe_placement::PlacementRequest;
use crate::player::recipe_placement::apply_plan;
use crate::player::recipe_placement::grid_produces;

/// Builds the crafting table menu with a 3x3 grid.
#[must_use]
pub fn crafting(inventory: Shared<PlayerInventory>, container_id: u8, block_pos: BlockPos) -> Menu {
    let crafting_container = CraftingContainer::new(3, 3).into_shared();
    let result_container = ResultContainer::new().into_shared();

    let handler = CraftingHandler::new(
        Arc::clone(&crafting_container),
        Arc::clone(&result_container),
        3,
    );

    let mut builder = MenuBuilder::new(&vanilla_menu_types::CRAFTING, container_id);
    // Vanilla `CraftingMenu` uses `ResultSlot`, which awards `ITEM_CRAFTED`.
    let result = builder.crafting_result_slot(handler.clone());
    let grid = builder.section_all(crafting_container);
    let player = builder.player_inventory(&inventory);

    builder.route(result, player.all(), FillDirection::Backward);
    builder.route(grid, player.all(), FillDirection::Forward);
    builder.route(
        player.main(),
        [grid, player.hotbar()],
        FillDirection::Forward,
    );
    builder.route(
        player.hotbar(),
        [grid, player.main()],
        FillDirection::Forward,
    );
    builder.drain(grid);

    builder.build(CraftingKind {
        result_container,
        result,
        block_pos,
        handler,
    })
}

/// Per-menu crafting state: result container, table position, and recipe handler.
pub struct CraftingKind {
    /// The result container.
    result_container: Shared<ResultContainer>,
    /// The result (slot 0).
    result: Section,
    /// The crafting table block position.
    block_pos: BlockPos,
    handler: CraftingHandler,
}

// SAFETY: This Steel-owned key uniquely identifies the concrete menu kind
// within the process.
unsafe impl steel_utils::DowncastType for CraftingKind {
    const TYPE_KEY: steel_utils::DowncastTypeKey =
        steel_utils::DowncastTypeKey::new("steel:menu/crafting");
}

impl MenuKind for CraftingKind {
    /// Prevents taking from the result slot during pickup-all.
    fn can_take_item_for_pick_all(&self, _carried: &ItemStack, slot_index: usize) -> bool {
        !self.result.contains(slot_index)
    }

    fn still_valid(&self, _behavior: &MenuBehavior, player: &Player) -> bool {
        still_valid_block(
            &player.get_world(),
            self.block_pos,
            &vanilla_blocks::CRAFTING_TABLE,
            player,
        )
    }

    /// Clears the virtual result on close. The grid is drained by [`Menu::removed`].
    fn removed(&mut self, _behavior: &mut MenuBehavior, _player: &Player) {
        self.result_container.lock().set_item(0, ItemStack::empty());
    }

    fn slots_changed(
        &mut self,
        _behavior: &mut MenuBehavior,
        guard: &mut ContainerLockGuard,
        _player: &Player,
    ) {
        self.handler.update_result(guard);
    }

    /// Vanilla `AbstractCraftingMenu.fillCraftSlotsStackedContents`.
    fn fill_craft_slots_stacked_contents(
        &self,
        _behavior: &MenuBehavior,
        guard: &ContainerLockGuard,
        contents: &mut StackedItemContents,
    ) -> bool {
        let Some(crafting) = guard.get_typed::<CraftingContainer>(self.handler.crafting_id())
        else {
            return false;
        };
        for stack in crafting.items() {
            // Vanilla `TransientCraftingContainer.fillStackedContents` calls
            // `accountSimpleStack`, which skips damaged/uncraftable stacks.
            contents.account_simple_stack(stack);
        }
        true
    }

    /// Vanilla `AbstractCraftingMenu.handlePlacement` for a 3x3 grid.
    fn handle_placement(
        &mut self,
        _behavior: &mut MenuBehavior,
        guard: &mut ContainerLockGuard,
        player: &Player,
        request: PlacementRequest<'_>,
    ) -> Option<PostPlaceAction> {
        let crafting = guard.get_typed::<CraftingContainer>(self.handler.crafting_id())?;
        let grid = crafting.items().to_vec();
        let matches = grid_produces(crafting, request.recipe_key);
        let inv_id = ContainerId::from_arc(&player.inventory);
        let inventory: Vec<ItemStack> = guard
            .get_typed::<PlayerInventory>(inv_id)?
            .get_items()
            .to_vec();
        let plan = plan_placement(
            &inventory,
            3,
            3,
            &grid,
            request.placement,
            request.recipe_size,
            matches,
            request.use_max_items,
            request.allow_dropping_items_to_clear,
            &[],
            |contents| {
                // Vanilla `AbstractCraftingMenu.fillCraftSlotsStackedContents`:
                // the grid's own contents count too, so a partially filled grid
                // can be topped up by the recipe book.
                for stack in &grid {
                    contents.account_simple_stack(stack);
                }
            },
        );
        Some(apply_plan(
            guard,
            player,
            plan,
            self.handler.crafting_id(),
            &self.handler,
        ))
    }
}

#[cfg(test)]
mod tests;
