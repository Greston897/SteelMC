//! Vanilla MC 26.3 `minecraft:environment_attribute` static registry (`51/51`).
//!
//! Mirrors `net.minecraft.world.attribute.EnvironmentAttributes`. Centralizes
//! which environment attributes carry `.syncable()` so `DimensionType`,
//! `Biome`, and `Timeline` network codecs (`NETWORK_CODEC`) emit strictly
//! syncable attributes and never crash a vanilla client with server-only
//! gameplay attributes.

use rustc_hash::FxHashMap;
use steel_utils::Identifier;

/// Top-level namespace bucket of an environment attribute path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnvironmentAttributeCategory {
    Audio,
    Gameplay,
    Visual,
}

/// Static definition of a `minecraft:environment_attribute` entry.
#[derive(Debug)]
pub struct EnvironmentAttribute {
    pub key: Identifier,
    pub category: EnvironmentAttributeCategory,
    /// Vanilla `EnvironmentAttribute.isSyncable()` — whether this attribute may
    /// appear inside `EnvironmentAttributeMap.NETWORK_CODEC` or
    /// `Timeline.NETWORK_CODEC`.
    pub syncable: bool,
    /// Vanilla `EnvironmentAttribute.isPositional()` — false only for global
    /// dimension attributes like `gameplay/sky_light_level` and `gameplay/fast_lava`.
    pub positional: bool,
    /// Vanilla `.spatiallyInterpolated()` — biome-blended across 4x4x4 cells.
    pub spatially_interpolated: bool,
}

pub type EnvironmentAttributeRef = &'static EnvironmentAttribute;

pub struct EnvironmentAttributeRegistry {
    attributes_by_id: Vec<EnvironmentAttributeRef>,
    attributes_by_key: FxHashMap<Identifier, usize>,
    allows_registering: bool,
}

impl EnvironmentAttributeRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            attributes_by_id: Vec::new(),
            attributes_by_key: FxHashMap::default(),
            allows_registering: true,
        }
    }
}

crate::impl_standard_methods!(
    EnvironmentAttributeRegistry,
    EnvironmentAttributeRef,
    attributes_by_id,
    attributes_by_key,
    allows_registering
);

crate::impl_registry!(
    EnvironmentAttributeRegistry,
    EnvironmentAttribute,
    attributes_by_id,
    attributes_by_key,
    environment_attributes
);

const fn def(
    path: &'static str,
    category: EnvironmentAttributeCategory,
    syncable: bool,
    positional: bool,
    spatially_interpolated: bool,
) -> EnvironmentAttribute {
    EnvironmentAttribute {
        key: Identifier::vanilla_static(path),
        category,
        syncable,
        positional,
        spatially_interpolated,
    }
}

use EnvironmentAttributeCategory::{Audio, Gameplay, Visual};

