//! Vanilla recipe-book placement.
//!
//! The player clicks a recipe in the book; the client sends a `place_recipe`
//! packet and the server alone decides what moves. This module mirrors vanilla
//! `ServerPlaceRecipe`, `PlaceRecipeHelper` and `StackedContents`, including the
//! bipartite maximum matching of `StackedContents.RecipePicker` that picks which
//! alternative of a multi-item ingredient is consumed.
//!
//! Nothing here locks a container on its own: the caller owns the menu, the
//! container guard and the player inventory, and applies the resulting
//! [`PlacementPlan`].

use rustc_hash::FxHashMap;
use steel_registry::RegistryExt as _;
use steel_registry::data_components::vanilla_components::MAX_STACK_SIZE;
use steel_registry::item_stack::ItemStack;
use steel_registry::items::ItemRef;
use steel_registry::recipe::{Ingredient, PlacementInfo};

use crate::inventory::container::Container as _;
use crate::inventory::lock::ContainerLockGuard;

/// Vanilla `ServerPlaceRecipe.ITEM_NOT_FOUND`.
const ITEM_NOT_FOUND: i32 = -1;

/// Vanilla `RecipeBookMenu.PostPlaceAction`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostPlaceAction {
    /// Nothing left to do; the grid holds the recipe.
    Nothing,
    /// The player cannot craft it, so the client shows a ghost recipe.
    PlaceGhostRecipe,
}

/// Counts how many of each item kind the crafting inventory holds.
///
/// Vanilla `StackedContents<Holder<Item>>`: the key is the item type, so a
/// damaged or enchanted stack still counts by its item.
#[derive(Debug, Default)]
pub struct StackedItemContents {
    amounts: FxHashMap<steel_utils::Identifier, i32>,
}

impl StackedItemContents {
    /// Creates an empty snapshot.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Vanilla `StackedItemContents.accountSimpleStack`.
    pub fn account_simple_stack(&mut self, stack: &ItemStack) {
        if is_usable_for_crafting(stack) {
            let max = stack.max_stack_size();
            self.account_stack_with_max(stack, max);
        }
    }

    /// Vanilla `StackedItemContents.accountStack(stack)`.
    pub fn account_stack(&mut self, stack: &ItemStack) {
        let max = stack.max_stack_size();
        self.account_stack_with_max(stack, max);
    }

    /// Vanilla `StackedItemContents.accountStack(stack, maxCount)`.
    pub fn account_stack_with_max(&mut self, stack: &ItemStack, max_count: i32) {
        if stack.is_empty() {
            return;
        }
        let count = max_count.min(stack.count());
        if count > 0 {
            self.add(stack.item(), count);
        }
    }

    fn add(&mut self, item: ItemRef, count: i32) {
        *self.amounts.entry(item.key.clone()).or_insert(0) += count;
    }

    fn has_at_least(&self, item: ItemRef, count: i32) -> bool {
        self.amounts.get(&item.key).copied().unwrap_or(0) >= count
    }

    /// Vanilla `StackedContents.tryPick(ingredients, amount, output)`.
    ///
    /// On success the chosen item for every ingredient is returned in placement
    /// order and the consumed amounts are written back, exactly like vanilla's
    /// `take`/`put` calls.
    pub fn try_pick(&mut self, ingredients: &[Ingredient], amount: i32) -> Option<Vec<ItemRef>> {
        if ingredients.is_empty() || amount <= 0 {
            return None;
        }
        let items = self.unique_available_ingredient_items(ingredients);
        let mut picker = RecipePicker::new(ingredients, items);
        let mut satisfied = 0usize;

        loop {
            if let Some(path) = picker.try_assigning_new_item(self, amount) {
                let assigned_item = picker.items[path[0]];
                if let Some(count) = self.amounts.get_mut(&assigned_item.key) {
                    *count -= amount;
                }
                let satisfied_ingredient = path[path.len() - 1];
                picker.set_satisfied(satisfied_ingredient);
                satisfied += 1;
            } else {
                let valid = satisfied == picker.ingredient_count();
                let mut output = Vec::with_capacity(picker.ingredient_count());
                picker.clear_all_visited();
                picker.clear_satisfied();
                if !valid {
                    return None;
                }
                for (item, _ingredient) in picker.assignments() {
                    self.add(item, amount);
                    output.push(item);
                }
                return Some(output);
            }
        }
    }

