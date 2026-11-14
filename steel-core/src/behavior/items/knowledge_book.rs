//! `knowledge_book` item behavior.
//!
//! Vanilla `KnowledgeBookItem.use` turns the `minecraft:recipes` component into
//! real items and hands them to the player, then consumes one book. Recipes the
//! player cannot resolve are skipped silently, which is why a book with an empty
//! or bogus recipe list simply disappears.

use steel_macros::item_behavior;
use steel_registry::data_components::vanilla_components::RECIPES;
use steel_registry::item_stack_template::ItemStackTemplate;
use steel_registry::recipe::{
    CookingRecipe, CraftingRecipe, SmithingRecipe, StonecuttingRecipe, UntypedRecipeRef,
};
use steel_registry::stat::vanilla_stat_types;
use steel_registry::{REGISTRY, item_stack::ItemStack};

use crate::behavior::context::{InteractionResult, UseItemContext};
use crate::behavior::item::ItemBehavior;
use crate::player::player_inventory::Prediction;

/// Behavior for the `knowledge_book` item (`KnowledgeBookItem`).
#[item_behavior(class = "KnowledgeBookItem")]
pub struct KnowledgeBookItem;

impl ItemBehavior for KnowledgeBookItem {
    fn use_item(&self, context: &mut UseItemContext) -> InteractionResult {
        let keys = context
            .inv
            .with_item(|item| item.get(RECIPES).map(|recipes| recipes.keys().to_vec()));
        let Some(keys) = keys else {
            // Vanilla returns early without a result when the component is
            // missing, which leaves the book untouched.
            return InteractionResult::Pass;
        };

        // Vanilla resolves every key against the recipe manager and jumps to the
        // next one when it is unknown; whatever resolves is granted in full.
        let items: Vec<ItemStack> = keys
            .iter()
            .filter_map(|key| REGISTRY.recipes.by_key(key))
            .filter_map(recipe_result)
            .collect();

        for stack in items {
            let leftover = context.player.inventory.lock().add_or_return(stack);
            if !leftover.is_empty() {
                // Vanilla drops anything the inventory could not take.
                let _ = context
                    .player
                    .drop_item(leftover, false, Prediction::Predicted);
            }
        }

        context.inv.with_item(|item| {
            context
                .player
                .award_stat(&vanilla_stat_types::ITEM_USED, item.item());
            item.shrink(1);
        });
        InteractionResult::Success
    }
}

/// Extracts the crafted result of one recipe.
///
/// Recipes without a result (the repair and map-extending families) contribute
/// nothing, exactly like vanilla.
fn recipe_result(recipe: UntypedRecipeRef) -> Option<ItemStack> {
    if let Some(data) = recipe.downcast_data::<CraftingRecipe>() {
        return data.result().map(ItemStackTemplate::create);
    }
    if let Some(data) = recipe.downcast_data::<SmithingRecipe>() {
        return match data {
            SmithingRecipe::Transform(transform) => Some(transform.result.create()),
            SmithingRecipe::Trim(_) => None,
        };
    }
    if let Some(data) = recipe.downcast_data::<StonecuttingRecipe>() {
        return Some(data.result.create());
    }
    if let Some(data) = recipe.downcast_data::<CookingRecipe>() {
        return Some(data.result.create());
    }
    None
}

#[cfg(test)]
mod tests {
    use steel_registry::data_components::Recipes;
    use steel_registry::data_components::vanilla_components::RECIPES;
    use steel_registry::item_stack::ItemStack;
    use steel_registry::{init_vanilla_registry, vanilla_items};
    use steel_utils::Identifier;

    use super::*;

    #[test]
    fn knowledge_book_resolves_recipe_results() {
        init_vanilla_registry();
        let key = Identifier::vanilla_static("stick");
        let recipe = REGISTRY.recipes.by_key(&key).expect("stick recipe exists");
        let stack = recipe_result(recipe).expect("stick has a result");
        assert_eq!(stack.item, &*vanilla_items::STICK);
        assert_eq!(stack.count, 4, "vanilla crafts four sticks at once");
    }

    #[test]
    fn knowledge_book_component_keeps_its_keys() {
        init_vanilla_registry();
        let stack = ItemStack::new(&*vanilla_items::KNOWLEDGE_BOOK);
        assert_eq!(
            stack.get(RECIPES),
            Some(&Recipes::empty()),
            "a plain knowledge book carries an empty recipe list"
        );
        let recipes = Recipes::new(vec![Identifier::vanilla_static("stick")]);
        assert_eq!(recipes.keys().len(), 1, "the recipe list round-trips");
        assert_eq!(recipes.keys()[0].path.as_ref(), "stick");
    }
}
