//! Smithing template items (`SmithingTemplateItem`, 19 items).
//!
//! A 26.3 smithing template carries no server-side mechanics: the item only
//! supplies the tooltip lines and the empty-slot illustrations of the smithing
//! screen, both of which the vanilla client renders from its own assets. The
//! real smithing logic lives in the smithing table menu.
//!
//! Steel registers the class so the 19 templates stop falling through to
//! [`DefaultItemBehavior`] and keeps vanilla's slot-texture tables here, next to
//! the class that owns them, for the future smithing menu.
//!
//! [`DefaultItemBehavior`]: crate::behavior::DefaultItemBehavior

use steel_macros::item_behavior;

use crate::behavior::ItemBehavior;

/// Vanilla `SmithingTemplateItem#getArmorTrimEmptyBaseSlotTextures`.
pub const ARMOR_TRIM_EMPTY_BASE_SLOT_TEXTURES: [&str; 4] = [
    "minecraft:container/slot/helmet",
    "minecraft:container/slot/chestplate",
    "minecraft:container/slot/leggings",
    "minecraft:container/slot/boots",
];

/// Vanilla `SmithingTemplateItem#getArmorTrimEmptyAdditionsSlotTextures`.
pub const ARMOR_TRIM_EMPTY_ADDITIONS_SLOT_TEXTURES: [&str; 7] = [
    "minecraft:container/slot/ingot",
    "minecraft:container/slot/redstone_dust",
    "minecraft:container/slot/lapis_lazuli",
    "minecraft:container/slot/quartz",
    "minecraft:container/slot/diamond",
    "minecraft:container/slot/emerald",
    "minecraft:container/slot/amethyst_shard",
];

/// Vanilla `SmithingTemplateItem#getNetheriteUpgradeEmptyBaseSlotTextures`.
pub const NETHERITE_UPGRADE_EMPTY_BASE_SLOT_TEXTURES: [&str; 11] = [
    "minecraft:container/slot/helmet",
    "minecraft:container/slot/sword",
    "minecraft:container/slot/chestplate",
    "minecraft:container/slot/pickaxe",
    "minecraft:container/slot/leggings",
    "minecraft:container/slot/axe",
    "minecraft:container/slot/boots",
    "minecraft:container/slot/hoe",
    "minecraft:container/slot/shovel",
    "minecraft:container/slot/nautilus_armor",
    "minecraft:container/slot/spear",
];

/// Vanilla `SmithingTemplateItem#getNetheriteUpgradeEmptyAdditionsSlotTextures`.
pub const NETHERITE_UPGRADE_EMPTY_ADDITIONS_SLOT_TEXTURES: [&str; 1] =
    ["minecraft:container/slot/ingot"];

/// Vanilla `SmithingTemplateItem` (armor trims and the netherite upgrade).
#[item_behavior]
pub struct SmithingTemplateItem;

impl SmithingTemplateItem {
    /// Vanilla `SmithingTemplateItem#getBaseSlotDescription` translation key
    /// for armor trim templates.
    pub const ARMOR_TRIM_BASE_SLOT_DESCRIPTION: &str =
        "item.minecraft.smithing_template.armor_trim.base_slot_description";

    /// Vanilla base-slot description key for the netherite upgrade template.
    pub const NETHERITE_UPGRADE_BASE_SLOT_DESCRIPTION: &str =
        "item.minecraft.smithing_template.netherite_upgrade.base_slot_description";

    /// Vanilla `of(settings)`: armor trims accept any armor piece as the base.
    #[must_use]
    pub const fn armor_trim_empty_base_slot_textures() -> &'static [&'static str] {
        &ARMOR_TRIM_EMPTY_BASE_SLOT_TEXTURES
    }

    /// Vanilla `createNetheriteUpgrade(settings)`: netherite accepts the
    /// equipment *and* tool list.
    #[must_use]
    pub const fn netherite_upgrade_empty_base_slot_textures() -> &'static [&'static str] {
        &NETHERITE_UPGRADE_EMPTY_BASE_SLOT_TEXTURES
    }

    /// Vanilla `getEmptyAdditionsSlotTextures` for armor trims.
    #[must_use]
    pub const fn armor_trim_empty_additions_slot_textures() -> &'static [&'static str] {
        &ARMOR_TRIM_EMPTY_ADDITIONS_SLOT_TEXTURES
    }

    /// Vanilla `getEmptyAdditionsSlotTextures` for the netherite upgrade.
    #[must_use]
    pub const fn netherite_upgrade_empty_additions_slot_textures() -> &'static [&'static str] {
        &NETHERITE_UPGRADE_EMPTY_ADDITIONS_SLOT_TEXTURES
    }
}

