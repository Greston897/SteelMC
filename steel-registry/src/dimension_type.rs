use rustc_hash::FxHashMap;
use simdnbt::ToNbtTag;
use simdnbt::owned::NbtTag;
use steel_utils::Identifier;
use text_components::TextComponent;

use crate::sound_event::SoundEventRef;
use crate::world_clock::WorldClockRef;

#[derive(Debug)]
pub struct BedRule {
    pub can_set_spawn: BedRuleValue,
    pub can_sleep: BedRuleValue,
    pub destroy_on_use: bool,
    pub destroy_on_leave: bool,
    pub error_message_key: Option<&'static str>,
    pub error_message: Option<fn() -> TextComponent>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BedRuleValue {
    Always,
    WhenDark,
    Never,
}

impl BedRuleValue {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::WhenDark => "when_dark",
            Self::Never => "never",
        }
    }
}

impl ToNbtTag for BedRuleValue {
    fn to_nbt_tag(self) -> NbtTag {
        self.as_str().to_nbt_tag()
    }
}

#[derive(Debug)]
pub struct MoodSound {
    pub sound: SoundEventRef,
    pub tick_delay: i32,
    pub block_search_extent: i32,
    pub offset: f64,
}

#[derive(Debug)]
pub struct MusicEntry {
    pub sound: SoundEventRef,
    pub min_delay: i32,
    pub max_delay: i32,
    pub replace_current_music: bool,
}

#[derive(Debug)]
pub struct BackgroundMusic {
    pub default: MusicEntry,
    pub creative: Option<MusicEntry>,
}

/// Represents a full dimension type definition from a data pack JSON file.
#[derive(Debug)]
pub struct DimensionType {
    pub key: Identifier,
    /// Vanilla MC 26.3 `has_fixed_time` — the only fixed-time state in
    /// `DimensionType.createDirectCodec`; 26.3 has no `fixed_time` value.
    pub has_fixed_time: bool,
    pub has_skylight: bool,
    pub has_ceiling: bool,
    pub coordinate_scale: f64,
    pub min_y: i32,
    pub height: i32,
    pub logical_height: i32,
    pub infiniburn: &'static str,
    pub ambient_light: f32,
    pub default_clock: Option<WorldClockRef>,
    pub timelines: Option<&'static str>,
    pub has_ender_dragon_fight: bool,
    pub monster_spawn_light_level: MonsterSpawnLightLevel,
    pub monster_spawn_block_light_limit: i32,

    // Top-level
    pub skybox: Option<&'static str>,
    pub cardinal_light: Option<&'static str>,

    // Attributes: visual
    pub sky_color: Option<&'static str>,
    pub fog_color: Option<&'static str>,
    pub cloud_color: Option<&'static str>,
    pub cloud_height: Option<f32>,
    pub ambient_light_color: Option<&'static str>,
    pub sky_light_color: Option<&'static str>,
    pub sky_light_factor: Option<f32>,
    pub fog_start_distance: Option<f32>,
    pub fog_end_distance: Option<f32>,
    pub default_dripstone_particle: Option<&'static str>,

    // Attributes: gameplay. `None` means the dimension JSON omits the entry
    // from its `attributes` map, so the vanilla attribute default applies and
    // neither codec emits the key (DFU encodes only present map entries).
    pub respawn_anchor_works: Option<bool>,
    pub can_start_raid: Option<bool>,
    pub fast_lava: Option<bool>,
    pub piglins_zombify: Option<bool>,
    pub sky_light_level: Option<f32>,
    pub snow_golem_melts: Option<bool>,
    pub water_evaporates: Option<bool>,
    pub nether_portal_spawns_piglin: Option<bool>,
    pub bed_rule: BedRule,
    pub straw_bed_rule: BedRule,

    // Attributes: audio
    pub mood_sound: Option<MoodSound>,
    pub background_music: Option<BackgroundMusic>,
}

impl DimensionType {
    /// Returns a boolean `minecraft:gameplay/...` environment attribute as the
    /// dimension type declares it, mirroring vanilla `EnvironmentAttributeMap`.
    ///
    /// Unknown attributes and attributes the dimension leaves at their default are
    /// reported as `None`, so callers fall back to the vanilla default.
    #[must_use]
    pub const fn gameplay_attribute_bool(&self, key: &str) -> Option<bool> {
        match key.as_bytes() {
            b"minecraft:gameplay/nether_portal_spawns_piglin" => self.nether_portal_spawns_piglin,
            b"minecraft:gameplay/piglins_zombify" => self.piglins_zombify,
            b"minecraft:gameplay/can_start_raid" => self.can_start_raid,
            b"minecraft:gameplay/respawn_anchor_works" => self.respawn_anchor_works,
            b"minecraft:gameplay/snow_golem_melts" => self.snow_golem_melts,
            b"minecraft:gameplay/water_evaporates" => self.water_evaporates,
            b"minecraft:gameplay/fast_lava" => self.fast_lava,
            _ => None,
        }
    }

