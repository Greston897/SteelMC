#![expect(
    clippy::unwrap_used,
    reason = "build script must fail immediately on invalid extracted dimension data"
)]

//! Build-time codegen for the `minecraft:dimension` registry (3/3).
//!
//! Also hosts the shared `LevelStem` generator parser reused by
//! `world_presets` (flat settings, biome sources).

use heck::ToShoutySnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::datapack::{quote_id, read_json_file, strip_vanilla};

fn generate_biome_source(owner: &str, json: &serde_json::Value) -> TokenStream {
    let kind = json
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("{owner} biome_source is missing type"));
    match kind {
        "minecraft:fixed" => {
            let biome = quote_id(
                json.get("biome")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_else(|| panic!("{owner} fixed biome_source is missing biome")),
            );
            quote! { LevelStemBiomeSource::Fixed { biome: #biome } }
        }
        "minecraft:multi_noise" => {
            let preset = match json.get("preset").and_then(serde_json::Value::as_str) {
                Some(id) => {
                    let quoted = quote_id(id);
                    quote! { Some(#quoted) }
                }
                None => quote! { None },
            };
            quote! { LevelStemBiomeSource::MultiNoise { preset: #preset } }
        }
        "minecraft:the_end" => quote! { LevelStemBiomeSource::TheEnd },
        "minecraft:checkerboard" => {
            let biomes = json
                .get("biomes")
                .and_then(serde_json::Value::as_array)
                .unwrap_or_else(|| panic!("{owner} checkerboard biome_source is missing biomes"))
                .iter()
                .map(|entry| {
                    quote_id(entry.as_str().unwrap_or_else(|| {
                        panic!("{owner} checkerboard biome must be a string: {entry}")
                    }))
                });
            let scale = json
                .get("scale")
                .and_then(serde_json::Value::as_i64)
                .and_then(|v| i32::try_from(v).ok())
                .unwrap_or_else(|| panic!("{owner} checkerboard biome_source is missing scale"));
            quote! {
                LevelStemBiomeSource::Checkerboard {
                    biomes: vec![#(#biomes),*],
                    scale: #scale,
                }
            }
        }
        other => panic!("{owner} has unknown biome_source type: {other}"),
    }
}

/// Parses flat generator/preset `settings` into the shared field tokens.
///
/// Returns `(biome, lakes, features, layers, structure_overrides)`.
fn generate_flat_settings(
    owner: &str,
    settings: &serde_json::Value,
) -> (TokenStream, bool, bool, Vec<TokenStream>, TokenStream) {
    let biome = quote_id(
        settings
            .get("biome")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("{owner} flat settings are missing biome")),
    );
    let lakes = settings
        .get("lakes")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let features = settings
        .get("features")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let layers = settings
        .get("layers")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("{owner} flat settings are missing layers"))
        .iter()
        .map(|layer| {
            let block = quote_id(
                layer
                    .get("block")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_else(|| panic!("{owner} flat layer is missing block")),
            );
            let height = layer
                .get("height")
                .and_then(serde_json::Value::as_i64)
                .and_then(|v| i32::try_from(v).ok())
                .unwrap_or_else(|| panic!("{owner} flat layer is missing height"));
            quote! { FlatBlockLayer { block: #block, height: #height } }
        })
        .collect::<Vec<_>>();
    // `structure_overrides` is a list of structure *tag* keys (`TagKey`
    // codec: plain strings without `#`), or a single tag string.
    let structure_overrides = match settings.get("structure_overrides") {
        None => quote! { None },
        Some(value) => {
            if let Some(single) = value.as_str() {
                let tag = quote_id(single);
                quote! { Some(vec![#tag]) }
            } else if let Some(list) = value.as_array() {
                let tags = list.iter().map(|entry| {
                    quote_id(entry.as_str().unwrap_or_else(|| {
                        panic!("{owner} structure override must be a string: {entry}")
                    }))
                });
                quote! { Some(vec![#(#tags),*]) }
            } else {
                panic!("{owner} structure_overrides must be a string or list")
            }
        }
    };
    (biome, lakes, features, layers, structure_overrides)
}

fn generate_generator(owner: &str, json: &serde_json::Value) -> TokenStream {
    let kind = json
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("{owner} generator is missing type"));
    match kind {
        "minecraft:noise" => {
            let settings = quote_id(
                json.get("settings")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_else(|| panic!("{owner} noise generator is missing settings")),
            );
            let biome_source = generate_biome_source(
                owner,
                json.get("biome_source").unwrap_or_else(|| {
                    panic!("{owner} noise generator is missing biome_source")
                }),
            );
            quote! {
                LevelStemGenerator::Noise {
                    settings: #settings,
                    biome_source: #biome_source,
                }
            }
        }
        "minecraft:flat" => {
            let settings = json
                .get("settings")
                .unwrap_or_else(|| panic!("{owner} flat generator is missing settings"));
            let (biome, lakes, features, layers, structure_overrides) =
                generate_flat_settings(owner, settings);
            quote! {
                LevelStemGenerator::Flat {
                    biome: #biome,
                    lakes: #lakes,
                    features: #features,
                    layers: vec![#(#layers),*],
                    structure_overrides: #structure_overrides,
                }
            }
        }
        "minecraft:debug" => quote! { LevelStemGenerator::Debug },
        other => panic!("{owner} has unknown generator type: {other}"),
    }
}

/// Parses one inline `dimension` stem (`dimension/*.json` or a world-preset
/// `dimensions` map value) into a `DimensionStem` literal.
pub(crate) fn generate_stem(owner: &str, key: &TokenStream, json: &serde_json::Value) -> TokenStream {
    let dimension_type = quote_id(
        json.get("type")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("{owner} stem is missing type")),
    );
    let generator = generate_generator(
        owner,
        json.get("generator")
            .unwrap_or_else(|| panic!("{owner} stem is missing generator")),
    );
    quote! {
        DimensionStem {
            key: #key,
            dimension_type: #dimension_type,
            generator: #generator,
        }
    }
}

pub(crate) fn build() -> TokenStream {
    // The vanilla *server* jar ships no `dimension/` dir (standalone stem
    // files exist in the client jar / mcmeta only); dedicated servers derive
    // stems from the active world preset. `normal.json` stems are
    // value-identical to mcmeta `dimension/*.json`, so the default world
    // preset is the codegen source for the 3 vanilla stems.
    let preset = read_json_file("worldgen/world_preset/normal.json");
    let stems = preset
        .get("dimensions")
        .and_then(serde_json::Value::as_object)
        .unwrap_or_else(|| panic!("world preset normal is missing dimensions"));
    let mut entries: Vec<(&String, &serde_json::Value)> = stems.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let keys: Vec<&str> = entries.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(
        keys,
        [
            "minecraft:overworld",
            "minecraft:the_end",
            "minecraft:the_nether",
        ],
        "expected the 3 vanilla dimension stems, got {keys:?}"
    );

    let mut stream = TokenStream::new();
    stream.extend(quote! {
        use std::sync::LazyLock;

        use steel_utils::Identifier;

        // NOTE: all three vanilla dimensions use noise generators, so
        // `FlatBlockLayer` stays unimported.
        use crate::world_preset::{
            DimensionStem, DimensionStemRegistry, LevelStemBiomeSource, LevelStemGenerator,
        };
    });

    let mut register_stream = TokenStream::new();
    for (stem_id, stem_json) in &entries {
        let name = strip_vanilla(stem_id);
        let ident = Ident::new(&name.to_shouty_snake_case(), Span::call_site());
        let key = quote_id(stem_id);
        let stem = generate_stem(&format!("dimension {name}"), &key, stem_json);
        stream.extend(quote! {
            pub static #ident: LazyLock<DimensionStem> = LazyLock::new(|| #stem);
        });
        register_stream.extend(quote! {
            registry.register(&#ident);
        });
    }

    stream.extend(quote! {
        pub fn register_dimensions(registry: &mut DimensionStemRegistry) {
            #register_stream
        }
    });
    stream
}
