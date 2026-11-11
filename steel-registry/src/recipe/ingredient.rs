//! Ingredient matching for recipes.

use steel_utils::Identifier;

use crate::{REGISTRY, TaggedRegistryExt, item_stack::ItemStack, items::ItemRef};

/// Represents what items can satisfy a recipe slot.
/// Matches Java's `Ingredient` class.
#[derive(Debug, Clone)]
pub enum Ingredient {
    /// Matches nothing (empty slot required).
    Empty,
    /// Matches a single specific item.
    Item(ItemRef),
    /// Matches any item in a tag.
    Tag(Identifier),
    /// Matches any of the listed items (OR).
    Choice(&'static [ItemRef]),
}

impl Ingredient {
    /// Tests if the given item stack satisfies this ingredient.
    #[must_use]
    pub fn test(&self, stack: &ItemStack) -> bool {
        match self {
            Self::Empty => stack.is_empty(),
            Self::Item(item) => !stack.is_empty() && *item == stack.item,
            Self::Tag(tag) => {
                if stack.is_empty() {
                    return false;
                }
                REGISTRY.items.is_in_tag(stack.item, tag)
            }
            Self::Choice(items) => {
                if stack.is_empty() {
                    return false;
                }
                items.contains(&stack.item)
            }
        }
    }

    /// Returns true if this ingredient matches an empty slot.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Returns all items that could satisfy this ingredient (for recipe book display).
    #[must_use]
    pub fn get_items(&self) -> Vec<ItemRef> {
        match self {
            Self::Empty => Vec::new(),
            Self::Item(item) => vec![*item],
            Self::Tag(tag) => REGISTRY.items.get_tag(tag).unwrap_or_default(),
            Self::Choice(items) => items.to_vec(),
        }
    }

    /// Compares two ingredients for equality (used for symmetry detection).
    #[must_use]
    pub fn eq_ingredient(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Empty, Self::Empty) => true,
            (Self::Item(a), Self::Item(b)) => a == b,
            (Self::Tag(a), Self::Tag(b)) => a == b,
            (Self::Choice(a), Self::Choice(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x == y)
            }
            _ => false,
        }
    }
}

/// Looks up an item's registry ID, failing instead of panicking before startup.
fn item_id(item: ItemRef) -> std::io::Result<i32> {
    let Some(id) = crate::RegistryEntry::try_id(item) else {
        return Err(std::io::Error::other(
            "item used in a recipe ingredient is not registered yet",
        ));
    };
    i32::try_from(id)
        .map_err(|_| std::io::Error::other("item ID does not fit in a VarInt"))
}

impl steel_utils::serial::WriteTo for Ingredient {
    /// Encodes the ingredient as a `HolderSet<Item>`.
    ///
    /// Vanilla uses zero to refer to a tag by name and `count + 1` for an inline
    /// item list, which is why an empty or single-item ingredient still has to
    /// write the item count.
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        use steel_utils::codec::VarInt;
        match self {
            Self::Empty => VarInt(0).write(writer),
            Self::Tag(tag) => {
                VarInt(0).write(writer)?;
                tag.write(writer)
            }
            Self::Item(item) => {
                VarInt(2).write(writer)?;
                VarInt(item_id(item)?).write(writer)
            }
            Self::Choice(items) => {
                let len = i32::try_from(items.len())
                    .map_err(|_| std::io::Error::other("ingredient is too large"))?;
                VarInt(
                    len.checked_add(1)
                        .ok_or_else(|| std::io::Error::other("ingredient is too large"))?,
                )
                .write(writer)?;
                for item in *items {
                    VarInt(item_id(item)?).write(writer)?;
                }
                Ok(())
            }
        }
    }
}