    #[must_use]
    pub const fn get_teleportation_scale(
        last_dimension_type: &Self,
        new_dimension_type: &Self,
    ) -> f64 {
        last_dimension_type.coordinate_scale / new_dimension_type.coordinate_scale
    }
}

/// Represents the complex structure for monster spawn light level.
#[derive(Debug)]
pub enum MonsterSpawnLightLevel {
    Simple(i32),
    Complex {
        distribution_type: &'static str,
        min_inclusive: i32,
        max_inclusive: i32,
    },
}

impl DimensionType {
    /// Vanilla MC 26.3 `DimensionType.NETWORK_CODEC`: emits `has_fixed_time`
    /// (26.3 has no `fixed_time` value) and strictly `.syncable()`
    /// environment attributes. Server-only gameplay attributes
    /// (`can_start_raid`, `bed_rule`, `straw_bed_rule`,
    /// `respawn_anchor_works`, `nether_portal_spawns_piglin`,
    /// `snow_golem_melts`, ...) are withheld — a vanilla client rejects them.
    #[must_use]
    pub fn to_network_nbt(&self) -> NbtTag {
        use simdnbt::owned::{NbtCompound, NbtTag};
        let mut compound = NbtCompound::new();

        // Exact `DimensionType.createDirectCodec` field order:
        // has_fixed_time?, has_skylight, has_ceiling,
        // has_ender_dragon_fight, coordinate_scale, min_y, height,
        // logical_height, infiniburn, ambient_light,
        // monster_spawn_light_level, monster_spawn_block_light_limit,
        // skybox?, cardinal_light?, attributes?, timelines?, default_clock?
        // DFU `optionalFieldOf` omits defaulted values on encode.
        if self.has_fixed_time {
            compound.insert("has_fixed_time", true);
        }
        compound.insert("has_skylight", self.has_skylight);
        compound.insert("has_ceiling", self.has_ceiling);
        compound.insert("has_ender_dragon_fight", self.has_ender_dragon_fight);
        compound.insert("coordinate_scale", self.coordinate_scale);
        compound.insert("min_y", self.min_y);
        compound.insert("height", self.height);
        compound.insert("logical_height", self.logical_height);
        compound.insert("infiniburn", self.infiniburn);
        compound.insert("ambient_light", self.ambient_light);
        compound.insert(
            "monster_spawn_light_level",
            match &self.monster_spawn_light_level {
                MonsterSpawnLightLevel::Simple(v) => NbtTag::Int(*v),
                MonsterSpawnLightLevel::Complex {
                    distribution_type,
                    min_inclusive,
                    max_inclusive,
                } => {
                    let mut inner = NbtCompound::new();
                    inner.insert("type", *distribution_type);
                    inner.insert("min_inclusive", *min_inclusive);
                    inner.insert("max_inclusive", *max_inclusive);
                    NbtTag::Compound(inner)
                }
            },
        );
        compound.insert(
            "monster_spawn_block_light_limit",
            self.monster_spawn_block_light_limit,
        );
        if let Some(skybox) = self.skybox {
            compound.insert("skybox", skybox);
        }
        if let Some(cardinal_light) = self.cardinal_light {
            compound.insert("cardinal_light", cardinal_light);
        }

        // Attributes compound
        let mut attributes = NbtCompound::new();

        // Visual attributes
        if let Some(sky_color) = self.sky_color {
            attributes.insert("minecraft:visual/sky_color", sky_color);
        }
        if let Some(fog_color) = self.fog_color {
            attributes.insert("minecraft:visual/fog_color", fog_color);
        }
        if let Some(cloud_color) = self.cloud_color {
            attributes.insert("minecraft:visual/cloud_color", cloud_color);
        }
        if let Some(cloud_height) = self.cloud_height {
            attributes.insert("minecraft:visual/cloud_height", cloud_height);
        }
        if let Some(ambient_light_color) = self.ambient_light_color {
            attributes.insert("minecraft:visual/ambient_light_color", ambient_light_color);
        }
        if let Some(sky_light_color) = self.sky_light_color {
            attributes.insert("minecraft:visual/sky_light_color", sky_light_color);
        }
        if let Some(sky_light_factor) = self.sky_light_factor {
            attributes.insert("minecraft:visual/sky_light_factor", sky_light_factor);
        }
        if let Some(fog_start_distance) = self.fog_start_distance {
            attributes.insert("minecraft:visual/fog_start_distance", fog_start_distance);
        }
        if let Some(fog_end_distance) = self.fog_end_distance {
            attributes.insert("minecraft:visual/fog_end_distance", fog_end_distance);
        }
        if let Some(particle_type) = self.default_dripstone_particle {
            let mut particle = NbtCompound::new();
            particle.insert("type", particle_type);
            attributes.insert(
                "minecraft:visual/default_dripstone_particle",
                NbtTag::Compound(particle),
            );
        }

        // Gameplay attributes (network: syncable-only). Vanilla
        // `NETWORK_CODEC` keeps every syncable entry present in the map —
        // even default-equal ones — so presence, not value, gates emission.
        if let Some(fast_lava) = self.fast_lava {
            attributes.insert("minecraft:gameplay/fast_lava", fast_lava);
        }
        if let Some(piglins_zombify) = self.piglins_zombify {
            attributes.insert("minecraft:gameplay/piglins_zombify", piglins_zombify);
        }
        if let Some(sky_light_level) = self.sky_light_level {
            attributes.insert("minecraft:gameplay/sky_light_level", sky_light_level);
        }
        if let Some(water_evaporates) = self.water_evaporates {
            attributes.insert("minecraft:gameplay/water_evaporates", water_evaporates);
        }

        // Audio attributes
        if let Some(mood) = &self.mood_sound {
            let mut mood_compound = NbtCompound::new();
            let sound = mood.sound.key.to_string();
            mood_compound.insert("sound", sound.as_str());
            mood_compound.insert("tick_delay", mood.tick_delay);
            mood_compound.insert("block_search_extent", mood.block_search_extent);
            mood_compound.insert("offset", mood.offset);
            let mut ambient_sounds = NbtCompound::new();
            ambient_sounds.insert("mood", NbtTag::Compound(mood_compound));
            attributes.insert(
                "minecraft:audio/ambient_sounds",
                NbtTag::Compound(ambient_sounds),
            );
        }
        if let Some(bg_music) = &self.background_music {
            let mut music_compound = NbtCompound::new();
            let mut default_entry = NbtCompound::new();
            let sound = bg_music.default.sound.key.to_string();
            default_entry.insert("sound", sound.as_str());
            default_entry.insert("min_delay", bg_music.default.min_delay);
            default_entry.insert("max_delay", bg_music.default.max_delay);
            if bg_music.default.replace_current_music {
                default_entry.insert(
                    "replace_current_music",
                    bg_music.default.replace_current_music,
                );
            }
            music_compound.insert("default", NbtTag::Compound(default_entry));
            if let Some(creative) = &bg_music.creative {
                let mut creative_entry = NbtCompound::new();
                let sound = creative.sound.key.to_string();
                creative_entry.insert("sound", sound.as_str());
                creative_entry.insert("min_delay", creative.min_delay);
                creative_entry.insert("max_delay", creative.max_delay);
                if creative.replace_current_music {
                    creative_entry.insert("replace_current_music", creative.replace_current_music);
                }
                music_compound.insert("creative", NbtTag::Compound(creative_entry));
            }
            attributes.insert(
                "minecraft:audio/background_music",
                NbtTag::Compound(music_compound),
            );
        }

        if !attributes.is_empty() {
            compound.insert("attributes", NbtTag::Compound(attributes));
        }
        if let Some(timelines) = self.timelines {
            compound.insert("timelines", timelines);
        }
        if let Some(clock) = self.default_clock {
            compound.insert("default_clock", clock.key.to_string().as_str());
        }

        NbtTag::Compound(compound)
    }

