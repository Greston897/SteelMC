//! Data components system for items and entities.
//!
//! Component values are type-erased through Steel's deterministic keyed
//! downcasting system. Vanilla and plugin-defined values use the same storage
//! path while retaining type-safe access through [`DataComponentType`].
//!
//! # Architecture
//!
//! - [`ComponentData`] - Type-erased component value storage
//! - [`Component`] - Object-safe behavior for stored values
//! - [`DataComponentType<T>`] - Compile-time type handle for accessing components
//! - [`DataComponentMap`] - Storage for component values on items
//! - [`DataComponentPatch`] - Diff representation for network/storage
//! - [`DataComponentRegistry`] - Registry of component types with serialization
//!
//! # Example
//! ```rust
//! use steel_registry::data_components::{DataComponentMap, vanilla_components::DAMAGE};
//! use steel_utils::Identifier;
//! let mut components = DataComponentMap::new();
//! assert_eq!(components.get(DAMAGE), None);
//! components.set(DAMAGE, Some(10));
//! assert_eq!(components.get(DAMAGE), Some(10));
//! assert_eq!(components.get_ref(DAMAGE), Some(&10));
//! assert!(components.get_raw(&Identifier::vanilla_static("damage")).is_some());
//! components.set(DAMAGE, None);
//! assert_eq!(components.get(DAMAGE), None);
//! ```

mod component_data;
pub mod components;
mod registry;
pub mod vanilla_components;

// Re-export core types
pub use crate::item_predicate::{AdventureModePredicate, BlockPredicate, LockCode};
pub use component_data::{Component, ComponentData};
pub use components::{
    ArmorTrim, BannerPatternLayer, BannerPatternLayers, BlocksAttacks, BundleContents,
    ChargedProjectiles, Consumable, CustomData, CustomModelData, DeathProtection, DyedItemColor,
    Enchantable, Equippable, EquippableAllowedEntities, InstrumentComponent,
    InvalidEnchantableValue, ItemContainerContents, JukeboxPlayable, MapDecorationEntry,
    MapDecorations, MapId, MapItemColor, OminousBottleAmplifier, PaintingVariantComponent,
    PotDecorations, PotionContents, ProvidesBannerPatterns, ProvidesTrimMaterial, Recipes,
    SeededContainerLoot, SulfurCubeContent, Tool, ToolRule, ToolRuleBlocks, UseRemainder,
};
pub use registry::{
    ComponentEntry,
    ComponentEntryRef,
    ComponentPatchEntry,
    DataComponentMap,
    DataComponentPatch,
    DataComponentRegistry,
    DataComponentType,
    NbtReader,
    NbtWriter,
    // Type aliases for reader/writer functions
    NetworkReader,
    NetworkWriter,
    component_try_into,
};