    /// Vanilla `StackedContents.tryPickAll`: binary-searches the largest
    /// craftable amount and then fills the output at that amount.
    pub fn try_pick_all(
        &mut self,
        ingredients: &[Ingredient],
        max_size: i32,
    ) -> (i32, Vec<ItemRef>) {
        let mut min = 0;
        let mut max = max_size.min(self.result_upper_bound(ingredients)) + 1;
        loop {
            let mid = i32::midpoint(min, max);
            if self.can_craft(ingredients, mid) {
                if max - min <= 1 {
                    let output = if mid > 0 {
                        self.try_pick(ingredients, mid).unwrap_or_default()
                    } else {
                        Vec::new()
                    };
                    return (mid, output);
                }
                min = mid;
            } else {
                max = mid;
            }
        }
    }

    fn can_craft(&self, ingredients: &[Ingredient], amount: i32) -> bool {
        let mut scratch = Self {
            amounts: self.amounts.clone(),
        };
        scratch.try_pick(ingredients, amount).is_some()
    }

    /// Vanilla `StackedContents.getResultUpperBound`.
    fn result_upper_bound(&self, ingredients: &[Ingredient]) -> i32 {
        let mut min = i32::MAX;
        for ingredient in ingredients {
            let mut max = 0;
            for (key, count) in &self.amounts {
                let Some(item) = steel_registry::REGISTRY.items.by_key(key) else {
                    continue;
                };
                if *count > max && ingredient.test_item(item) {
                    max = *count;
                }
                if max >= min {
                    break;
                }
            }
            min = max;
            if max == 0 {
                break;
            }
        }
        min
    }

    /// Vanilla `StackedContents.getUniqueAvailableIngredientItems`.
    fn unique_available_ingredient_items(&self, ingredients: &[Ingredient]) -> Vec<ItemRef> {
        let mut result = Vec::new();
        for (key, count) in &self.amounts {
            if *count <= 0 {
                continue;
            }
            let Some(item) = steel_registry::REGISTRY.items.by_key(key) else {
                continue;
            };
            if ingredients
                .iter()
                .any(|ingredient| ingredient.test_item(item))
            {
                result.push(item);
            }
        }
        result
    }
}

/// Vanilla `Inventory.isUsableForCrafting`.
#[must_use]
pub fn is_usable_for_crafting(stack: &ItemStack) -> bool {
    !stack.is_damaged()
        && !stack.is_enchanted()
        && !stack.has(steel_registry::data_components::vanilla_components::CUSTOM_NAME)
}

/// Vanilla max stack size of an item type, used by `clampToMaxStackSize`.
fn item_max_stack_size(item: ItemRef) -> i32 {
    item.components.get(MAX_STACK_SIZE).unwrap_or(1)
}

/// Vanilla `StackedContents.RecipePicker`: the bipartite matching over the
/// available items and the recipe's ingredient slots, packed into one bit set
/// with vanilla's offsets.
struct RecipePicker {
    ingredients: Vec<Ingredient>,
    items: Vec<ItemRef>,
    data: Vec<bool>,
}

impl RecipePicker {
    fn new(ingredients: &[Ingredient], items: Vec<ItemRef>) -> Self {
        let mut picker = Self {
            ingredients: ingredients.to_vec(),
            items,
            data: Vec::new(),
        };
        let total = picker.visited_ingredient_count()
            + picker.visited_item_count()
            + picker.satisfied_count()
            + picker.connection_count()
            + picker.residual_count();
        picker.data = vec![false; total];
        picker.set_initial_connections();
        picker
    }

    const fn ingredient_count(&self) -> usize {
        self.ingredients.len()
    }

    const fn item_count(&self) -> usize {
        self.items.len()
    }

    fn set_initial_connections(&mut self) {
        for ingredient in 0..self.ingredient_count() {
            for item in 0..self.item_count() {
                if self.ingredients[ingredient].test_item(self.items[item]) {
                    self.set_connection(item, ingredient);
                }
            }
        }
    }

    /// Vanilla `RecipePicker.tryAssigningNewItem`.
    fn try_assigning_new_item(
        &mut self,
        contents: &StackedItemContents,
        capacity: i32,
    ) -> Option<Vec<usize>> {
        self.clear_all_visited();
        for item in 0..self.item_count() {
            if contents.has_at_least(self.items[item], capacity)
                && let Some(path) = self.find_new_item_assignment_path(item)
            {
                return Some(path);
            }
        }
        None
    }

