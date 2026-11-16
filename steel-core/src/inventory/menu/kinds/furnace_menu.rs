//! Furnace, blast-furnace, and smoker menus.

use crate::player::player_inventory::Prediction;
use std::array::from_fn;

use steel_registry::recipe::SingleItemRecipeInput;
use steel_registry::{REGISTRY, menu_type::MenuTypeRef, vanilla_menu_types};
use steel_utils::{DowncastType, DowncastTypeKey, locks::Shared};

use crate::block_entity::entities::{FurnaceContainer, FurnaceKind};
use crate::inventory::fuel_values::VANILLA_FUEL_VALUES;
use crate::inventory::menu::builder::SectionKind;
use crate::inventory::prelude::*;
use crate::inventory::slots::{FurnaceFuelSlot, FurnaceResultSlot};
use crate::player::player_inventory::PlayerInventory;

/// Builds the three-slot Vanilla menu for one furnace-family block entity.
#[must_use]
pub fn furnace_menu(
    inventory: Shared<PlayerInventory>,
    container_id: u8,
    container: ContainerRef,
    kind: FurnaceKind,
) -> Menu {
    let mut builder = MenuBuilder::new(menu_type(kind), container_id);

    let input = builder.section_at(container.clone(), [0], SectionKind::Normal);
    let fuel = builder.section_at(
        container.clone(),
        [1],
        SectionKind::custom(|container, index| {
            Box::new(FurnaceFuelSlot::new(container.clone(), index))
        }),
    );
    let result = builder.section_at(
        container.clone(),
        [2],
        SectionKind::custom(|container, index| {
            Box::new(FurnaceResultSlot::new(container.clone(), index))
        }),
    );
    let player = builder.player_inventory(&inventory);
    let data = from_fn(|_| builder.data_slot(0));

    builder.build(FurnaceMenuKind {
        container,
        kind,
        input,
        fuel,
        result,
        player,
        data,
    })
}

#[must_use]
const fn menu_type(kind: FurnaceKind) -> MenuTypeRef {
    match kind {
        FurnaceKind::Furnace => &vanilla_menu_types::FURNACE,
        FurnaceKind::BlastFurnace => &vanilla_menu_types::BLAST_FURNACE,
        FurnaceKind::Smoker => &vanilla_menu_types::SMOKER,
    }
}

/// State and shift-click behavior shared by all furnace-family menus.
pub struct FurnaceMenuKind {
    container: ContainerRef,
    kind: FurnaceKind,
    input: Section,
    fuel: Section,
    result: Section,
    player: PlayerInventorySections,
    data: [DataSlot; 4],
}

// SAFETY: This Steel-owned key uniquely identifies furnace-family menu behavior.
unsafe impl DowncastType for FurnaceMenuKind {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:menu/abstract_furnace");
}

impl FurnaceMenuKind {
    fn update_data(&self, behavior: &mut MenuBehavior, guard: &ContainerLockGuard) {
        let Some(container) = guard.get_typed::<FurnaceContainer>(self.container.container_id())
        else {
            return;
        };
        for (slot, value) in self.data.into_iter().zip(container.data()) {
            slot.set(behavior, value);
        }
    }

    fn can_smelt(&self, stack: &ItemStack) -> bool {
        REGISTRY
            .recipes
            .find_match(
                self.kind.recipe_type(),
                &SingleItemRecipeInput::new(stack.clone()),
            )
            .is_some()
    }
}

impl MenuKind for FurnaceMenuKind {
    fn still_valid(&self, _behavior: &MenuBehavior, player: &Player) -> bool {
        self.container.still_valid(player)
    }

    /// Vanilla `AbstractFurnaceMenu.fillCraftSlotsStackedContents`: only the
    /// input slot (0) is part of the placement snapshot.
    fn fill_craft_slots_stacked_contents(
        &self,
        _behavior: &MenuBehavior,
        guard: &ContainerLockGuard,
        contents: &mut crate::player::recipe_placement::StackedItemContents,
    ) -> bool {
        let Some(container) = guard.get(self.container.container_id()) else {
            return false;
        };
        contents.account_stack(container.get_item(0));
        true
    }

    /// Vanilla `AbstractFurnaceMenu.handlePlacement`: the grid is the single
    /// input slot, and clearing the grid also empties the fuel slot (2).
    fn handle_placement(
        &mut self,
        _behavior: &mut MenuBehavior,
        guard: &mut ContainerLockGuard,
        player: &Player,
        request: crate::player::recipe_placement::PlacementRequest<'_>,
    ) -> Option<crate::player::recipe_placement::PostPlaceAction> {
        let Some(container) = guard.get(self.container.container_id()) else {
            return None;
        };
        let grid = vec![container.get_item(0).clone()];
        // Vanilla `AbstractFurnaceMenu.handlePlacement` clears the input *and*
        // the fuel slot, so a placed recipe never sits on a running burn.
        let slots_to_clear = [container.get_item(0).clone(), container.get_item(2).clone()];
        let inventory: Vec<ItemStack> = player.inventory.lock().get_items().to_vec();
        let matches = crate::player::recipe_placement::single_item_matches(
            container.get_item(0),
            request.placement,
        );
        let plan = crate::player::recipe_placement::plan_placement(
            &inventory,
            1,
            1,
            &grid,
            request.placement,
            request.recipe_size,
            matches,
            request.use_max_items,
            request.allow_dropping_items_to_clear,
            // Vanilla `inputGridSlots` is `{slot 0}`; the *cleared* set is
            // `{slot 0, slot 2}` and `clearGrid` returns both to the player. The
            // extra slot is handed to `apply_plan` through `inventory_give`, so
            // the plan carries it and nothing is dropped.
            &slots_to_clear[1..],
            |contents| {
                // Vanilla `AbstractFurnaceBlockEntity.fillStackedContents`
                // accounts every slot, fuel included.
                for stack in &grid {
                    contents.account_stack(stack);
                }
            },
        );

        for stack in plan.inventory_give {
            player.add_item_or_drop_with_guard(
                guard,
                stack,
                crate::player::player_inventory::Prediction::Predicted,
            );
        }
        crate::player::recipe_placement::apply_inventory_take(guard, player, &plan.inventory_take);
        if let Some(container) = guard.get_mut(self.container.container_id()) {
            container.set_item(0, plan.grid[0].clone());
            if plan.clear_applied {
                container.set_item(2, ItemStack::empty());
            }
            container.set_changed();
        }
        Some(plan.action)
    }

