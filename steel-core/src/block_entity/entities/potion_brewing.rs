//! Vanilla `PotionBrewing`: the hard-coded mapping from a base potion and an
//! ingredient to a result potion.
//!
//! Vanilla keeps this table in code rather than in a data pack, so it lives here
//! as a static slice. The helpers mirror the predicates that decide whether a
//! brew operation may start, matching `PotionBrewing` method for method.

use steel_registry::item_stack::ItemStack;
use steel_registry::{REGISTRY, RegistryExt, vanilla_items};
use steel_utils::Identifier;

/// One base-potion-plus-ingredient brewing recipe.
#[derive(Debug)]
pub struct PotionMix {
    /// Ingredient consumed by the operation.
    pub ingredient: Identifier,
    /// Potion in the slot before the operation.
    pub from: Identifier,
    /// Potion in the slot after the operation.
    pub to: Identifier,
}

const fn mix(ingredient: &'static str, from: &'static str, to: &'static str) -> PotionMix {
    PotionMix {
        ingredient: Identifier::vanilla_static(ingredient),
        from: Identifier::vanilla_static(from),
        to: Identifier::vanilla_static(to),
    }
}

/// Base-potion-plus-ingredient recipes, matching vanilla `PotionBrewing.POTION_MIXES`.
pub static POTION_MIXES: &[PotionMix] = &[
    mix("nether_wart", "water", "awkward"),
    mix("redstone", "water", "mundane"),
    mix("fermented_spider_eye", "water", "weakness"),
    mix("glowstone_dust", "water", "thick"),
    // Awkward base.
    mix("golden_carrot", "awkward", "night_vision"),
    mix("rabbit_foot", "awkward", "leaping"),
    mix("magma_cream", "awkward", "fire_resistance"),
    mix("blaze_powder", "awkward", "strength"),
    mix("sugar", "awkward", "swiftness"),
    mix("pufferfish", "awkward", "water_breathing"),
    mix("glistering_melon_slice", "awkward", "healing"),
    mix("spider_eye", "awkward", "poison"),
    mix("phantom_membrane", "awkward", "slow_falling"),
    // Amplified tier.
    mix("glowstone_dust", "swiftness", "strong_swiftness"),
    mix("glowstone_dust", "leaping", "strong_leaping"),
    mix("glowstone_dust", "healing", "strong_healing"),
    mix("glowstone_dust", "poison", "strong_poison"),
    mix("glowstone_dust", "strength", "strong_strength"),
    // Extended tier.
    mix("redstone", "swiftness", "long_swiftness"),
    mix("redstone", "leaping", "long_leaping"),
    mix("redstone", "poison", "long_poison"),
    mix("redstone", "strength", "long_strength"),
    mix("redstone", "night_vision", "long_night_vision"),
    mix("redstone", "fire_resistance", "long_fire_resistance"),
    mix("redstone", "water_breathing", "long_water_breathing"),
    mix("redstone", "slow_falling", "long_slow_falling"),
    // Corruptions.
    mix("fermented_spider_eye", "swiftness", "slowness"),
    mix("fermented_spider_eye", "leaping", "slowness"),
    mix("fermented_spider_eye", "healing", "harming"),
    mix("fermented_spider_eye", "poison", "harming"),
];

/// Mirrors vanilla `PotionBrewing.mix(ItemStack ingredient, ItemStack input)`.
///
/// Returns the potion that `input` becomes, or `None` when no recipe matches.
#[must_use]
pub fn mix(input: &ItemStack, ingredient: &ItemStack) -> Option<Identifier> {
    let current = input.potion_key()?;
    let ingredient_key = ingredient.item().key();
    POTION_MIXES
        .iter()
        .find(|recipe| &recipe.ingredient == ingredient_key && recipe.from == current)
        .map(|recipe| recipe.to.clone())
}

/// Vanilla `PotionBrewing.isBrewablePotion`: potions and bottles can brew.
#[must_use]
pub fn is_brewable_potion(stack: &ItemStack) -> bool {
    let key = stack.item().key();
    key == &Identifier::vanilla_static("potion")
        || key == &Identifier::vanilla_static("splash_potion")
        || key == &Identifier::vanilla_static("lingering_potion")
        || key == &Identifier::vanilla_static("glass_bottle")
}

/// Vanilla `PotionBrewing.isIngredient`: whether the stack can start a brew.
#[must_use]
pub fn is_ingredient(stack: &ItemStack) -> bool {
    !stack.is_empty() && is_brewable_potion(stack)
}

/// Vanilla `PotionBrewing.hasRecipe`: whether a potion and an ingredient combine.
#[must_use]
pub fn has_recipe(input: &ItemStack, ingredient: &ItemStack) -> bool {
    mix(input, ingredient).is_some()
}

/// Vanilla `PotionBrewing.isFuel`: blaze powder fuels the stand.
#[must_use]
pub fn is_fuel(stack: &ItemStack) -> bool {
    stack.item().key() == &Identifier::vanilla_static("blaze_powder")
}

/// Returns whether the given potion key is registered.
#[must_use]
pub fn is_registered_potion(key: &Identifier) -> bool {
    REGISTRY.potions.by_key(key).is_some()
}

/// The vanilla potion item, used when creating a brewed result.
#[must_use]
pub fn potion_item() -> &'static Identifier {
    vanilla_items::POTION.key()
}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_items};

    use super::*;

    fn potion(key: &str) -> ItemStack {
        let mut stack = ItemStack::new(&vanilla_items::POTION);
        stack.set_potion(&Identifier::new("minecraft", key));
        stack
    }

    #[test]
    fn water_plus_nether_wart_makes_awkward() {
        init_vanilla_registry();
        let result = mix(
            &potion("water"),
            &ItemStack::with_count(&vanilla_items::NETHER_WART, 1),
        )
        .expect("recipe");
        assert_eq!(result, Identifier::vanilla_static("awkward"));
    }

    #[test]
    fn redstone_extends_swiftness() {
        init_vanilla_registry();
        let result = mix(
            &potion("swiftness"),
            &ItemStack::with_count(&vanilla_items::REDSTONE, 1),
        )
        .expect("recipe");
        assert_eq!(result, Identifier::vanilla_static("long_swiftness"));
    }

    #[test]
    fn unknown_combination_has_no_recipe() {
        init_vanilla_registry();
        assert!(
            mix(
                &potion("water"),
                &ItemStack::with_count(&vanilla_items::SUGAR, 1),
            )
            .is_none()
        );
    }

    #[test]
    fn every_recipe_targets_a_registered_potion() {
        init_vanilla_registry();
        for recipe in POTION_MIXES {
            assert!(is_registered_potion(&recipe.to), "unknown target {:?}", recipe.to);
            assert!(is_registered_potion(&recipe.from), "unknown base {:?}", recipe.from);
        }
    }

    #[test]
    fn blaze_powder_is_fuel_and_never_an_ingredient() {
        init_vanilla_registry();
        let powder = ItemStack::with_count(&vanilla_items::BLAZE_POWDER, 1);
        assert!(is_fuel(&powder));
        assert!(!is_ingredient(&powder));
    }

    #[test]
    fn potions_are_brewable() {
        init_vanilla_registry();
        assert!(is_brewable_potion(&potion("water")));
        assert!(is_brewable_potion(&ItemStack::new(&vanilla_items::GLASS_BOTTLE)));
        assert!(!is_brewable_potion(&ItemStack::new(
            &vanilla_items::NETHER_WART
        )));
    }
}