    /// Vanilla `RecipePicker.findNewItemAssignmentPath`.
    fn find_new_item_assignment_path(&mut self, starting_item: usize) -> Option<Vec<usize>> {
        self.clear_all_visited();
        self.visit_item(starting_item);
        let mut path = vec![starting_item];

        while !path.is_empty() {
            let path_length = path.len();
            if path_length % 2 == 1 {
                let item_to_assign = path[path_length - 1];
                for ingredient in 0..self.ingredient_count() {
                    if !self.has_visited_ingredient(ingredient)
                        && self.has_connection(item_to_assign, ingredient)
                        && !self.is_assigned(item_to_assign, ingredient)
                    {
                        self.visit_ingredient(ingredient);
                        path.push(ingredient);
                        break;
                    }
                }
            } else {
                let last_assigned_ingredient = path[path_length - 1];
                if !self.is_satisfied(last_assigned_ingredient) {
                    return Some(path);
                }
                for item in 0..self.item_count() {
                    if !self.has_visited_item(item)
                        && self.is_assigned(item, last_assigned_ingredient)
                    {
                        self.visit_item(item);
                        path.push(item);
                        break;
                    }
                }
            }

            if path.len() == path_length {
                path.pop();
            }
        }
        None
    }

    /// The current `(item, ingredient)` assignments in vanilla's iteration
    /// order: the first item assigned to each ingredient.
    fn assignments(&self) -> Vec<(ItemRef, usize)> {
        let mut result = Vec::new();
        for ingredient in 0..self.ingredient_count() {
            for item in 0..self.item_count() {
                if self.is_assigned(item, ingredient) {
                    result.push((self.items[item], ingredient));
                    break;
                }
            }
        }
        result
    }

    const fn visited_ingredient_offset(&self) -> usize {
        0
    }

    const fn visited_ingredient_count(&self) -> usize {
        self.ingredient_count()
    }

    const fn visited_item_offset(&self) -> usize {
        self.visited_ingredient_offset() + self.visited_ingredient_count()
    }

    const fn visited_item_count(&self) -> usize {
        self.item_count()
    }

    const fn satisfied_offset(&self) -> usize {
        self.visited_item_offset() + self.visited_item_count()
    }

    const fn satisfied_count(&self) -> usize {
        self.ingredient_count()
    }

    const fn connection_offset(&self) -> usize {
        self.satisfied_offset() + self.satisfied_count()
    }

    const fn connection_count(&self) -> usize {
        self.ingredient_count() * self.item_count()
    }

    fn residual_offset(&self) -> usize {
        self.connection_offset() + self.connection_count()
    }

    const fn residual_count(&self) -> usize {
        self.ingredient_count() * self.item_count()
    }

    fn is_satisfied(&self, ingredient: usize) -> bool {
        self.data[self.satisfied_offset() + ingredient]
    }

    fn set_satisfied(&mut self, ingredient: usize) {
        let index = self.satisfied_offset() + ingredient;
        self.data[index] = true;
    }

    fn clear_satisfied(&mut self) {
        let offset = self.satisfied_offset();
        let count = self.satisfied_count();
        self.data[offset..offset + count].fill(false);
    }

    fn set_connection(&mut self, item: usize, ingredient: usize) {
        let index = self.connection_offset() + item * self.ingredient_count() + ingredient;
        self.data[index] = true;
    }

    fn has_connection(&self, item: usize, ingredient: usize) -> bool {
        self.data[self.connection_offset() + item * self.ingredient_count() + ingredient]
    }

    fn is_assigned(&self, item: usize, ingredient: usize) -> bool {
        self.data[self.residual_offset() + item * self.ingredient_count() + ingredient]
    }

    fn visit_ingredient(&mut self, ingredient: usize) {
        let index = self.visited_ingredient_offset() + ingredient;
        self.data[index] = true;
    }

    fn has_visited_ingredient(&self, ingredient: usize) -> bool {
        self.data[self.visited_ingredient_offset() + ingredient]
    }

    fn visit_item(&mut self, item: usize) {
        let index = self.visited_item_offset() + item;
        self.data[index] = true;
    }

    fn has_visited_item(&self, item: usize) -> bool {
        self.data[self.visited_item_offset() + item]
    }

