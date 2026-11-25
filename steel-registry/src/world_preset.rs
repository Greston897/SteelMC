//! Vanilla MC 26.3 `minecraft:dimension` (`3/3`),
//! `minecraft:worldgen/flat_level_generator_preset` (`9/9`, tag `visible`), and
//! `minecraft:worldgen/world_preset` (`7/7`, tags `normal`, `extended`)
//! registries.

use rustc_hash::FxHashMap;
use steel_utils::Identifier;

use crate::RegistryTags;

/// A single horizontal block layer inside a superflat generator setting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatBlockLayer {
    pub block: Identifier,
    pub height: i32,
}

/// Biome source declared by a noise `LevelStem` generator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LevelStemBiomeSource {
    /// `minecraft:fixed` — a single biome everywhere.
    Fixed { biome: Identifier },
    /// `minecraft:multi_noise` — vanilla multi-noise fields, optionally from a
    /// `multi_noise_preset` (`None` means inline parameters, unused by vanilla).
    MultiNoise { preset: Option<Identifier> },
    /// `minecraft:the_end` — the End island source.
    TheEnd,
    /// `minecraft:checkerboard` — cycling `biomes` at `scale`.
    Checkerboard { biomes: Vec<Identifier>, scale: i32 },
}

/// Chunk generator kind declared by a `LevelStem` (`dimension` / `world_preset`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LevelStemGenerator {
    /// `minecraft:noise` — noise generator with settings ID and biome source.
    Noise {
        settings: Identifier,
        biome_source: LevelStemBiomeSource,
    },
    /// `minecraft:flat` — flat generator with biome and layer list.
    ///
    /// `structure_overrides` holds structure-*tag* ids; `None` (field absent)
    /// means no override filter — every structure may generate — while
    /// `Some([])` disables all structures.
    Flat {
        biome: Identifier,
        lakes: bool,
        features: bool,
        layers: Vec<FlatBlockLayer>,
        structure_overrides: Option<Vec<Identifier>>,
    },
    /// `minecraft:debug` — debug all-block-states grid generator.
    Debug,
}

/// Registered `minecraft:dimension` (`LevelStem`) entry.
#[derive(Debug, Clone)]
pub struct DimensionStem {
    pub key: Identifier,
    pub dimension_type: Identifier,
    pub generator: LevelStemGenerator,
}

pub type DimensionStemRef = &'static DimensionStem;

pub struct DimensionStemRegistry {
    stems_by_id: Vec<DimensionStemRef>,
    stems_by_key: FxHashMap<Identifier, usize>,
    allows_registering: bool,
}

impl DimensionStemRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            stems_by_id: Vec::new(),
            stems_by_key: FxHashMap::default(),
            allows_registering: true,
        }
    }
}

crate::impl_standard_methods!(
    DimensionStemRegistry,
    DimensionStemRef,
    stems_by_id,
    stems_by_key,
    allows_registering
);

crate::impl_registry!(
    DimensionStemRegistry,
    DimensionStem,
    stems_by_id,
    stems_by_key,
    dimensions
);

/// Registered `minecraft:worldgen/flat_level_generator_preset` entry.
#[derive(Debug, Clone)]
pub struct FlatLevelGeneratorPreset {
    pub key: Identifier,
    pub display_item: Identifier,
    pub biome: Identifier,
    pub lakes: bool,
    pub features: bool,
    pub layers: Vec<FlatBlockLayer>,
    /// Structure-*tag* ids overriding structure generation (`None` when the
    /// file omits the field: every structure may generate).
    pub structure_overrides: Option<Vec<Identifier>>,
}

impl FlatLevelGeneratorPreset {
    /// Total height in blocks across all configured superflat layers.
    #[must_use]
    pub fn total_layer_height(&self) -> i32 {
        self.layers.iter().map(|layer| layer.height).sum()
    }
}

pub type FlatLevelGeneratorPresetRef = &'static FlatLevelGeneratorPreset;

pub struct FlatLevelGeneratorPresetRegistry {
    presets_by_id: Vec<FlatLevelGeneratorPresetRef>,
    presets_by_key: FxHashMap<Identifier, usize>,
    tags: RegistryTags,
    allows_registering: bool,
}

impl FlatLevelGeneratorPresetRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            presets_by_id: Vec::new(),
            presets_by_key: FxHashMap::default(),
            tags: RegistryTags::default(),
            allows_registering: true,
        }
    }
}

crate::impl_standard_methods!(
    FlatLevelGeneratorPresetRegistry,
    FlatLevelGeneratorPresetRef,
    presets_by_id,
    presets_by_key,
    allows_registering
);

crate::impl_registry!(
    FlatLevelGeneratorPresetRegistry,
    FlatLevelGeneratorPreset,
    presets_by_id,
    presets_by_key,
    flat_level_generator_presets
);

crate::impl_tagged_registry!(
    FlatLevelGeneratorPresetRegistry,
    presets_by_key,
    "worldgen/flat_level_generator_preset"
);

/// Registered `minecraft:worldgen/world_preset` entry.
#[derive(Debug, Clone)]
pub struct WorldPreset {
    pub key: Identifier,
    pub dimensions: Vec<DimensionStem>,
}

impl WorldPreset {
    /// Looks up a dimension stem inside this preset (e.g. `minecraft:overworld`).
    #[must_use]
    pub fn dimension(&self, key: &Identifier) -> Option<&DimensionStem> {
        self.dimensions.iter().find(|stem| &stem.key == key)
    }
}

pub type WorldPresetRef = &'static WorldPreset;

pub struct WorldPresetRegistry {
    presets_by_id: Vec<WorldPresetRef>,
    presets_by_key: FxHashMap<Identifier, usize>,
    tags: RegistryTags,
    allows_registering: bool,
}

impl WorldPresetRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            presets_by_id: Vec::new(),
            presets_by_key: FxHashMap::default(),
            tags: RegistryTags::default(),
            allows_registering: true,
        }
    }
}

crate::impl_standard_methods!(
    WorldPresetRegistry,
    WorldPresetRef,
    presets_by_id,
    presets_by_key,
    allows_registering
);

crate::impl_registry!(
    WorldPresetRegistry,
    WorldPreset,
    presets_by_id,
    presets_by_key,
    world_presets
);

crate::impl_tagged_registry!(WorldPresetRegistry, presets_by_key, "worldgen/world_preset");