    /// Full direct codec: network form plus all server-only gameplay
    /// attributes (`bed_rule`, `straw_bed_rule`, `can_start_raid`,
    /// `respawn_anchor_works`, `nether_portal_spawns_piglin`,
    /// `snow_golem_melts`). Vanilla `DIRECT_CODEC` shares the network field
    /// order and encodes every map entry present in the dimension JSON.
    #[must_use]
    pub fn to_direct_nbt_tag(&self) -> NbtTag {
        use simdnbt::owned::{NbtCompound, NbtTag};
        let NbtTag::Compound(mut compound) = self.to_network_nbt() else {
            unreachable!("dimension type network NBT is always a compound");
        };
        let mut server_attributes = NbtCompound::new();
        if let Some(respawn_anchor_works) = self.respawn_anchor_works {
            server_attributes.insert(
                "minecraft:gameplay/respawn_anchor_works",
                respawn_anchor_works,
            );
        }
        if let Some(can_start_raid) = self.can_start_raid {
            server_attributes.insert("minecraft:gameplay/can_start_raid", can_start_raid);
        }
        if let Some(snow_golem_melts) = self.snow_golem_melts {
            server_attributes.insert("minecraft:gameplay/snow_golem_melts", snow_golem_melts);
        }
        if let Some(nether_portal_spawns_piglin) = self.nether_portal_spawns_piglin {
            server_attributes.insert(
                "minecraft:gameplay/nether_portal_spawns_piglin",
                nether_portal_spawns_piglin,
            );
        }
        for (key, rule) in [
            ("minecraft:gameplay/bed_rule", &self.bed_rule),
            ("minecraft:gameplay/straw_bed_rule", &self.straw_bed_rule),
        ] {
            // Vanilla `BedRule.CODEC` order: can_sleep, can_set_spawn,
            // destroy_on_use?, destroy_on_leave?, error_message?.
            let mut bed_rule = NbtCompound::new();
            bed_rule.insert("can_sleep", rule.can_sleep);
            bed_rule.insert("can_set_spawn", rule.can_set_spawn);
            if rule.destroy_on_use {
                bed_rule.insert("destroy_on_use", rule.destroy_on_use);
            }
            if rule.destroy_on_leave {
                bed_rule.insert("destroy_on_leave", rule.destroy_on_leave);
            }
            if let Some(key) = rule.error_message_key {
                let mut msg = NbtCompound::new();
                msg.insert("translate", key);
                bed_rule.insert("error_message", NbtTag::Compound(msg));
            }
            server_attributes.insert(key, NbtTag::Compound(bed_rule));
        }
        // Merge server-only attributes into the existing `attributes` compound.
        let mut merged = match compound.take("attributes") {
            Some(NbtTag::Compound(existing)) => existing,
            _ => NbtCompound::new(),
        };
        for (key, value) in server_attributes.iter() {
            let key_str = key.to_str();
            merged.insert(key_str.as_ref(), value.clone());
        }
        compound.insert("attributes", NbtTag::Compound(merged));
        NbtTag::Compound(compound)
    }
}

