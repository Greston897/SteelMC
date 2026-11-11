//! Per-player recipe book state.
//!
//! Vanilla keeps a `ServerRecipeBook` on every `ServerPlayer`: the set of recipe
//! keys the player has unlocked plus the four tab settings. The client mirrors the
//! same list, so every change has to produce a delta packet with the display index
//! the client knows that recipe by.
//!
//! `update_recipes` only carries the lookup tables and the stonecutter entries;
//! the displays themselves go out in `recipe_book_add`.

use std::collections::HashMap;

use steel_protocol::packets::game::{
    CRecipeBookAdd, CRecipeBookRemove, CRecipeBookSettings, CUpdateRecipes, RecipeBookAddEntry,
    RecipeBookTypeSettings, StonecutterEntry,
};
use steel_registry::recipe::{Ingredient, StonecuttingRecipe, display_for_recipe};
use steel_registry::slot_display::{RecipeDisplay, SlotDisplay};
use steel_registry::{REGISTRY, RegistryEntry as _, TaggedRegistryExt as _};
use steel_utils::Identifier;

/// The four recipe book tabs, in packet order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecipeBookType {
    /// Crafting table.
    Crafting,
    /// Furnace.
    Furnace,
    /// Blast furnace.
    BlastFurnace,
    /// Smoker.
    Smoker,
}

/// Open and filter state of one tab.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BookTypeSettings {
    /// Whether the book is open in this tab.
    pub open: bool,
    /// Whether the book is filtered to craftable recipes.
    pub filtering: bool,
}

impl From<BookTypeSettings> for RecipeBookTypeSettings {
    fn from(settings: BookTypeSettings) -> Self {
        Self {
            open: settings.open,
            filtering: settings.filtering,
        }
    }
}

/// One recipe as it is shown in the book.
#[derive(Debug, Clone)]
pub struct RecipeBookEntry {
    /// The recipe key, used for persistence and lookups.
    pub key: Identifier,
    /// Group used to sort the book, if the recipe declares one.
    pub group: Option<String>,
    /// Recipe book category index.
    pub category: i32,
    /// Ingredients needed for ghost placement, if any.
    pub requirements: Option<Vec<Ingredient>>,
    /// Position in the client's display list.
    pub display_index: i32,
}

/// A player's recipe book.
///
/// The display list is stable: entries are appended and only compacted on reset,
/// which keeps the indices in `recipe_book_add` and `recipe_book_remove` valid.
#[derive(Debug)]
pub struct ServerRecipeBook {
    /// Displays in the order the client knows them, indexed by display index.
    displays: Vec<Identifier>,
    /// Per key bookkeeping, so removals and dedup stay cheap.
    entries: HashMap<Identifier, RecipeBookEntry>,
    settings: HashMap<RecipeBookType, BookTypeSettings>,
    /// Group name to group index, assigned in first-seen order like vanilla.
    group_ids: HashMap<String, i32>,
}

impl Default for ServerRecipeBook {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerRecipeBook {
    /// Creates an empty book with every tab closed and unfiltered.
    #[must_use]
    pub fn new() -> Self {
        let mut settings = HashMap::new();
        for book_type in [
            RecipeBookType::Crafting,
            RecipeBookType::Furnace,
            RecipeBookType::BlastFurnace,
            RecipeBookType::Smoker,
        ] {
            settings.insert(book_type, BookTypeSettings::default());
        }
        Self {
            displays: Vec::new(),
            entries: HashMap::new(),
            settings,
            group_ids: HashMap::new(),
        }
    }

    /// Returns the settings of one tab.
    #[must_use]
    pub fn settings(&self, book_type: RecipeBookType) -> BookTypeSettings {
        self.settings.get(&book_type).copied().unwrap_or_default()
    }

    /// Replaces the settings of one tab.
    pub fn set_settings(&mut self, book_type: RecipeBookType, settings: BookTypeSettings) {
        self.settings.insert(book_type, settings);
    }

