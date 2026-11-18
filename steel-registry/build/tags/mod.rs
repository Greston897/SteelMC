//! Build-time generation for vanilla registry tags.

mod block;
mod common;
mod fluid;
mod item;

use proc_macro2::TokenStream;

pub(crate) use block::build as block;
pub(crate) use fluid::build as fluid;
pub(crate) use item::build as item;

macro_rules! simple_tag_builder {
    ($function:ident, $tag_subdir:literal, $registry_module:literal, $registry_type:literal) => {
        pub(crate) fn $function() -> TokenStream {
            common::build_simple_tags($tag_subdir, $registry_module, $registry_type)
        }
    };
    (
        $function:ident,
        $tag_subdir:literal,
        $registry_module:literal,
        $registry_type:literal,
        $tag_struct:literal,
        $register_fn:literal
    ) => {
        pub(crate) fn $function() -> TokenStream {
            common::build_simple_tags_named(
                $tag_subdir,
                $registry_module,
                $registry_type,
                $tag_struct,
                $register_fn,
            )
        }
    };
}

simple_tag_builder!(
    banner_pattern,
    "banner_pattern",
    "banner_pattern",
    "BannerPatternRegistry"
);
simple_tag_builder!(biome, "worldgen/biome", "biome", "BiomeRegistry");
simple_tag_builder!(
    damage_type,
    "damage_type",
    "damage_type",
    "DamageTypeRegistry"
);
simple_tag_builder!(dialog, "dialog", "dialog", "DialogRegistry");
simple_tag_builder!(
    enchantment,
    "enchantment",
    "enchantment",
    "EnchantmentRegistry"
);
simple_tag_builder!(
    entity_type,
    "entity_type",
    "entity_type",
    "EntityTypeRegistry"
);
simple_tag_builder!(instrument, "instrument", "instrument", "InstrumentRegistry");
simple_tag_builder!(
    painting_variant,
    "painting_variant",
    "painting_variant",
    "PaintingVariantRegistry"
);
simple_tag_builder!(poi_type, "point_of_interest_type", "poi", "PoiTypeRegistry");
simple_tag_builder!(potion, "potion", "potion", "PotionRegistry");
simple_tag_builder!(
    structure,
    "worldgen/structure",
    "structure",
    "StructureRegistry"
);
simple_tag_builder!(timeline, "timeline", "timeline", "TimelineRegistry");
simple_tag_builder!(
    game_event,
    "game_event",
    "game_events",
    "GameEventRegistry",
    "GameEventTag",
    "register_game_event_tags"
);
simple_tag_builder!(
    villager_trade,
    "villager_trade",
    "villager_trade",
    "VillagerTradeRegistry"
);
simple_tag_builder!(
    flat_level_generator_preset,
    "worldgen/flat_level_generator_preset",
    "world_preset",
    "FlatLevelGeneratorPresetRegistry",
    "FlatLevelGeneratorPresetTag",
    "register_flat_level_generator_preset_tags"
);
simple_tag_builder!(
    world_preset,
    "worldgen/world_preset",
    "world_preset",
    "WorldPresetRegistry"
);
simple_tag_builder!(
    feature,
    "worldgen/feature",
    "feature",
    "ConfiguredFeatureRegistry"
);