    fn clear_all_visited(&mut self) {
        let ingredient_offset = self.visited_ingredient_offset();
        let ingredient_count = self.visited_ingredient_count();
        self.data[ingredient_offset..ingredient_offset + ingredient_count].fill(false);
        let item_offset = self.visited_item_offset();
        let item_count = self.visited_item_count();
        self.data[item_offset..item_offset + item_count].fill(false);
    }
}

/// Vanilla `PlaceRecipeHelper.placeRecipe`, over a `grid_width * grid_height`
/// grid.
///
/// `entries` is `PlacementInfo.slots_to_ingredient`: `None` is an empty recipe
/// slot. `output` receives `(ingredient_index, grid_index)`.
pub fn place_recipe_layout(
    grid_width: usize,
    grid_height: usize,
    recipe_width: usize,
    recipe_height: usize,
    entries: &[Option<usize>],
    mut output: impl FnMut(usize, usize),
) {
    let mut iterator = entries.iter();
    let mut grid_index = 0usize;

    for grid_y in 0..grid_height {
        let should_center = (recipe_height as f32) < grid_height as f32 / 2.0;
        let start_pos_center =
            (grid_height as f32 / 2.0 - recipe_height as f32 / 2.0).floor() as usize;
        if should_center && start_pos_center > grid_y {
            grid_index += grid_width;
            continue;
        }

        for grid_x in 0..grid_width {
            let Some(entry) = iterator.next() else {
                return;
            };

            let should_center = (recipe_width as f32) < grid_width as f32 / 2.0;
            let start_pos_center =
                (grid_width as f32 / 2.0 - recipe_width as f32 / 2.0).floor() as usize;
            let mut total_recipe_width = recipe_width;
            let mut add_ingredient = grid_x < recipe_width;
            if should_center {
                total_recipe_width = start_pos_center + recipe_width;
                add_ingredient =
                    start_pos_center <= grid_x && grid_x < start_pos_center + recipe_width;
            }

            if add_ingredient {
                if let Some(ingredient_index) = *entry {
                    output(ingredient_index, grid_index);
                }
            } else if total_recipe_width == grid_x {
                grid_index += grid_width - grid_x;
                break;
            }

            grid_index += 1;
        }
    }
}

/// Everything the caller has to apply after one placement request.
pub struct PlacementPlan {
    /// The action vanilla reports.
    pub action: PostPlaceAction,
    /// Stacks taken out of the inventory, as `(inventory_slot, stack)`.
    pub inventory_take: Vec<(usize, ItemStack)>,
    /// Stacks the cleared grid returns to the inventory.
    pub inventory_give: Vec<ItemStack>,
    /// The new crafting grid, in the same order as the input grid.
    pub grid: Vec<ItemStack>,
    /// Whether vanilla's `clearGrid` ran, i.e. the grid (and every extra slot a
    /// menu asks to clear) is emptied and its content moved back to the player.
    ///
    /// Menus use this to know whether the "unchanged" early-returns fired.
    pub clear_applied: bool,
}

impl PlacementPlan {
    fn unchanged(grid: &[ItemStack]) -> Self {
        Self {
            action: PostPlaceAction::Nothing,
            inventory_take: Vec::new(),
            inventory_give: Vec::new(),
            grid: grid.to_vec(),
            clear_applied: false,
        }
    }
}

/// What [`crate::player::Player::handle_place_recipe`] sends back after a
/// placement.
///
/// Vanilla answers `PLACE_GHOST_RECIPE` with the display of the recipe it could
/// not place, so the display has to survive the menu call.
pub struct PlacementOutcome {
    /// The action the menu reported.
    pub action: PostPlaceAction,
    /// The display of the recipe that was asked for.
    pub display: steel_registry::slot_display::RecipeDisplay,
}

