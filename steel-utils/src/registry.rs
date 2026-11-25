use crate::Identifier;

/// Creates a vanilla or a new custom tag for the generator build scripts
#[must_use]
pub fn registry_vanilla_or_custom_tag(key: &'static str) -> Identifier {
    if let Some(key) = key.strip_prefix("c:") {
        Identifier::new_static("c", key)
    } else {
        Identifier::vanilla_static(key)
    }
}

/// Returns `true` iff `key` (either `"minecraft:<path>"` or bare `"<path>"`)
/// is marked `.syncable()` in vanilla MC 26.3 `EnvironmentAttributes`.
///
/// Canonical single source of truth shared by the `steel-registry` build
/// script (biome/dimension/timeline codegen) and the runtime network codecs
/// (`Biome`, `DimensionType`, `Timeline` `NETWORK_CODEC`). All 24 `visual/*`
/// and all 4 `audio/*` attributes are syncable; of the 23 `gameplay/*`
/// attributes only `sky_light_level`, `water_evaporates`, `fast_lava`,
/// `piglins_zombify`, and `creaking_active` are syncable (33 total).
#[must_use]
pub fn is_syncable_environment_attribute(key: &str) -> bool {
    let path = match key.split_once(':') {
        Some(("minecraft", path)) => path,
        Some(_) => return false,
        None => key,
    };
    matches!(
        path,
        "visual/fog_color"
            | "visual/fog_start_distance"
            | "visual/fog_end_distance"
            | "visual/sky_fog_end_distance"
            | "visual/cloud_fog_end_distance"
            | "visual/water_fog_color"
            | "visual/water_fog_start_distance"
            | "visual/water_fog_end_distance"
            | "visual/sky_color"
            | "visual/sunrise_sunset_color"
            | "visual/cloud_color"
            | "visual/cloud_height"
            | "visual/sun_angle"
            | "visual/moon_angle"
            | "visual/star_angle"
            | "visual/moon_phase"
            | "visual/star_brightness"
            | "visual/block_light_tint"
            | "visual/sky_light_color"
            | "visual/sky_light_factor"
            | "visual/night_vision_color"
            | "visual/ambient_light_color"
            | "visual/default_dripstone_particle"
            | "visual/ambient_particles"
            | "audio/background_music"
            | "audio/music_volume"
            | "audio/ambient_sounds"
            | "audio/firefly_bush_sounds"
            | "gameplay/sky_light_level"
            | "gameplay/water_evaporates"
            | "gameplay/fast_lava"
            | "gameplay/piglins_zombify"
            | "gameplay/creaking_active"
    )
}
