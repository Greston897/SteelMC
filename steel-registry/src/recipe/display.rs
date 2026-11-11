//! Conversion from internal recipe data to the displays sent in `update_recipes`.
//!
//! Java builds these on demand from the loaded recipe manager; Steel does the same
//! so a datapack reload only has to invalidate the cached packet.

use crate::item_stack_template::ItemStackTemplate;
use crate::recipe::{CookingRecipe, CraftingRecipe, SmithingRecipe, StonecuttingRecipe};
use crate::slot_display::{RecipeDisplay, SlotDisplay};
use crate::vanilla_items;

/// Builds the display of one registered recipe.
///
/// Returns `None` for recipe data Steel has no display shape for, which the
/// caller skips: the client tolerates a shorter list and simply cannot see those
/// entries in the recipe book.
#[must_use]
pub fn display_for_recipe(recipe: crate::recipe::UntypedRecipeRef) -> Option<RecipeDisplay> {
    if let Some(data) = recipe.downcast_data::<CraftingRecipe>() {
        return Some(crafting_display(data));
    }
    if let Some(data) = recipe.downcast_data::<SmithingRecipe>() {
        return Some(smithing_display(data));
    }
    // Every cooking recipe shares one data type, so the registered recipe type
    // decides which station icon the client shows.
    let station = match recipe.recipe_type().key.path.as_ref() {
        "smelting" => &vanilla_items::FURNACE,
        "blasting" => &vanilla_items::BLAST_FURNACE,
        "smoking" => &vanilla_items::SMOKER,
        "campfire_cooking" => &vanilla_items::CAMPFIRE,
        "stonecutting" => {
            let data = recipe.downcast_data::<StonecuttingRecipe>()?;
            return Some(RecipeDisplay::Stonecutter {
                ingredient: SlotDisplay::from_ingredient(&data.ingredient),
                result: result_display(&data.result),
                crafting_station: SlotDisplay::Item(&vanilla_items::STONECUTTER),
            });
        }
        _ => return None,
    };
    let data = recipe.downcast_data::<CookingRecipe>()?;
    Some(RecipeDisplay::Furnace {
        ingredient: SlotDisplay::from_ingredient(&data.ingredient),
        fuel: SlotDisplay::AnyFuel,
        result: result_display(&data.result),
        crafting_station: SlotDisplay::Item(station),
        cooking_time: data.cooking_time,
        experience: data.experience,
    })
}

fn crafting_display(recipe: &CraftingRecipe) -> RecipeDisplay {
    let station = RecipeDisplay::crafting_table_station();
    match recipe {
        CraftingRecipe::Shaped(shaped) => RecipeDisplay::CraftingShaped {
            width: i32::try_from(shaped.width).unwrap_or_default(),
            height: i32::try_from(shaped.height).unwrap_or_default(),
            ingredients: shaped
                .pattern
                .iter()
                .map(SlotDisplay::from_ingredient)
                .collect(),
            result: result_display(&shaped.result),
            crafting_station: station,
        },
        CraftingRecipe::Shapeless(shapeless) => RecipeDisplay::CraftingShapeless {
            ingredients: shapeless
                .ingredients
                .iter()
                .map(SlotDisplay::from_ingredient)
                .collect(),
            result: result_display(&shapeless.result),
            crafting_station: station,
        },
        other => {
            // Remaining crafting types have no dedicated display in vanilla either;
            // they are rendered from the first ingredient and the result.
            let ingredients =
                other
                    .properties()
                    .placement
                    .as_ref()
                    .map_or_else(Vec::new, |placement| {
                        placement
                            .ingredients
                            .iter()
                            .map(SlotDisplay::from_ingredient)
                            .collect()
                    });
            let result = other
                .result()
                .map_or_else(|| SlotDisplay::Empty, result_display);
            RecipeDisplay::CraftingShapeless {
                ingredients,
                result,
                crafting_station: station,
            }
        }
    }
}

fn smithing_display(recipe: &SmithingRecipe) -> RecipeDisplay {
    match recipe {
        SmithingRecipe::Transform(transform) => RecipeDisplay::Smithing {
            template: transform
                .template
                .as_ref()
                .map_or(SlotDisplay::Empty, SlotDisplay::from_ingredient),
            base: SlotDisplay::from_ingredient(&transform.base),
            addition: transform
                .addition
                .as_ref()
                .map_or(SlotDisplay::Empty, SlotDisplay::from_ingredient),
            result: result_display(&transform.result),
            crafting_station: SlotDisplay::Item(&vanilla_items::SMITHING_TABLE),
        },
        SmithingRecipe::Trim(trim) => RecipeDisplay::Smithing {
            template: SlotDisplay::from_ingredient(&trim.template),
            base: SlotDisplay::from_ingredient(&trim.base),
            addition: SlotDisplay::from_ingredient(&trim.addition),
            result: SlotDisplay::Empty,
            crafting_station: SlotDisplay::Item(&vanilla_items::SMITHING_TABLE),
        },
    }
}

/// Renders a recipe result as a full stack so counts and components survive.
fn result_display(template: &ItemStackTemplate) -> SlotDisplay {
    SlotDisplay::ItemStack(template.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_vanilla_registry;

    #[test]
    fn every_vanilla_recipe_produces_a_display() {
        init_vanilla_registry();
        let mut total = 0;
        let mut missing = Vec::new();
        for recipe in crate::REGISTRY.recipes.iter() {
            total += 1;
            if display_for_recipe(recipe).is_none() {
                missing.push(recipe.key().to_string());
            }
        }
        assert!(total > 0, "vanilla registry should contain recipes");
        assert!(missing.is_empty(), "recipes without a display: {missing:?}");
    }

    #[test]
    fn shaped_recipes_report_a_full_grid() {
        init_vanilla_registry();
        let recipe = crate::REGISTRY
            .recipes
            .by_key(&steel_utils::Identifier::vanilla_static("crafting_table"))
            .expect("crafting table recipe exists");
        match display_for_recipe(recipe).expect("crafting table has a display") {
            RecipeDisplay::CraftingShaped {
                width,
                height,
                ingredients,
                ..
            } => {
                assert_eq!(width, 2);
                assert_eq!(height, 2);
                assert_eq!(ingredients.len(), 4);
            }
            other => panic!("expected a shaped display, got {other:?}"),
        }
    }
}
