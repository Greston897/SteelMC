//! Clientbound recipe book packets.
//!
//! Vanilla splits the recipe book into four packets that always travel together:
//! `update_recipes` carries the lookup tables and the stonecutter entries,
//! `recipe_book_add` and `recipe_book_remove` mutate the numbered display list,
//! and `recipe_book_settings` carries the filter and open state per book type.
//!
//! Note that the recipe displays themselves live in `recipe_book_add`: the
//! `update_recipes` packet does not repeat them.

use std::io::{self, Result, Write};

use steel_macros::ClientPacket;
use steel_registry::packets::play::{
    C_RECIPE_BOOK_ADD, C_RECIPE_BOOK_REMOVE, C_RECIPE_BOOK_SETTINGS, C_UPDATE_RECIPES,
};
use steel_registry::recipe::Ingredient;
use steel_registry::slot_display::{RecipeDisplay, SlotDisplay};
use steel_utils::Identifier;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;

/// One entry of the stonecutter menu.
#[derive(Clone, Debug)]
pub struct StonecutterEntry {
    /// What may be cut.
    pub input: Ingredient,
    /// What the entry produces, shown as the button icon.
    pub display: SlotDisplay,
}

/// Replaces the client's recipe lookup tables and stonecutter menu.
///
/// Vanilla derives the stonecutter list from nothing but this packet, so it must
/// be sent on join or the menu stays empty even though crafting works.
#[derive(ClientPacket, Clone, Debug)]
#[packet_id(Play = C_UPDATE_RECIPES)]
pub struct CUpdateRecipes {
    /// Items accepted by each recipe property set, keyed by set identifier.
    pub property_sets: Vec<(Identifier, Vec<i32>)>,
    /// The stonecutter entries.
    pub stonecutter: Vec<StonecutterEntry>,
}

impl WriteTo for CUpdateRecipes {
    fn write(&self, writer: &mut impl Write) -> Result<()> {
        VarInt(len_i32(self.property_sets.len())?).write(writer)?;
        for (key, items) in &self.property_sets {
            key.write(writer)?;
            VarInt(len_i32(items.len())?).write(writer)?;
            for item in items {
                VarInt(*item).write(writer)?;
            }
        }

        VarInt(len_i32(self.stonecutter.len())?).write(writer)?;
        for entry in &self.stonecutter {
            entry.input.write(writer)?;
            entry.display.write(writer)?;
        }
        Ok(())
    }
}

/// One entry of [`CRecipeBookAdd`].
#[derive(Clone, Debug)]
pub struct RecipeBookAddEntry {
    /// Index of this display in the client's recipe book list.
    pub id: i32,
    /// The display to render.
    pub display: RecipeDisplay,
    /// Group index the recipe belongs to, or `None` for no group.
    pub group: Option<i32>,
    /// Recipe book category the entry is filed under.
    pub category: i32,
    /// Ingredients the client needs for ghost placement, or `None`.
    pub requirements: Option<Vec<Ingredient>>,
    /// Notification and highlight bits.
    pub flags: u8,
}

impl WriteTo for RecipeBookAddEntry {
    fn write(&self, writer: &mut impl Write) -> Result<()> {
        VarInt(self.id).write(writer)?;
        self.display.write(writer)?;
        // Optional VarInt: zero means absent, otherwise the value plus one.
        match self.group {
            None => VarInt(0).write(writer)?,
            Some(group) => VarInt(
                group
                    .checked_add(1)
                    .ok_or_else(|| io::Error::other("recipe group overflows a VarInt"))?,
            )
            .write(writer)?,
        }
        VarInt(self.category).write(writer)?;
        match &self.requirements {
            None => writer.write_all(&[0])?,
            Some(ingredients) => {
                writer.write_all(&[1])?;
                VarInt(len_i32(ingredients.len())?).write(writer)?;
                for ingredient in ingredients {
                    ingredient.write(writer)?;
                }
            }
        }
        writer.write_all(&[self.flags])
    }
}

/// Adds recipes to the player's book.
#[derive(ClientPacket, Clone, Debug)]
#[packet_id(Play = C_RECIPE_BOOK_ADD)]
pub struct CRecipeBookAdd {
    /// The entries to add.
    pub entries: Vec<RecipeBookAddEntry>,
    /// Whether the entries replace the whole book, as on a datapack reload.
    pub replace: bool,
}

impl WriteTo for CRecipeBookAdd {
    fn write(&self, writer: &mut impl Write) -> Result<()> {
        VarInt(len_i32(self.entries.len())?).write(writer)?;
        for entry in &self.entries {
            entry.write(writer)?;
        }
        writer.write_all(&[u8::from(self.replace)])
    }
}

/// Removes displays from the player's book by index.
#[derive(ClientPacket, Clone, Debug)]
#[packet_id(Play = C_RECIPE_BOOK_REMOVE)]
pub struct CRecipeBookRemove {
    /// Display indices to drop.
    pub recipes: Vec<i32>,
}

impl WriteTo for CRecipeBookRemove {
    fn write(&self, writer: &mut impl Write) -> Result<()> {
        VarInt(len_i32(self.recipes.len())?).write(writer)?;
        for recipe in &self.recipes {
            VarInt(*recipe).write(writer)?;
        }
        Ok(())
    }
}