    /// Number of recipes the player has unlocked.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the player has unlocked nothing yet.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Whether the player has unlocked the recipe with this key.
    #[must_use]
    pub fn contains(&self, key: &Identifier) -> bool {
        self.entries.contains_key(key)
    }

    /// Returns every unlocked recipe key.
    pub fn keys(&self) -> impl Iterator<Item = &Identifier> + '_ {
        self.entries.keys()
    }

    /// Forgets every unlock and display, as when a datapack reload invalidates them.
    pub fn clear(&mut self) {
        self.displays.clear();
        self.entries.clear();
        self.group_ids.clear();
    }

    /// Unlocks one recipe.
    ///
    /// Returns `None` when the key is already known or the registry has no
    /// display for it, so the caller can decide whether to send a packet.
    pub fn add(&mut self, key: &Identifier) -> Option<i32> {
        if self.entries.contains_key(key) {
            return None;
        }
        let recipe = REGISTRY.recipes.by_key(key)?;
        let display = display_for_recipe(recipe)?;

        let index = i32::try_from(self.displays.len()).ok()?;
        let entry = RecipeBookEntry {
            key: key.clone(),
            group: None,
            category: 0,
            requirements: requirements_of(&display),
            display_index: index,
        };
        self.displays.push(key.clone());
        self.entries.insert(key.clone(), entry);
        Some(index)
    }

    /// Unlocks every registered recipe the server can render.
    ///
    /// Vanilla does this from the recipe manager when a player joins a world
    /// that grants everything, and a knowledge book with a full recipe list ends
    /// up with the same contents. Recipes Steel has no display for are skipped.
    pub fn add_all_registered(&mut self) -> usize {
        let keys: Vec<Identifier> = REGISTRY
            .recipes
            .iter()
            .map(|recipe| recipe.key().clone())
            .collect();
        let mut added = 0;
        for key in keys {
            if self.add(&key).is_some() {
                added += 1;
            }
        }
        added
    }

    /// Locks one recipe again.
    ///
    /// The display index stays allocated so the client's numbering does not shift.
    pub fn remove(&mut self, key: &Identifier) -> Option<i32> {
        let entry = self.entries.remove(key)?;
        Some(entry.display_index)
    }

    /// Builds the packets that populate a freshly joined client.
    ///
    /// Vanilla sends them together, in this order, right after the player spawns:
    /// the displays first, then the lookup tables that the stonecutter menu needs.
    #[must_use]
    pub fn join_packets(&self) -> RecipeBookPackets {
        let mut entries = Vec::with_capacity(self.entries.len());
        for key in &self.displays {
            let Some(entry) = self.entries.get(key) else {
                continue;
            };
            let Some(recipe) = REGISTRY.recipes.by_key(key) else {
                continue;
            };
            let Some(display) = display_for_recipe(recipe) else {
                continue;
            };
            entries.push(RecipeBookAddEntry {
                id: entry.display_index,
                display,
                group: entry
                    .group
                    .as_ref()
                    .and_then(|group| self.group_ids.get(group).copied()),
                category: entry.category,
                requirements: entry.requirements.clone(),
                // Notification is only interesting for newly unlocked recipes.
                flags: 0,
            });
        }

        RecipeBookPackets {
            settings: self.settings_packet(),
            add: CRecipeBookAdd {
                entries,
                replace: true,
            },
            update: update_recipes_packet(),
        }
    }

    /// Builds the settings packet for all four tabs.
    #[must_use]
    pub fn settings_packet(&self) -> CRecipeBookSettings {
        CRecipeBookSettings {
            crafting: self.settings(RecipeBookType::Crafting).into(),
            furnace: self.settings(RecipeBookType::Furnace).into(),
            blast_furnace: self.settings(RecipeBookType::BlastFurnace).into(),
            smoker: self.settings(RecipeBookType::Smoker).into(),
        }
    }

