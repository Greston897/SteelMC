#![expect(
    clippy::unwrap_used,
    reason = "build script must fail immediately on invalid extracted preset data"
)]

//! Build-time codegen for `minecraft:worldgen/flat_level_generator_preset`
//! (9/9) and `minecraft:worldgen/world_preset` (7/7).

use heck::ToShoutySnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::datapack::{quote_id, read_entries, strip_vanilla};
use crate::dimensions::generate_stem;

pub(crate) fn build_flat() -> TokenStream {
    let entries = read_entries("worldgen/flat_level_generator_preset");
    assert_eq!(
        entries.len(),
        9,
        "expected 9 vanilla flat_level_generator_preset files, got {}",
        entries.len()
    );

    let mut stream = TokenStream::new();
    stream.extend(quote! {
        use std::sync::LazyLock;

        use steel_utils::Identifier;

        use crate::world_preset::{
            FlatBlockLayer, FlatLevelGeneratorPreset, FlatLevelGeneratorPresetRegistry,
        };
    });

    let mut register_stream = TokenStream::new();
    for (name, json) in &entries {
        let ident = Ident::new(&name.to_shouty_snake_case(), Span::call_site());
        let key = quote_id(&format!("minecraft:{name}"));
        let display_item = quote_id(
            json.get("display")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_else(|| panic!("flat preset {name} is missing display")),
        );
        let settings = json
            .get("settings")
            .unwrap_or_else(|| panic!("flat preset {name} is missing settings"));
        let biome = quote_id(
            settings
                .get("biome")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_else(|| panic!("flat preset {name} settings are missing biome")),
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
            .unwrap_or_else(|| panic!("flat preset {name} settings are missing layers"))
            .iter()
            .map(|layer| {
                let block = quote_id(
                    layer
                        .get("block")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_else(|| panic!("flat preset {name} layer is missing block")),
                );
                let height = layer
                    .get("height")
                    .and_then(serde_json::Value::as_i64)
                    .and_then(|v| i32::try_from(v).ok())
                    .unwrap_or_else(|| panic!("flat preset {name} layer is missing height"));
                quote! { FlatBlockLayer { block: #block, height: #height } }
            });
        let structure_overrides = match settings.get("structure_overrides") {
            None => quote! { None },
            Some(value) => {
                if let Some(single) = value.as_str() {
                    let tag = quote_id(single);
                    quote! { Some(vec![#tag]) }
                } else if let Some(list) = value.as_array() {
                    let tags = list.iter().map(|entry| {
                        quote_id(entry.as_str().unwrap_or_else(|| {
                            panic!("flat preset {name} structure override must be a string")
                        }))
                    });
                    quote! { Some(vec![#(#tags),*]) }
                } else {
                    panic!("flat preset {name} structure_overrides must be a string or list")
                }
            }
        };
        stream.extend(quote! {
            pub static #ident: LazyLock<FlatLevelGeneratorPreset> =
                LazyLock::new(|| FlatLevelGeneratorPreset {
                    key: #key,
                    display_item: #display_item,
                    biome: #biome,
                    lakes: #lakes,
                    features: #features,
                    layers: vec![#(#layers),*],
                    structure_overrides: #structure_overrides,
                });
        });
        register_stream.extend(quote! {
            registry.register(&#ident);
        });
    }

    stream.extend(quote! {
        pub fn register_flat_level_generator_presets(
            registry: &mut FlatLevelGeneratorPresetRegistry,
        ) {
            #register_stream
        }
    });
    stream
}

pub(crate) fn build_world() -> TokenStream {
    let entries = read_entries("worldgen/world_preset");
    assert_eq!(
        entries.len(),
        7,
        "expected 7 vanilla world_preset files, got {}",
        entries.len()
    );

    let mut stream = TokenStream::new();
    stream.extend(quote! {
        use std::sync::LazyLock;

        use steel_utils::Identifier;

        use crate::world_preset::{
            DimensionStem, FlatBlockLayer, LevelStemBiomeSource, LevelStemGenerator, WorldPreset,
            WorldPresetRegistry,
        };
    });

    let mut register_stream = TokenStream::new();
    for (name, json) in &entries {
        let ident = Ident::new(&name.to_shouty_snake_case(), Span::call_site());
        let key = quote_id(&format!("minecraft:{name}"));
        let dimensions = json
            .get("dimensions")
            .and_then(serde_json::Value::as_object)
            .unwrap_or_else(|| panic!("world preset {name} is missing dimensions"))
            .iter()
            .map(|(stem_id, stem)| {
                let stem_key = quote_id(&format!("minecraft:{}", strip_vanilla(stem_id)));
                generate_stem(&format!("world preset {name} stem {stem_id}"), &stem_key, stem)
            });
        stream.extend(quote! {
            pub static #ident: LazyLock<WorldPreset> = LazyLock::new(|| WorldPreset {
                key: #key,
                dimensions: vec![#(#dimensions),*],
            });
        });
        register_stream.extend(quote! {
            registry.register(&#ident);
        });
    }

    stream.extend(quote! {
        pub fn register_world_presets(registry: &mut WorldPresetRegistry) {
            #register_stream
        }
    });
    stream
}