/// The pure half of vanilla `ServerPlaceRecipe.placeRecipe`.
///
/// `inventory` is the player's 36 non-equipment stacks in slot order, `grid` is
/// the menu's input grid, and `fill_menu_contents` lets the menu add its own
/// slots to the availability snapshot
/// (`RecipeBookMenu.fillCraftSlotsStackedContents`).
#[expect(
    clippy::too_many_arguments,
    reason = "mirrors vanilla ServerPlaceRecipe.placeRecipe"
)]
pub fn plan_placement(
    inventory: &[ItemStack],
    grid_width: usize,
    grid_height: usize,
    grid: &[ItemStack],
    placement: &PlacementInfo,
    recipe_size: (usize, usize),
    recipe_matches_placed: bool,
    use_max_items: bool,
    allow_dropping_items_to_clear: bool,
    extra_slots_to_clear: &[ItemStack],
    fill_menu_contents: impl FnOnce(&mut StackedItemContents),
) -> PlacementPlan {
    let mut contents = StackedItemContents::new();
    for stack in inventory {
        contents.account_simple_stack(stack);
    }
    fill_menu_contents(&mut contents);

    // Vanilla `testClearGrid`: without room for the grid contents, nothing moves.
    if !allow_dropping_items_to_clear && !test_clear_grid(inventory, grid) {
        return PlacementPlan::unchanged(grid);
    }

    let (biggest_craftable, _) = contents.try_pick_all(&placement.ingredients, i32::MAX);
    if biggest_craftable <= 0 {
        // `tryPlaceRecipe` clears the grid and asks for a ghost recipe.
        return PlacementPlan {
            action: PostPlaceAction::PlaceGhostRecipe,
            inventory_take: Vec::new(),
            inventory_give: grid_plus_extra(grid, extra_slots_to_clear),
            grid: vec![ItemStack::empty(); grid.len()],
            clear_applied: true,
        };
    }

    if recipe_matches_placed {
        for stack in grid {
            if !stack.is_empty()
                && biggest_craftable.min(stack.max_stack_size()) < stack.count() + 1
            {
                return PlacementPlan::unchanged(grid);
            }
        }
    }

    let amount_to_craft = if use_max_items {
        biggest_craftable
    } else if recipe_matches_placed {
        let mut smallest = i32::MAX;
        for stack in grid {
            if !stack.is_empty() && smallest > stack.count() {
                smallest = stack.count();
            }
        }
        if smallest == i32::MAX {
            smallest
        } else {
            smallest + 1
        }
    } else {
        1
    };

    let mut scratch = StackedItemContents {
        amounts: contents.amounts.clone(),
    };
    let Some(mut items_used) = scratch.try_pick(&placement.ingredients, amount_to_craft) else {
        return PlacementPlan::unchanged(grid);
    };

    // Vanilla `clampToMaxStackSize`: a grid slot cannot hold more than one
    // ingredient's stack limit.
    let mut adjusted = amount_to_craft;
    for item in &items_used {
        adjusted = adjusted.min(item_max_stack_size(item));
    }
    if adjusted != amount_to_craft {
        let mut scratch = StackedItemContents {
            amounts: contents.amounts.clone(),
        };
        let Some(items) = scratch.try_pick(&placement.ingredients, adjusted) else {
            return PlacementPlan::unchanged(grid);
        };
        items_used = items;
    }

    let mut inventory = inventory.to_vec();
    let mut inventory_take: Vec<(usize, ItemStack)> = Vec::new();
    let mut new_grid = vec![ItemStack::empty(); grid.len()];
    let mut failed = false;

    place_recipe_layout(
        grid_width,
        grid_height,
        recipe_size.0,
        recipe_size.1,
        &placement.slots_to_ingredient,
        |ingredient_index, grid_index| {
            if failed {
                return;
            }
            let Some(item) = items_used.get(ingredient_index).copied() else {
                failed = true;
                return;
            };
            let mut remaining = adjusted;
            while remaining > 0 {
                let next = move_item_to_grid(
                    &mut inventory,
                    &mut new_grid,
                    grid_index,
                    item,
                    remaining,
                    &mut inventory_take,
                );
                if next == ITEM_NOT_FOUND {
                    failed = true;
                    return;
                }
                remaining = next;
            }
        },
    );

    if failed {
        return PlacementPlan::unchanged(grid);
    }

    PlacementPlan {
        action: PostPlaceAction::Nothing,
        inventory_take,
        inventory_give: grid_plus_extra(grid, extra_slots_to_clear),
        grid: new_grid,
        clear_applied: true,
    }
}

/// Vanilla `ServerPlaceRecipe.clearGrid` returns `slotsToClear`, which is the
/// input grid plus whatever extra slots a menu adds (the furnace fuel slot).
fn grid_plus_extra(grid: &[ItemStack], extra: &[ItemStack]) -> Vec<ItemStack> {
    grid.iter()
        .chain(extra)
        .filter(|stack| !stack.is_empty())
        .cloned()
        .collect()
}

