use crate::player::Player;
use crate::player::player_inventory::Prediction;
use std::sync::Arc;

use steel_registry::item_stack::ItemStack;
use steel_utils::Identifier;
use steel_utils::locks::{Shared, SyncMutex};

use crate::inventory::container::{CraftingContainer, ResultContainer};
use crate::{
    inventory::{
        container::Container,
        lock::{ContainerId, ContainerLockGuard, ContainerRef},
        recipe_manager,
        slots::result_handler::ResultHandler,
    },
    player::Player,
};

/// A Recipe Handler for Crafting Recipes
#[derive(Clone)]
pub struct CraftingHandler {
    crafting_container: Shared<CraftingContainer>,
    result_container: Shared<ResultContainer>,
    grid_size: usize,
    /// The recipe currently shown in the result slot, retained so the limited
    /// crafting rule can re-check the same recipe on every grid change.
    last_recipe: SyncMutex<Option<Identifier>>,
}

impl CraftingHandler {
    /// Creates a new Crafting Recipe Handler
    pub const fn new(
        crafting_container: Shared<CraftingContainer>,
        result_container: Shared<ResultContainer>,
        grid_size: usize,
    ) -> Self {
        Self {
            crafting_container,
            result_container,
            grid_size,
            last_recipe: SyncMutex::new(None),
        }
    }

    /// The `ContainerId` of the crafting container
    #[must_use]
    pub fn crafting_id(&self) -> ContainerId {
        ContainerId::from_arc(&self.crafting_container)
    }

    /// A shared handle to the crafting container.
    #[must_use]
    pub fn crafting_container(&self) -> Shared<CraftingContainer> {
        Arc::clone(&self.crafting_container)
    }

    /// The `ContainerId` of the result container
    #[must_use]
    pub fn result_id(&self) -> ContainerId {
        ContainerId::from_arc(&self.result_container)
    }
}

impl ResultHandler for CraftingHandler {
    fn result_container(&self) -> ContainerRef {
        ContainerRef::from(Arc::clone(&self.result_container))
    }

    fn dependencies(&self) -> Vec<ContainerRef> {
        vec![ContainerRef::from(Arc::clone(&self.crafting_container))]
    }

    /// Vanilla `ResultSlot.checkTakeAchievements` recipe-unlock half.
    ///
    /// Every recipe that matches the current grid and that the player does not
    /// know yet is added to the recipe book. Returns how many were unlocked.
    fn unlock_new_recipes(&self, player: &Player) -> usize {
        let grid = self.crafting_container.lock();
        let mut unlocked = 0;
        let mut book = player.recipe_book.lock();
        for recipe in recipe_manager::matching_recipes_for(&grid) {
            if book.add(recipe.key()).is_some() {
                unlocked += 1;
            }
        }
        unlocked
    }

    /// Returns the key of the recipe the current grid produces, if any.
    fn crafted_recipe(&self, guard: &ContainerLockGuard) -> Option<steel_utils::Identifier> {
        let crafting = guard.get_typed::<CraftingContainer>(self.crafting_id())?;
        recipe_manager::find_recipe(crafting).map(|recipe| recipe.key().clone())
    }

    fn update_result(&self, guard: &mut ContainerLockGuard) {
        let crafting = guard
            .get_typed::<CraftingContainer>(self.crafting_id())
            .expect("crafting container not locked");

        let recipe = recipe_manager::find_recipe(crafting);
        // Vanilla `CraftingMenu.slotChangedCraftingGrid` keeps the previous recipe
        // in `lastRecipe` and clears the result when the new one is blocked by the
        // server's limited-crafting rule.
        let recipe_key = recipe.as_ref().map(|recipe| recipe.key().clone());
        *self.last_recipe.lock() = recipe_key;

        let result_stack =
            recipe_manager::assemble_for_container(crafting).unwrap_or_else(ItemStack::empty);

        let result_container = guard
            .get_typed_mut::<ResultContainer>(self.result_id())
            .expect("result container not locked");
        result_container.set_item(0, result_stack);
        result_container.set_changed();
    }

