//! Slot display payloads used by the recipe book and `update_recipes`.
//!
//! Vanilla sends recipes as *displays*: a [`RecipeDisplay`] describes the grid and
//! every slot is a [`SlotDisplay`] describing what may go there. The client picks
//! a concrete item out of a tag or a choice, which is why the wire format keeps
//! the ingredient shape instead of flattening it to one stack.
//!
//! Both types are written inline into `update_recipes`, so their writers are
//! used directly by the packet.

use std::io::{Error, Result, Write};

use steel_utils::Identifier;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;

use crate::item_stack_template::ItemStackTemplate;
use crate::items::Item;

/// Identifier of the `minecraft:slot_display` registry, in registry order.
///
/// Vanilla registers the payloads in this order, so the discriminant is the
/// numeric ID written on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum SlotDisplayKind {
    /// Matches nothing.
    Empty = 0,
    /// Any furnace fuel.
    AnyFuel = 1,
    /// A base display restricted to potion carriers.
    WithAnyPotion = 2,
    /// A base display restricted to stacks carrying a component.
    OnlyWithComponent = 3,
    /// A single item.
    Item = 4,
    /// A full item stack.
    ItemStack = 5,
    /// Any item in an item tag.
    Tag = 6,
    /// A dye applied to a target.
    Dyed = 7,
    /// A smithing trim template.
    SmithingTrim = 8,
    /// A display plus the crafting remainder of its item.
    WithRemainder = 9,
    /// A list of alternative displays.
    Composite = 10,
}

impl SlotDisplayKind {
    #[must_use]
    const fn id(self) -> i32 {
        self as i32
    }
}