    /// Builds a removal packet for the given displays.
    #[must_use]
    pub fn remove_packet(displays: Vec<i32>) -> CRecipeBookRemove {
        CRecipeBookRemove { recipes: displays }
    }
}

/// The packets that populate a client's recipe book on join.
#[derive(Debug, Clone)]
pub struct RecipeBookPackets {
    /// Tab open and filter state.
    pub settings: CRecipeBookSettings,
    /// Every display, replacing the client's book.
    pub add: CRecipeBookAdd,
    /// Lookup tables and the stonecutter menu.
    pub update: CUpdateRecipes,
}

impl RecipeBookPackets {
    /// Returns the packets in the order vanilla sends them.
    ///
    /// The display list has to reach the client before the lookup tables, because
    /// a stonecutter entry refers to a display the client already knows.
    #[must_use]
    pub fn in_order(&self) -> (&CRecipeBookAdd, &CUpdateRecipes, &CRecipeBookSettings) {
        (&self.add, &self.update, &self.settings)
    }
}

/// Builds the `update_recipes` packet from the registry.
///
/// The property sets are the item sets vanilla uses to decide whether a stack is
/// a valid input for a menu: furnace fuel and the smelting, blasting and smoking
/// inputs. The stonecutter entries are one per registered stonecutting recipe,
/// which is the only source the client has for that menu.
#[must_use]
pub fn update_recipes_packet() -> CUpdateRecipes {
    let mut stonecutter = Vec::new();
    for recipe in REGISTRY.recipes.iter() {
        if recipe.recipe_type().key.path.as_ref() != "stonecutting" {
            continue;
        }
        let Some(data) = recipe.downcast_data::<StonecuttingRecipe>() else {
            continue;
        };
        stonecutter.push(StonecutterEntry {
            input: data.ingredient.clone(),
            display: SlotDisplay::ItemStack(data.result.clone()),
        });
    }

    CUpdateRecipes {
        property_sets: property_sets(),
        stonecutter,
    }
}

/// Item sets the client uses to filter menu inputs.
fn property_sets() -> Vec<(Identifier, Vec<i32>)> {
    let mut sets = Vec::new();
    for (name, tag) in [
        ("minecraft:furnace_input", "minecraft:furnace_input"),
        ("minecraft:blast_furnace_input", "minecraft:blast_furnace_input"),
        ("minecraft:smoker_input", "minecraft:smoker_input"),
        ("minecraft:campfire_input", "minecraft:campfire_input"),
        ("minecraft:furnace_fuel", "minecraft:furnace_fuel"),
    ] {
        let tag = Identifier::vanilla_static(tag);
        let items = REGISTRY
            .items
            .get_tag(&tag)
            .unwrap_or_default()
            .iter()
            .filter_map(|item| i32::try_from(item.try_id()?).ok())
            .collect();
        sets.push((Identifier::vanilla_static(name), items));
    }
    sets
}

/// Extracts the ingredients the client needs for ghost placement.
///
/// Only crafting displays carry them: a ghost recipe places items in a grid, and
/// the grid shape is exactly what `requirements` describes.
fn requirements_of(display: &RecipeDisplay) -> Option<Vec<Ingredient>> {
    match display {
        RecipeDisplay::CraftingShaped { ingredients, .. }
        | RecipeDisplay::CraftingShapeless { ingredients, .. } => Some(
            ingredients
                .iter()
                .filter_map(ingredient_of_display)
                .collect(),
        ),
        RecipeDisplay::Furnace { ingredient, .. } => {
            Some(ingredient_of_display(ingredient).into_iter().collect())
        }
        RecipeDisplay::Stonecutter { ingredient, .. } => {
            Some(ingredient_of_display(ingredient).into_iter().collect())
        }
        RecipeDisplay::Smithing { .. } => None,
    }
}