impl ToNbtTag for &DimensionType {
    fn to_nbt_tag(self) -> NbtTag {
        self.to_network_nbt()
    }
}

pub type DimensionTypeRef = &'static DimensionType;

pub struct DimensionTypeRegistry {
    dimension_types_by_id: Vec<DimensionTypeRef>,
    dimension_types_by_key: FxHashMap<Identifier, usize>,
    allows_registering: bool,
}

impl DimensionTypeRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            dimension_types_by_id: Vec::new(),
            dimension_types_by_key: FxHashMap::default(),
            allows_registering: true,
        }
    }

    #[must_use]
    pub fn get_ids(&self) -> Vec<Identifier> {
        self.dimension_types_by_key.keys().cloned().collect()
    }
}

crate::impl_standard_methods!(
    DimensionTypeRegistry,
    DimensionTypeRef,
    dimension_types_by_id,
    dimension_types_by_key,
    allows_registering
);

crate::impl_registry!(
    DimensionTypeRegistry,
    DimensionType,
    dimension_types_by_id,
    dimension_types_by_key,
    dimension_types
);

#[cfg(test)]
mod tests {
    use crate::dimension_type::DimensionType;
    use crate::vanilla_dimension_types::{OVERWORLD, THE_END, THE_NETHER};

    #[test]
    fn teleportation_scale_matches_vanilla_coordinate_ratio() {
        assert_eq!(
            DimensionType::get_teleportation_scale(&OVERWORLD, &THE_NETHER),
            0.125
        );
        assert_eq!(
            DimensionType::get_teleportation_scale(&THE_NETHER, &OVERWORLD),
            8.0
        );
        assert_eq!(
            DimensionType::get_teleportation_scale(&THE_END, &THE_NETHER),
            0.125
        );
    }
}