impl ItemBehavior for SmithingTemplateItem {}

#[cfg(test)]
mod tests {
    use steel_registry::item_stack::ItemStack;
    use steel_registry::{init_vanilla_registry, vanilla_items};

    use super::{
        ARMOR_TRIM_EMPTY_ADDITIONS_SLOT_TEXTURES, ARMOR_TRIM_EMPTY_BASE_SLOT_TEXTURES,
        NETHERITE_UPGRADE_EMPTY_ADDITIONS_SLOT_TEXTURES,
        NETHERITE_UPGRADE_EMPTY_BASE_SLOT_TEXTURES, SmithingTemplateItem,
    };

    #[test]
    fn armor_trims_expose_the_vanilla_slot_lists() {
        assert_eq!(
            SmithingTemplateItem::armor_trim_empty_base_slot_textures().len(),
            4,
            "helmet, chestplate, leggings and boots"
        );
        assert_eq!(
            SmithingTemplateItem::armor_trim_empty_additions_slot_textures().len(),
            7,
            "vanilla lists seven trim materials as slot hints"
        );
        assert_eq!(
            ARMOR_TRIM_EMPTY_BASE_SLOT_TEXTURES[0],
            "minecraft:container/slot/helmet"
        );
        assert!(
            ARMOR_TRIM_EMPTY_ADDITIONS_SLOT_TEXTURES.contains(&"minecraft:container/slot/ingot")
        );
    }

    #[test]
    fn the_netherite_upgrade_lists_equipment_and_tools() {
        assert_eq!(
            SmithingTemplateItem::netherite_upgrade_empty_base_slot_textures().len(),
            11
        );
        let additions = SmithingTemplateItem::netherite_upgrade_empty_additions_slot_textures();
        assert_eq!(additions.len(), 1);
        assert_eq!(additions[0], "minecraft:container/slot/ingot");
        assert!(
            NETHERITE_UPGRADE_EMPTY_BASE_SLOT_TEXTURES.contains(&"minecraft:container/slot/spear")
        );
        assert_eq!(NETHERITE_UPGRADE_EMPTY_ADDITIONS_SLOT_TEXTURES.len(), 1);
    }

    /// Translation key of the shared "smithing template" tooltip header.
    const SMITHING_TEMPLATE_TOOLTIP_KEY: &str = "item.minecraft.smithing_template";

    /// Translation key of the "applies to" tooltip header.
    const APPLIES_TO_TOOLTIP_KEY: &str = "item.minecraft.smithing_template.applies_to";

    /// Translation key of the "ingredients" tooltip header.
    const INGREDIENTS_TOOLTIP_KEY: &str = "item.minecraft.smithing_template.ingredients";

    #[test]
    fn tooltip_keys_match_the_vanilla_translation_ids() {
        assert_eq!(
            SMITHING_TEMPLATE_TOOLTIP_KEY,
            "item.minecraft.smithing_template"
        );
        assert_eq!(
            INGREDIENTS_TOOLTIP_KEY,
            "item.minecraft.smithing_template.ingredients"
        );
        assert_eq!(
            APPLIES_TO_TOOLTIP_KEY,
            "item.minecraft.smithing_template.applies_to"
        );
        assert_eq!(
            SmithingTemplateItem::ARMOR_TRIM_BASE_SLOT_DESCRIPTION,
            "item.minecraft.smithing_template.armor_trim.base_slot_description"
        );
    }

    #[test]
    fn every_template_item_resolves_to_this_class() {
        init_vanilla_registry();
        // The 19 template items share one class, so the registry must accept all
        // of them as plain stacks with the vanilla name component.
        for item in [
            &vanilla_items::NETHERITE_UPGRADE_SMITHING_TEMPLATE,
            &vanilla_items::SENTRY_ARMOR_TRIM_SMITHING_TEMPLATE,
            &vanilla_items::SHAPER_ARMOR_TRIM_SMITHING_TEMPLATE,
        ] {
            let stack = ItemStack::new(item);
            assert!(!stack.is_empty());
        }
    }
}