/// Recovers an [`Ingredient`] from a slot display built by `from_ingredient`.
fn ingredient_of_display(display: &SlotDisplay) -> Option<Ingredient> {
    match display {
        SlotDisplay::Empty => None,
        SlotDisplay::Item(item) => Some(Ingredient::Item(item)),
        SlotDisplay::Tag(tag) => Some(Ingredient::Tag(tag.clone())),
        SlotDisplay::Composite(displays) => {
            let items: Vec<_> = displays
                .iter()
                .filter_map(|display| match display {
                    SlotDisplay::Item(item) => Some(*item),
                    _ => None,
                })
                .collect();
            if items.len() == displays.len() && !items.is_empty() {
                // Rebuild the choice from the leaked slice the registry keeps.
                Some(Ingredient::Choice(leak_items(items)))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Leaks a small item slice so it can back [`Ingredient::Choice`].
///
/// The original ingredient already lives for the lifetime of the process, but the
/// display only kept the items, so the choice has to be rebuilt once here.
fn leak_items(
    items: Vec<steel_registry::items::ItemRef>,
) -> &'static [steel_registry::items::ItemRef] {
    Box::leak(items.into_boxed_slice())
}

#[cfg(test)]
mod tests {
    use super::*;
    use steel_registry::init_vanilla_registry;

    #[test]
    fn a_new_book_is_empty_and_closed() {
        let book = ServerRecipeBook::new();
        assert!(book.is_empty());
        assert_eq!(book.len(), 0);
        assert_eq!(book.settings(RecipeBookType::Crafting), BookTypeSettings::default());
    }

    #[test]
    fn adding_a_known_recipe_allocates_a_display_index() {
        init_vanilla_registry();
        let mut book = ServerRecipeBook::new();
        let key = Identifier::vanilla_static("stick");
        assert_eq!(book.add(&key), Some(0));
        assert!(book.contains(&key));
        assert_eq!(book.add(&key), None, "adding twice is a no-op");
        assert_eq!(book.len(), 1);
    }

    #[test]
    fn unknown_recipes_are_rejected() {
        init_vanilla_registry();
        let mut book = ServerRecipeBook::new();
        let key = Identifier::vanilla_static("not_a_recipe");
        assert_eq!(book.add(&key), None);
        assert!(book.is_empty());
    }

    #[test]
    fn removal_keeps_the_display_numbering_stable() {
        init_vanilla_registry();
        let mut book = ServerRecipeBook::new();
        let first = Identifier::vanilla_static("stick");
        let second = Identifier::vanilla_static("torch");
        assert_eq!(book.add(&first), Some(0));
        assert_eq!(book.add(&second), Some(1));
        assert_eq!(book.remove(&first), Some(0));
        assert_eq!(book.add(&first), None, "a removed key is not re-added automatically");
    }

    #[test]
    fn join_packets_carry_every_unlocked_display() {
        init_vanilla_registry();
        let mut book = ServerRecipeBook::new();
        book.add(&Identifier::vanilla_static("stick"));
        book.add(&Identifier::vanilla_static("torch"));
        let packets = book.join_packets();
        assert_eq!(packets.add.entries.len(), 2);
        assert!(packets.add.replace, "join replaces the client's book");
    }

    #[test]
    fn add_all_registered_unlocks_the_whole_registry() {
        init_vanilla_registry();
        let mut book = ServerRecipeBook::new();
        let added = book.add_all_registered();
        let expected = REGISTRY
            .recipes
            .iter()
            .filter(|recipe| display_for_recipe(*recipe).is_some())
            .count();
        assert_eq!(added, expected);
        assert_eq!(book.len(), expected);
        assert!(expected > 1000, "vanilla ships over a thousand recipes");
    }

    #[test]
    fn update_recipes_lists_every_stonecutter_recipe() {
        init_vanilla_registry();
        let packet = update_recipes_packet();
        let expected = REGISTRY
            .recipes
            .iter()
            .filter(|recipe| recipe.recipe_type().key.path.as_ref() == "stonecutting")
            .count();
        assert_eq!(packet.stonecutter.len(), expected);
        assert!(expected > 0, "vanilla has stonecutting recipes");
    }
}