/// All 51 vanilla MC 26.3 environment attributes in exact
/// `EnvironmentAttributes.java` registration order.
pub static VANILLA_ENVIRONMENT_ATTRIBUTES: [EnvironmentAttribute; 51] = [
    // Visual (24 — all syncable)
    def("visual/fog_color", Visual, true, true, true),
    def("visual/fog_start_distance", Visual, true, true, true),
    def("visual/fog_end_distance", Visual, true, true, true),
    def("visual/sky_fog_end_distance", Visual, true, true, true),
    def("visual/cloud_fog_end_distance", Visual, true, true, true),
    def("visual/water_fog_color", Visual, true, true, true),
    def("visual/water_fog_start_distance", Visual, true, true, true),
    def("visual/water_fog_end_distance", Visual, true, true, true),
    def("visual/sky_color", Visual, true, true, true),
    def("visual/sunrise_sunset_color", Visual, true, true, true),
    def("visual/cloud_color", Visual, true, true, true),
    def("visual/cloud_height", Visual, true, true, true),
    def("visual/sun_angle", Visual, true, true, true),
    def("visual/moon_angle", Visual, true, true, true),
    def("visual/star_angle", Visual, true, true, true),
    def("visual/moon_phase", Visual, true, true, false),
    def("visual/star_brightness", Visual, true, true, true),
    def("visual/block_light_tint", Visual, true, true, true),
    def("visual/sky_light_color", Visual, true, true, true),
    def("visual/sky_light_factor", Visual, true, true, true),
    def("visual/night_vision_color", Visual, true, true, true),
    def("visual/ambient_light_color", Visual, true, true, true),
    def(
        "visual/default_dripstone_particle",
        Visual,
        true,
        true,
        false,
    ),
    def("visual/ambient_particles", Visual, true, true, false),
    // Audio (4 — all syncable)
    def("audio/background_music", Audio, true, true, false),
    def("audio/music_volume", Audio, true, true, false),
    def("audio/ambient_sounds", Audio, true, true, false),
    def("audio/firefly_bush_sounds", Audio, true, true, false),
    // Gameplay (23 — 5 syncable, 18 server-only)
    def("gameplay/sky_light_level", Gameplay, true, false, false),
    def("gameplay/can_start_raid", Gameplay, false, true, false),
    def("gameplay/water_evaporates", Gameplay, true, true, false),
    def("gameplay/bed_rule", Gameplay, false, true, false),
    def("gameplay/straw_bed_rule", Gameplay, false, true, false),
    def(
        "gameplay/respawn_anchor_works",
        Gameplay,
        false,
        true,
        false,
    ),
    def(
        "gameplay/nether_portal_spawns_piglin",
        Gameplay,
        false,
        true,
        false,
    ),
    def("gameplay/fast_lava", Gameplay, true, false, false),
    def(
        "gameplay/increased_fire_burnout",
        Gameplay,
        false,
        true,
        false,
    ),
    def("gameplay/eyeblossom_open", Gameplay, false, true, false),
    def(
        "gameplay/turtle_egg_hatch_chance",
        Gameplay,
        false,
        true,
        false,
    ),
    def("gameplay/piglins_zombify", Gameplay, true, true, false),
    def("gameplay/snow_golem_melts", Gameplay, false, true, false),
    def("gameplay/creaking_active", Gameplay, true, true, false),
    def(
        "gameplay/surface_slime_spawn_chance",
        Gameplay,
        false,
        true,
        false,
    ),
    def(
        "gameplay/cat_waking_up_gift_chance",
        Gameplay,
        false,
        true,
        false,
    ),
    def("gameplay/bees_stay_in_hive", Gameplay, false, true, false),
    def("gameplay/monsters_burn", Gameplay, false, true, false),
    def(
        "gameplay/can_pillager_patrol_spawn",
        Gameplay,
        false,
        true,
        false,
    ),
    def(
        "gameplay/creature_world_gen_spawn_probability",
        Gameplay,
        false,
        true,
        false,
    ),
    def("gameplay/natural_mob_spawns", Gameplay, false, true, false),
    def("gameplay/villager_activity", Gameplay, false, true, false),
    def(
        "gameplay/baby_villager_activity",
        Gameplay,
        false,
        true,
        false,
    ),
];

/// Registers all 51 vanilla MC 26.3 environment attributes.
pub fn register_environment_attributes(registry: &mut EnvironmentAttributeRegistry) {
    for attribute in &VANILLA_ENVIRONMENT_ATTRIBUTES {
        registry.register(attribute);
    }
}

/// Returns `true` iff `key` (either `"minecraft:<path>"` or bare `"<path>"`) is
/// marked `.syncable()` in vanilla MC 26.3 `EnvironmentAttributes`.
///
/// Delegates to the canonical [`steel_utils::registry::is_syncable_environment_attribute`]
/// so build-time codegen and runtime codecs can never disagree.
#[must_use]
pub fn is_syncable_environment_attribute(key: &str) -> bool {
    steel_utils::registry::is_syncable_environment_attribute(key)
}
