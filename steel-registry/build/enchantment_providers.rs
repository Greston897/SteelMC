#![expect(
    clippy::unwrap_used,
    reason = "build script must fail immediately on invalid extracted provider data"
)]

//! Build-time codegen for the `minecraft:enchantment_provider` registry (7/7).

use heck::ToShoutySnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use steel_utils::value_providers::IntProvider;

use crate::datapack::{quote_id, read_entries, strip_tag};
use crate::features::generate_int_provider;

/// Parses an enchantment holder set (`#tag`, single id, or id list).
fn generate_enchantment_set(holder: &serde_json::Value) -> TokenStream {
    if let Some(single) = holder.as_str() {
        if single.starts_with('#') {
            let tag = quote_id(strip_tag(single));
            return quote! { EnchantmentProviderSet::Tag(#tag) };
        }
        let id = quote_id(single);
        return quote! { EnchantmentProviderSet::List(vec![#id]) };
    }
    let list = holder
        .as_array()
        .unwrap_or_else(|| panic!("enchantments holder must be a string or list: {holder}"));
    let ids = list.iter().map(|entry| {
        let id = entry
            .as_str()
            .unwrap_or_else(|| panic!("enchantment entry must be a string: {entry}"));
        assert!(
            !id.starts_with('#'),
            "tag references cannot appear inside enchantment lists: {id}"
        );
        quote_id(id)
    });
    quote! { EnchantmentProviderSet::List(vec![#(#ids),*]) }
}

fn generate_kind(name: &str, json: &serde_json::Value) -> TokenStream {
    let kind = json
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("enchantment_provider {name} is missing type"));
    match kind {
        "minecraft:single" => {
            let enchantment = quote_id(
                json.get("enchantment")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_else(|| {
                        panic!("enchantment_provider {name} single is missing enchantment")
                    }),
            );
            let level: IntProvider = serde_json::from_value(
                json.get("level")
                    .cloned()
                    .unwrap_or_else(|| panic!("enchantment_provider {name} single is missing level")),
            )
            .unwrap_or_else(|e| panic!("enchantment_provider {name} level: {e}"));
            let level = generate_int_provider(&level);
            quote! {
                EnchantmentProviderKind::Single { enchantment: #enchantment, level: #level }
            }
        }
        "minecraft:by_cost" => {
            let enchantments = generate_enchantment_set(
                json.get("enchantments").unwrap_or_else(|| {
                    panic!("enchantment_provider {name} by_cost is missing enchantments")
                }),
            );
            let cost: IntProvider = serde_json::from_value(
                json.get("cost")
                    .cloned()
                    .unwrap_or_else(|| panic!("enchantment_provider {name} by_cost is missing cost")),
            )
            .unwrap_or_else(|e| panic!("enchantment_provider {name} cost: {e}"));
            let cost = generate_int_provider(&cost);
            quote! {
                EnchantmentProviderKind::ByCost { enchantments: #enchantments, cost: #cost }
            }
        }
        "minecraft:by_cost_with_difficulty" => {
            let enchantments = generate_enchantment_set(
                json.get("enchantments").unwrap_or_else(|| {
                    panic!("enchantment_provider {name} is missing enchantments")
                }),
            );
            let min_cost = json
                .get("min_cost")
                .and_then(serde_json::Value::as_i64)
                .and_then(|v| i32::try_from(v).ok())
                .unwrap_or_else(|| {
                    panic!("enchantment_provider {name} is missing min_cost")
                });
            let max_cost_span = json
                .get("max_cost_span")
                .and_then(serde_json::Value::as_i64)
                .and_then(|v| i32::try_from(v).ok())
                .unwrap_or_else(|| {
                    panic!("enchantment_provider {name} is missing max_cost_span")
                });
            quote! {
                EnchantmentProviderKind::ByCostWithDifficulty {
                    enchantments: #enchantments,
                    min_cost: #min_cost,
                    max_cost_span: #max_cost_span,
                }
            }
        }
        other => panic!("enchantment_provider {name} has unknown type: {other}"),
    }
}

pub(crate) fn build() -> TokenStream {
    let entries = read_entries("enchantment_provider");
    assert_eq!(
        entries.len(),
        7,
        "expected 7 vanilla enchantment_provider files, got {}",
        entries.len()
    );

    let mut stream = TokenStream::new();
    stream.extend(quote! {
        use std::sync::LazyLock;

        use steel_utils::Identifier;
        // NOTE: only bare-int levels occur in vanilla data, so
        // `WeightedIntProvider` stays unimported.
        use steel_utils::value_providers::IntProvider;

        use crate::enchantment_provider::{
            EnchantmentProvider, EnchantmentProviderKind, EnchantmentProviderRegistry,
            EnchantmentProviderSet,
        };
    });

    let mut register_stream = TokenStream::new();
    for (name, json) in &entries {
        let ident = Ident::new(
            &name.replace('/', "_").to_shouty_snake_case(),
            Span::call_site(),
        );
        let key = quote_id(&format!("minecraft:{name}"));
        let kind = generate_kind(name, json);
        stream.extend(quote! {
            pub static #ident: LazyLock<EnchantmentProvider> =
                LazyLock::new(|| EnchantmentProvider { key: #key, kind: #kind });
        });
        register_stream.extend(quote! {
            registry.register(&#ident);
        });
    }

    stream.extend(quote! {
        pub fn register_enchantment_providers(registry: &mut EnchantmentProviderRegistry) {
            #register_stream
        }
    });
    stream
}