/// One recipe book tab's state.
#[derive(Clone, Copy, Debug, Default)]
pub struct RecipeBookTypeSettings {
    /// Whether the book is open in this tab.
    pub open: bool,
    /// Whether the book is filtered to craftable recipes.
    pub filtering: bool,
}

/// Updates the open and filter state of every recipe book tab.
///
/// The four tabs are always sent in full: crafting, furnace, blast furnace and
/// smoker. There is no length prefix.
#[derive(ClientPacket, Clone, Debug)]
#[packet_id(Play = C_RECIPE_BOOK_SETTINGS)]
pub struct CRecipeBookSettings {
    /// State of the crafting tab.
    pub crafting: RecipeBookTypeSettings,
    /// State of the furnace tab.
    pub furnace: RecipeBookTypeSettings,
    /// State of the blast furnace tab.
    pub blast_furnace: RecipeBookTypeSettings,
    /// State of the smoker tab.
    pub smoker: RecipeBookTypeSettings,
}

impl WriteTo for CRecipeBookSettings {
    fn write(&self, writer: &mut impl Write) -> Result<()> {
        for tab in [&self.crafting, &self.furnace, &self.blast_furnace, &self.smoker] {
            writer.write_all(&[u8::from(tab.open), u8::from(tab.filtering)])?;
        }
        Ok(())
    }
}

fn len_i32(len: usize) -> Result<i32> {
    i32::try_from(len).map_err(|_| io::Error::other("length does not fit in a VarInt"))
}

#[cfg(test)]
mod tests {
    use steel_registry::init_vanilla_registry;
    use steel_registry::recipe::Ingredient;
    use steel_registry::slot_display::{RecipeDisplay, SlotDisplay};
    use steel_registry::vanilla_items;
    use steel_utils::serial::WriteTo as _;

    use super::{
        CRecipeBookAdd, CRecipeBookRemove, CRecipeBookSettings, CUpdateRecipes, RecipeBookAddEntry,
        RecipeBookTypeSettings, StonecutterEntry,
    };

    fn stick_display() -> RecipeDisplay {
        RecipeDisplay::CraftingShapeless {
            ingredients: vec![SlotDisplay::Item(&*vanilla_items::OAK_PLANKS)],
            result: SlotDisplay::Item(&*vanilla_items::STICK),
            crafting_station: RecipeDisplay::crafting_table_station(),
        }
    }

    #[test]
    fn update_recipes_writes_set_count_then_stonecutter_count() {
        init_vanilla_registry();
        let packet = CUpdateRecipes {
            property_sets: Vec::new(),
            stonecutter: vec![StonecutterEntry {
                input: Ingredient::Item(&vanilla_items::STONE),
                display: SlotDisplay::Item(&*vanilla_items::STONE_STAIRS),
            }],
        };
        let mut bytes = Vec::new();
        packet.write(&mut bytes).expect("packet should encode");
        assert_eq!(bytes[0], 0, "no property sets");
        assert_eq!(bytes[1], 1, "one stonecutter entry");
        assert_eq!(bytes[2], 2, "the ingredient is a two-item holder set");
    }

    #[test]
    fn recipe_book_add_ends_with_the_replace_flag() {
        init_vanilla_registry();
        let packet = CRecipeBookAdd {
            entries: vec![RecipeBookAddEntry {
                id: 7,
                display: stick_display(),
                group: Some(1),
                category: 0,
                requirements: None,
                flags: 2,
            }],
            replace: true,
        };
        let mut bytes = Vec::new();
        packet.write(&mut bytes).expect("packet should encode");
        assert_eq!(bytes[0], 1, "one entry");
        assert_eq!(bytes[1], 7, "entry id");
        assert_eq!(*bytes.last().expect("non-empty"), 1, "replace flag");
    }

    #[test]
    fn optional_group_is_encoded_plus_one() {
        init_vanilla_registry();
        let entry = RecipeBookAddEntry {
            id: 0,
            display: stick_display(),
            group: Some(0),
            category: 0,
            requirements: None,
            flags: 0,
        };
        let mut bytes = Vec::new();
        entry.write(&mut bytes).expect("entry should encode");
        // The display ends with the crafting-table station, so the group byte is
        // the last one before the category.
        let tail = &bytes[bytes.len() - 3..];
        assert_eq!(tail, [1, 0, 0], "group 0 becomes 1, then category, then flags");
    }

    #[test]
    fn recipe_book_remove_writes_indices() {
        let packet = CRecipeBookRemove {
            recipes: vec![3, 9],
        };
        let mut bytes = Vec::new();
        packet.write(&mut bytes).expect("packet should encode");
        assert_eq!(bytes, [2, 3, 9]);
    }

    #[test]
    fn settings_write_four_unprefixed_pairs() {
        let packet = CRecipeBookSettings {
            crafting: RecipeBookTypeSettings {
                open: true,
                filtering: false,
            },
            furnace: RecipeBookTypeSettings::default(),
            blast_furnace: RecipeBookTypeSettings::default(),
            smoker: RecipeBookTypeSettings::default(),
        };
        let mut bytes = Vec::new();
        packet.write(&mut bytes).expect("packet should encode");
        assert_eq!(bytes, [1, 0, 0, 0, 0, 0, 0, 0]);
    }
}
