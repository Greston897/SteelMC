//! Vanilla `PotionBrewing`: the hard-coded mapping from a base potion and an
//! ingredient to a result potion.
//!
//! Vanilla keeps this table in code rather than in a data pack, so it lives here
//! as a static slice. The helpers mirror the predicates that decide whether a
//! brew operation may start, matching `PotionBrewing` method for method.

use steel_registry::item_stack::ItemStack;
use steel_registry::{REGISTRY, RegistryEntry as _, RegistryExt, vanilla_items};
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

const fn potion_mix(ingredient: &'static str, from: &'static str, to: &'static str) -> PotionMix {
    PotionMix {
        ingredient: Identifier::vanilla_static(ingredient),
        from: Identifier::vanilla_static(from),
        to: Identifier::vanilla_static(to),
    }
}

/// Base-potion-plus-ingredient recipes, matching vanilla `PotionBrewing.POTION_MIXES`.
pub static POTION_MIXES: &[PotionMix] = &[
    potion_mix("nether_wart", "water", "awkward"),
    potion_mix("redstone", "water", "mundane"),
    potion_mix("fermented_spider_eye", "water", "weakness"),
    potion_mix("glowstone_dust", "water", "thick"),
    // Awkward base.
    potion_mix("golden_carrot", "awkward", "night_vision"),
    potion_mix("rabbit_foot", "awkward", "leaping"),
    potion_mix("magma_cream", "awkward", "fire_resistance"),
    potion_mix("blaze_powder", "awkward", "strength"),
    potion_mix("sugar", "awkward", "swiftness"),
    potion_mix("pufferfish", "awkward", "water_breathing"),
    potion_mix("glistering_melon_slice", "awkward", "healing"),
    potion_mix("spider_eye", "awkward", "poison"),
    potion_mix("phantom_membrane", "awkward", "slow_falling"),
    // Amplified tier.
    potion_mix("glowstone_dust", "swiftness", "strong_swiftness"),
    potion_mix("glowstone_dust", "leaping", "strong_leaping"),
    potion_mix("glowstone_dust", "healing", "strong_healing"),
    potion_mix("glowstone_dust", "poison", "strong_poison"),
    potion_mix("glowstone_dust", "strength", "strong_strength"),
    // Extended tier.
    potion_mix("redstone", "swiftness", "long_swiftness"),
    potion_mix("redstone", "leaping", "long_leaping"),
    potion_mix("redstone", "poison", "long_poison"),
    potion_mix("redstone", "strength", "long_strength"),
    potion_mix("redstone", "night_vision", "long_night_vision"),
    potion_mix("redstone", "fire_resistance", "long_fire_resistance"),
    potion_mix("redstone", "water_breathing", "long_water_breathing"),
    potion_mix("redstone", "slow_falling", "long_slow_falling"),
    // Corruptions.
    potion_mix("fermented_spider_eye", "swiftness", "slowness"),
    potion_mix("fermented_spider_eye", "leaping", "slowness"),
    potion_mix("fermented_spider_eye", "healing", "harming"),
    potion_mix("fermented_spider_eye", "poison", "harming"),
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