/// One slot of a recipe display.
#[derive(Debug, Clone)]
pub enum SlotDisplay {
    /// Matches nothing.
    Empty,
    /// Any furnace fuel.
    AnyFuel,
    /// A single item, without count or components.
    Item(&'static Item),
    /// A full stack, including count and components.
    ItemStack(ItemStackTemplate),
    /// Any item in the tag.
    Tag(Identifier),
    /// A dye applied to a target.
    Dyed {
        /// The dye slot.
        dye: Box<SlotDisplay>,
        /// The item being dyed.
        target: Box<SlotDisplay>,
    },
    /// A display plus the crafting remainder of its item.
    WithRemainder(Box<SlotDisplay>),
    /// Alternatives, tried in order by the client.
    Composite(Vec<SlotDisplay>),
}

impl SlotDisplay {
    /// Builds the display vanilla uses for a recipe ingredient.
    ///
    /// A single item becomes `minecraft:item`, a tag becomes `minecraft:tag` and
    /// a multi-item choice becomes `minecraft:composite`. This mirrors
    /// `Ingredient.display()`.
    #[must_use]
    pub fn from_ingredient(ingredient: &crate::recipe::Ingredient) -> Self {
        match ingredient {
            crate::recipe::Ingredient::Empty => Self::Empty,
            crate::recipe::Ingredient::Item(item) => Self::Item(item),
            crate::recipe::Ingredient::Tag(tag) => Self::Tag(tag.clone()),
            crate::recipe::Ingredient::Choice(items) => {
                Self::Composite(items.iter().map(|item| Self::Item(item)).collect())
            }
        }
    }
}

impl WriteTo for SlotDisplay {
    fn write(&self, writer: &mut impl Write) -> Result<()> {
        match self {
            Self::Empty => write_kind(writer, SlotDisplayKind::Empty),
            Self::AnyFuel => write_kind(writer, SlotDisplayKind::AnyFuel),
            Self::Item(item) => {
                write_kind(writer, SlotDisplayKind::Item)?;
                write_item_id(writer, item)
            }
            Self::ItemStack(stack) => {
                write_kind(writer, SlotDisplayKind::ItemStack)?;
                stack.write(writer)
            }
            Self::Tag(tag) => {
                write_kind(writer, SlotDisplayKind::Tag)?;
                tag.write(writer)
            }
            Self::Dyed { dye, target } => {
                write_kind(writer, SlotDisplayKind::Dyed)?;
                dye.write(writer)?;
                target.write(writer)
            }
            Self::WithRemainder(display) => {
                write_kind(writer, SlotDisplayKind::WithRemainder)?;
                display.write(writer)
            }
            Self::Composite(displays) => {
                write_kind(writer, SlotDisplayKind::Composite)?;
                write_var_int(writer, displays.len())?;
                for display in displays {
                    display.write(writer)?;
                }
                Ok(())
            }
        }
    }
}

/// One recipe as the recipe book renders it.
#[derive(Debug, Clone)]
pub enum RecipeDisplay {
    /// A shapeless crafting recipe.
    CraftingShapeless {
        /// One display per ingredient.
        ingredients: Vec<SlotDisplay>,
        /// The crafted result.
        result: SlotDisplay,
        /// The station icon shown in the notification.
        crafting_station: SlotDisplay,
    },
    /// A shaped crafting recipe, row-major.
    CraftingShaped {
        /// Grid width, one to three.
        width: i32,
        /// Grid height, one to three.
        height: i32,
        /// `width * height` displays, row-major.
        ingredients: Vec<SlotDisplay>,
        /// The crafted result.
        result: SlotDisplay,
        /// The station icon shown in the notification.
        crafting_station: SlotDisplay,
    },
    /// A furnace-family recipe.
    Furnace {
        /// The smelted input.
        ingredient: SlotDisplay,
        /// The fuel slot, usually `minecraft:any_fuel`.
        fuel: SlotDisplay,
        /// The smelted result.
        result: SlotDisplay,
        /// The station icon shown in the notification.
        crafting_station: SlotDisplay,
        /// Cooking time in ticks.
        cooking_time: i32,
        /// Experience granted per item.
        experience: f32,
    },
    /// A stonecutter recipe.
    Stonecutter {
        /// The cut input.
        ingredient: SlotDisplay,
        /// The cut result.
        result: SlotDisplay,
        /// The station icon shown in the notification.
        crafting_station: SlotDisplay,
    },
    /// A smithing recipe.
    Smithing {
        /// The template slot.
        template: SlotDisplay,
        /// The base item.
        base: SlotDisplay,
        /// The addition slot.
        addition: SlotDisplay,
        /// The smithed result.
        result: SlotDisplay,
        /// The station icon shown in the notification.
        crafting_station: SlotDisplay,
    },
}

impl RecipeDisplay {
    /// Vanilla's station icon for the crafting-table family.
    #[must_use]
    pub fn crafting_table_station() -> SlotDisplay {
        SlotDisplay::Item(&crate::vanilla_items::CRAFTING_TABLE)
    }
}

impl WriteTo for RecipeDisplay {
    fn write(&self, writer: &mut impl Write) -> Result<()> {
        match self {
            Self::CraftingShapeless {
                ingredients,
                result,
                crafting_station,
            } => {
                write_var_int(writer, 0)?;
                write_var_int(writer, ingredients.len())?;
                for ingredient in ingredients {
                    ingredient.write(writer)?;
                }
                result.write(writer)?;
                crafting_station.write(writer)
            }
            Self::CraftingShaped {
                width,
                height,
                ingredients,
                result,
                crafting_station,
            } => {
                write_var_int(writer, 1)?;
                write_var_int(writer, usize::try_from(*width).unwrap_or_default())?;
                write_var_int(writer, usize::try_from(*height).unwrap_or_default())?;
                write_var_int(writer, ingredients.len())?;
                for ingredient in ingredients {
                    ingredient.write(writer)?;
                }
                result.write(writer)?;
                crafting_station.write(writer)
            }
            Self::Furnace {
                ingredient,
                fuel,
                result,
                crafting_station,
                cooking_time,
                experience,
            } => {
                write_var_int(writer, 2)?;
                ingredient.write(writer)?;
                fuel.write(writer)?;
                result.write(writer)?;
                crafting_station.write(writer)?;
                write_var_int(writer, usize::try_from(*cooking_time).unwrap_or_default())?;
                writer.write_all(&experience.to_be_bytes())
            }
            Self::Stonecutter {
                ingredient,
                result,
                crafting_station,
            } => {
                write_var_int(writer, 3)?;
                ingredient.write(writer)?;
                result.write(writer)?;
                crafting_station.write(writer)
            }
            Self::Smithing {
                template,
                base,
                addition,
                result,
                crafting_station,
            } => {
                write_var_int(writer, 4)?;
                template.write(writer)?;
                base.write(writer)?;
                addition.write(writer)?;
                result.write(writer)?;
                crafting_station.write(writer)
            }
        }
    }
}

fn write_kind(writer: &mut impl Write, kind: SlotDisplayKind) -> Result<()> {
    write_var_int(writer, usize::try_from(kind.id()).unwrap_or_default())
}

fn write_var_int(writer: &mut impl Write, value: usize) -> Result<()> {
    let value = i32::try_from(value)
        .map_err(|_| Error::other("recipe display value does not fit in a VarInt"))?;
    VarInt(value).write(writer)
}

/// Writes an item registry ID as a `VarInt`.
fn write_item_id(writer: &mut impl Write, item: &'static Item) -> Result<()> {
    let Some(id) = crate::RegistryEntry::try_id(item) else {
        return Err(Error::other(
            "item used in a slot display is not registered yet",
        ));
    };
    let id = i32::try_from(id).map_err(|_| Error::other("item ID does not fit in a VarInt"))?;
    VarInt(id).write(writer)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use steel_utils::codec::VarInt;
    use steel_utils::serial::ReadFrom as _;
    use steel_utils::serial::WriteTo as _;

    use super::{RecipeDisplay, SlotDisplay};
    use crate::init_vanilla_registry;
    use crate::vanilla_items;

    #[test]
    fn empty_and_any_fuel_displays_write_only_their_kind() {
        let mut bytes = Vec::new();
        SlotDisplay::Empty
            .write(&mut bytes)
            .expect("empty display should encode");
        assert_eq!(bytes, [0]);

        let mut bytes = Vec::new();
        SlotDisplay::AnyFuel
            .write(&mut bytes)
            .expect("fuel display should encode");
        assert_eq!(bytes, [1]);
    }

    #[test]
    fn item_display_writes_the_registry_id() {
        init_vanilla_registry();
        let mut bytes = Vec::new();
        SlotDisplay::Item(&vanilla_items::DIAMOND)
            .write(&mut bytes)
            .expect("item display should encode");
        assert_eq!(bytes[0], 4, "minecraft:item is slot display kind 4");
        let mut cursor = Cursor::new(&bytes[1..]);
        let id = VarInt::read(&mut cursor).expect("id should decode");
        assert_eq!(
            usize::try_from(id.0).expect("id is non-negative"),
            crate::RegistryEntry::id(&*vanilla_items::DIAMOND),
        );
    }

    #[test]
    fn composite_display_writes_its_length() {
        init_vanilla_registry();
        let display = SlotDisplay::Composite(vec![
            SlotDisplay::Item(&vanilla_items::DIAMOND),
            SlotDisplay::Item(&vanilla_items::EMERALD),
        ]);
        let mut bytes = Vec::new();
        display.write(&mut bytes).expect("composite should encode");
        assert_eq!(bytes[0], 10, "minecraft:composite is slot display kind 10");
        assert_eq!(bytes[1], 2, "length is written as a VarInt");
    }

    #[test]
    fn shaped_display_writes_kind_width_height_and_count() {
        init_vanilla_registry();
        let display = RecipeDisplay::CraftingShaped {
            width: 2,
            height: 2,
            ingredients: vec![
                SlotDisplay::Item(&vanilla_items::OAK_PLANKS),
                SlotDisplay::Item(&vanilla_items::OAK_PLANKS),
                SlotDisplay::Item(&vanilla_items::OAK_PLANKS),
                SlotDisplay::Item(&vanilla_items::OAK_PLANKS),
            ],
            result: SlotDisplay::Item(&vanilla_items::CRAFTING_TABLE),
            crafting_station: RecipeDisplay::crafting_table_station(),
        };
        let mut bytes = Vec::new();
        display.write(&mut bytes).expect("shaped display should encode");
        assert_eq!(&bytes[..4], [1, 2, 2, 4], "kind, width, height, count");
    }
}