    fn on_open(
        &mut self,
        behavior: &mut MenuBehavior,
        guard: &mut ContainerLockGuard,
        _player: &Player,
    ) {
        self.update_data(behavior, guard);
    }

    fn on_tick(
        &mut self,
        behavior: &mut MenuBehavior,
        guard: &mut ContainerLockGuard,
        _player: &Player,
    ) {
        self.update_data(behavior, guard);
    }

    fn quick_move(
        &mut self,
        behavior: &mut MenuBehavior,
        guard: &mut ContainerLockGuard,
        slot_index: usize,
        player: &Player,
    ) -> Option<ItemStack> {
        if slot_index >= behavior.slots().len() {
            return Some(ItemStack::empty());
        }
        let clicked = behavior.slots()[slot_index].get_item(guard).clone();
        if clicked.is_empty() {
            return Some(ItemStack::empty());
        }
        let mut remaining = clicked.clone();

        let moved = if self.result.contains(slot_index) {
            behavior.move_item_stack_to(
                guard,
                slot_index,
                &mut remaining,
                self.player.all().start(),
                self.player.all().end(),
                FillDirection::Backward,
            )
        } else if self.input.contains(slot_index) || self.fuel.contains(slot_index) {
            behavior.move_item_stack_to(
                guard,
                slot_index,
                &mut remaining,
                self.player.all().start(),
                self.player.all().end(),
                FillDirection::Forward,
            )
        } else if self.can_smelt(&clicked) {
            behavior.move_item_stack_to(
                guard,
                slot_index,
                &mut remaining,
                self.input.start(),
                self.input.end(),
                FillDirection::Forward,
            )
        } else if VANILLA_FUEL_VALUES.is_fuel(clicked.item()) {
            behavior.move_item_stack_to(
                guard,
                slot_index,
                &mut remaining,
                self.fuel.start(),
                self.fuel.end(),
                FillDirection::Forward,
            )
        } else if self.player.main().contains(slot_index) {
            behavior.move_item_stack_to(
                guard,
                slot_index,
                &mut remaining,
                self.player.hotbar().start(),
                self.player.hotbar().end(),
                FillDirection::Forward,
            )
        } else if self.player.hotbar().contains(slot_index) {
            behavior.move_item_stack_to(
                guard,
                slot_index,
                &mut remaining,
                self.player.main().start(),
                self.player.main().end(),
                FillDirection::Forward,
            )
        } else {
            false
        };

        if !moved {
            return Some(ItemStack::empty());
        }
        behavior.update_quick_move_source(guard, slot_index, &remaining, &clicked);
        if remaining.count() == clicked.count() {
            return Some(ItemStack::empty());
        }
        // Vanilla `AbstractContainerMenu.doClick` reports the moved amount to the
        // slot, which is how the furnace result counts a shift-clicked craft.
        let taken = clicked.copy_with_count(clicked.count - remaining.count);
        behavior.slots()[slot_index].on_quick_craft(&taken, player);
        if let Some(remainder) = behavior.slots()[slot_index].on_take(guard, &remaining, player) {
            player.add_item_or_drop_with_guard(guard, remainder, Prediction::Predicted);
        }
        Some(clicked)
    }

    fn can_take_item_for_pick_all(&self, _carried: &ItemStack, slot_index: usize) -> bool {
        !self.result.contains(slot_index)
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_items};
    use steel_utils::locks::IntoShared as _;

    use super::*;
    use crate::inventory::container::SimpleContainer;

    #[test]
    fn manual_input_accepts_items_without_a_matching_recipe() {
        init_vanilla_registry();

        for (kind, item) in [
            (FurnaceKind::Furnace, &*vanilla_items::FURNACE),
            (FurnaceKind::Smoker, &*vanilla_items::RAW_IRON),
        ] {
            let stack = ItemStack::new(item);
            assert!(
                REGISTRY
                    .recipes
                    .find_match(
                        kind.recipe_type(),
                        &SingleItemRecipeInput::new(stack.clone()),
                    )
                    .is_none()
            );

            let menu = furnace_menu(
                PlayerInventory::new().into_shared(),
                1,
                ContainerRef::from(SimpleContainer::new(3).into_shared()),
                kind,
            );

            assert!(menu.behavior().slots()[0].may_place(&stack));
        }
    }
}