    fn on_result_taken(
        &self,
        guard: &mut ContainerLockGuard,
        player: &Player,
    ) -> Option<ItemStack> {
        // Vanilla `ResultSlot.checkTakeAchievements`: unlock every recipe the player
        // can now craft, then fire `recipe_crafted` for the crafted result.
        let unlocked = self.unlock_new_recipes(player);
        let crafted_recipe = self.crafted_recipe(guard);
        let mut remainder_overflow: Vec<ItemStack> = Vec::new();

        let remainders_and_positioned = {
            let crafting = guard
                .get_typed::<CraftingContainer>(self.crafting_id())
                .expect("crafting container not locked");
            recipe_manager::get_remaining_items(crafting)
        };

        let Some((remainders, positioned)) = remainders_and_positioned else {
            guard
                .get_typed_mut::<ResultContainer>(self.result_id())
                .expect("result container not locked")
                .set_item(0, ItemStack::empty());
            return None;
        };

        {
            let crafting = guard
                .get_typed_mut::<CraftingContainer>(self.crafting_id())
                .expect("crafting container not locked");

            let input = &positioned.input;

            for y in 0..input.height {
                for x in 0..input.width {
                    let grid_slot = positioned.to_grid_slot(x, y, self.grid_size);
                    let remainder_idx = x + y * input.width;
                    let replacement = if remainder_idx < remainders.len() {
                        remainders[remainder_idx].clone()
                    } else {
                        ItemStack::empty()
                    };

                    {
                        let item = crafting.get_item_mut(grid_slot);
                        if !item.is_empty() {
                            item.shrink_one();
                        }
                    }

                    if !replacement.is_empty() {
                        let current_item = crafting.get_item(grid_slot).clone();

                        if current_item.is_empty() {
                            crafting.set_item(grid_slot, replacement);
                        } else if ItemStack::is_same_item_same_components(
                            &current_item,
                            &replacement,
                        ) {
                            crafting.get_item_mut(grid_slot).grow(replacement.count());
                        } else {
                            remainder_overflow.push(replacement);
                        }
                    }
                }
            }

            crafting.set_changed();
        }

        self.update_result(guard);

        // Vanilla fires the triggers after the grid was consumed, so a failed craft
        // never unlocks anything.
        let _ = unlocked;
        if let Some(recipe) = crafted_recipe {
            player.trigger_recipe_crafted(&recipe);
            // Vanilla `CraftingMenu#slotsChanged` fires `CRAFTING_TABLE_USED`
            // when the menu is a full 3x3 crafting table (the 2x2 player grid
            // does not count), so the `minecraft:recipes/...` style advancements
            // that gate on a crafting table behave correctly.
            if self.grid_size == 9 {
                player.trigger_advancement_with_context(
                    &steel_registry::vanilla_criteria_triggers::CRAFTING_TABLE_USED,
                    &crate::player::advancement::predicate::TriggerContext::default(),
                );
            }
        }

        for remainder in remainder_overflow {
            player.add_item_or_drop_with_guard(guard, remainder, Prediction::Predicted);
        }

        None
    }

    fn is_result_valid(&self, guard: &ContainerLockGuard, player: &Player) -> bool {
        let Some(result) = guard.get(self.result_id()) else {
            return false;
        };
        let result_item = result.get_item(0);
        if result_item.is_empty() {
            return false;
        }

        // Vanilla `CraftingMenu.slotChangedCraftingGrid`: with the
        // `limited_crafting` game rule on, a player who has not unlocked the
        // recipe in their recipe book cannot take the result.
        if player
            .get_world()
            .get_game_rule(&steel_utils::LIMITED_CRAFTING)
            && let Some(key) = self.last_recipe.lock().clone()
            && !player.recipe_book.lock().contains(&key)
        {
            return false;
        }

        let Some(crafting) = guard.get_typed::<CraftingContainer>(self.crafting_id()) else {
            return false;
        };

        let Some(assembled) = recipe_manager::assemble_for_container(crafting) else {
            return false;
        };

        ItemStack::matches(result_item, &assembled)
    }
}