/// Vanilla `ServerPlaceRecipe.testClearGrid`.
fn test_clear_grid(inventory: &[ItemStack], grid: &[ItemStack]) -> bool {
    let mut free_slots: Vec<ItemStack> = Vec::new();
    let free_slots_in_inventory = inventory.iter().filter(|stack| stack.is_empty()).count();

    for stack in grid {
        if stack.is_empty() {
            continue;
        }
        // Vanilla: a slot with remaining space takes the whole stack; otherwise
        // the stack is first merged into an already-listed free slot, and only
        // then does it claim one of its own.
        if slot_with_remaining_space(inventory, stack).is_some() {
            continue;
        }
        if free_slots.len() > free_slots_in_inventory {
            return false;
        }
        let mut merged = false;
        for listed in &mut free_slots {
            if ItemStack::is_same_item(listed, stack)
                && listed.count() != listed.max_stack_size()
                && listed.count() + stack.count() <= listed.max_stack_size()
            {
                let count = stack.count();
                *listed = listed.copy_with_count(listed.count() + count);
                merged = true;
                break;
            }
        }
        if merged {
            continue;
        }
        if free_slots.len() >= free_slots_in_inventory {
            return false;
        }
        free_slots.push(stack.clone());
    }
    true
}

/// Vanilla `Inventory.getSlotWithRemainingSpace`.
fn slot_with_remaining_space(inventory: &[ItemStack], new_stack: &ItemStack) -> Option<usize> {
    inventory.iter().position(|stack| {
        !stack.is_empty()
            && ItemStack::is_same_item(stack, new_stack)
            && stack.count() + new_stack.count() <= stack.max_stack_size()
    })
}

/// Vanilla `ServerPlaceRecipe.moveItemToGrid`.
fn move_item_to_grid(
    inventory: &mut [ItemStack],
    grid: &mut [ItemStack],
    grid_index: usize,
    item: ItemRef,
    count: i32,
    inventory_take: &mut Vec<(usize, ItemStack)>,
) -> i32 {
    let existing = grid[grid_index].clone();
    let Some(inventory_slot) = inventory.iter().position(|stack| {
        !stack.is_empty()
            && stack.item() == item
            && is_usable_for_crafting(stack)
            && (existing.is_empty() || ItemStack::is_same_item_same_components(&existing, stack))
    }) else {
        return ITEM_NOT_FOUND;
    };

    let inventory_item = &mut inventory[inventory_slot];
    let taken = if count < inventory_item.count() {
        inventory_item.split(count)
    } else {
        std::mem::take(inventory_item)
    };
    let taken_count = taken.count();
    inventory_take.push((inventory_slot, taken.copy_with_count(taken_count)));

    if grid[grid_index].is_empty() {
        grid[grid_index] = taken;
    } else {
        let total = grid[grid_index].count() + taken_count;
        grid[grid_index] = grid[grid_index].copy_with_count(total);
    }

    count - taken_count
}

/// One `place_recipe` request as the menu sees it.
pub struct PlacementRequest<'a> {
    /// The recipe's key, needed for `CraftingMenuAccess.recipeMatches`.
    pub recipe_key: &'a steel_utils::Identifier,
    /// The recipe's ingredient layout.
    pub placement: &'a PlacementInfo,
    /// `(width, height)` of the recipe's own grid.
    pub recipe_size: (usize, usize),
    /// Whether the client asked for the largest craftable amount.
    pub use_max_items: bool,
    /// Vanilla's `allowDroppingItemsToClear` (creative players).
    pub allow_dropping_items_to_clear: bool,
}

/// Vanilla `CraftingMenuAccess.recipeMatches` for a crafting grid: the grid
/// currently produces `recipe_key`.
#[must_use]
pub fn grid_produces(
    crafting: &crate::inventory::container::CraftingContainer,
    recipe_key: &steel_utils::Identifier,
) -> bool {
    crate::inventory::recipe_manager::find_recipe(crafting)
        .is_some_and(|recipe| recipe.key() == recipe_key)
}

