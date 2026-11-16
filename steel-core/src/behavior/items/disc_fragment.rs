//! Music disc fragment (`DiscFragmentItem`).
//!
//! A 26.3 disc fragment has no server-side mechanics: it exists only so the
//! tooltip can show a grey "combine nine of these"-style description line, which
//! the vanilla client builds from `<item translation key>.desc`. Registering the
//! class keeps the item out of [`DefaultItemBehavior`] and gives the future
//! crafting/advancement code a single place to ask for that key.
//!
//! [`DefaultItemBehavior`]: crate::behavior::DefaultItemBehavior

use steel_macros::item_behavior;
use steel_registry::item_stack::ItemStack;

use crate::behavior::ItemBehavior;

/// Vanilla `DiscFragmentItem`.
#[item_behavior]
pub struct DiscFragmentItem;

impl DiscFragmentItem {
    /// Vanilla `DiscFragmentItem#getDescription` translation key suffix.
    pub const DESCRIPTION_SUFFIX: &str = ".desc";

    /// Vanilla `DiscFragmentItem#getDescription`: the item's translation key
    /// plus `.desc`.
    #[must_use]
    pub fn description_translation_key(stack: &ItemStack) -> String {
        let key = &stack.item().key;
        format!(
            "item.{}.{}{}",
            key.namespace,
            key.path,
            Self::DESCRIPTION_SUFFIX
        )
    }
}

impl ItemBehavior for DiscFragmentItem {}

#[cfg(test)]
mod tests {
    use steel_registry::item_stack::ItemStack;
    use steel_registry::{init_vanilla_registry, vanilla_items};

    use super::DiscFragmentItem;

    #[test]
    fn the_description_key_is_the_item_key_plus_desc() {
        init_vanilla_registry();
        let stack = ItemStack::new(&vanilla_items::DISC_FRAGMENT_5);

        assert_eq!(
            DiscFragmentItem::description_translation_key(&stack),
            "item.minecraft.disc_fragment_5.desc"
        );
        assert_eq!(
            DiscFragmentItem::description_translation_key(&stack)
                .split('.')
                .next_back(),
            Some("desc")
        );
    }
}