/// Removes the planned stacks from the player's inventory.
///
/// Vanilla `ServerPlaceRecipe.moveItemToGrid` takes from the slot its own search
/// picked. The grid contents are put back into the inventory first, so the exact
/// slot may have changed; the recorded slot is therefore tried first and the
/// item type is resolved again if the slot no longer matches.
pub fn apply_inventory_take(
    guard: &mut ContainerLockGuard,
    player: &crate::player::Player,
    take: &[(usize, ItemStack)],
) {
    if take.is_empty() {
        return;
    }
    let id = crate::inventory::lock::ContainerId::from_arc(&player.inventory);
    let Some(inventory) = guard.get_mut(id) else {
        return;
    };
    let size = inventory.get_container_size();
    for (slot, stack) in take {
        let mut remaining = stack.count();
        let preferred =
            (*slot < size && inventory.get_item(*slot).item() == stack.item()).then_some(*slot);
        let target = preferred.or_else(|| {
            (0..size).find(|index| {
                let current = inventory.get_item(*index);
                !current.is_empty() && current.item() == stack.item()
            })
        });
        let Some(slot) = target else {
            continue;
        };
        while remaining > 0 {
            let current = inventory.get_item(slot).clone();
            if current.is_empty() || current.item() != stack.item() {
                break;
            }
            if current.count() <= remaining {
                remaining -= current.count();
                inventory.set_item(slot, ItemStack::empty());
            } else {
                inventory.set_item(slot, current.copy_with_count(current.count() - remaining));
                remaining = 0;
            }
        }
    }
    inventory.set_changed();
}

/// Applies a [`PlacementPlan`] to a crafting grid container and the player's
/// inventory, then recomputes the result slot.
///
/// This is vanilla's `ServerPlaceRecipe.clearGrid` + `PlaceRecipeHelper`
/// output application, shared by every crafting menu.
pub fn apply_plan(
    guard: &mut ContainerLockGuard,
    player: &crate::player::Player,
    plan: PlacementPlan,
    crafting: crate::inventory::lock::ContainerId,
    handler: &crate::inventory::slots::CraftingHandler,
) -> PostPlaceAction {
    for stack in plan.inventory_give {
        player.add_item_or_drop_with_guard(
            guard,
            stack,
            crate::player::player_inventory::Prediction::Predicted,
        );
    }
    apply_inventory_take(guard, player, &plan.inventory_take);

    if let Some(container) =
        guard.get_typed_mut::<crate::inventory::container::CraftingContainer>(crafting)
    {
        for (index, stack) in plan.grid.into_iter().enumerate() {
            if index < container.get_container_size() {
                container.set_item(index, stack);
            }
        }
        container.set_changed();
    }

    crate::inventory::slots::ResultHandler::update_result(handler, guard);
    plan.action
}

/// Vanilla `AbstractFurnaceMenu`'s `recipeMatches`: the single input slot
/// satisfies the recipe's only ingredient.
#[must_use]
pub fn single_item_matches(input: &ItemStack, placement: &PlacementInfo) -> bool {
    placement
        .ingredients
        .first()
        .is_some_and(|ingredient| ingredient.test(input))
        && placement.slots_to_ingredient.len() == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(item: ItemRef, count: i32) -> ItemStack {
        ItemStack::with_count(item, count)
    }

    /// Vanilla `testClearGrid` merges repeated items into a single free slot
    /// before spending another one, so a grid of two identical stacks needs only
    /// one free inventory slot.
    #[test]
    fn clear_grid_merges_equal_stacks_into_one_free_slot() {
        let stone = &*steel_registry::vanilla_items::STONE;
        let inventory = vec![stack(stone, 64), ItemStack::empty(), ItemStack::empty()];
        let grid = vec![stack(stone, 10), stack(stone, 10)];

        assert!(
            test_clear_grid(&inventory, &grid),
            "two 10-stone stacks merge into the single 14-stone free slot"
        );
    }

    /// Without any free space and without a matching partial slot the grid
    /// cannot be cleared, so vanilla returns early with `NOTHING`.
    #[test]
    fn clear_grid_fails_when_the_inventory_is_full() {
        let stone = &*steel_registry::vanilla_items::STONE;
        let dirt = &*steel_registry::vanilla_items::DIRT;
        let inventory = vec![stack(dirt, 64), stack(dirt, 64), stack(dirt, 64)];
        let grid = vec![stack(stone, 1)];

        assert!(!test_clear_grid(&inventory, &grid));
    }

    /// A stack with remaining space absorbs everything, whatever the count.
    #[test]
    fn clear_grid_accepts_a_partially_filled_matching_slot() {
        let stone = &*steel_registry::vanilla_items::STONE;
        let inventory = vec![stack(stone, 60), stack(stone, 64)];
        let grid = vec![stack(stone, 4)];

        assert!(test_clear_grid(&inventory, &grid));
    }
}
