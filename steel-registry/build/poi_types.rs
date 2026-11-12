use std::collections::BTreeMap;
use std::fs;

use heck::ToShoutySnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
struct BlockStateEntry {
    block: String,
    #[serde(default)]
    properties: BTreeMap<String, String>,
}

#[derive(Deserialize, Debug, Clone)]
struct PoiTypeJson {
    id: u32,
    name: String,
    ticket_count: u32,
    valid_range: u32,
    block_states: Vec<BlockStateEntry>,
}

#[derive(Deserialize, Debug)]
struct PoiTypesFile {
    poi_types: Vec<PoiTypeJson>,
}

/// A block plus the property constraints that identify which of its states belong to the
/// POI. An empty `properties` set means every state of the block matches.
struct BlockMatcher {
    block: String,
    properties: Vec<(String, String)>,
}

/// Collapses a POI type's enumerated `(block, properties)` states back into block
/// matchers, deriving the minimal property filter for each block.
///
/// A property is *constrained* if every state of the block shares one identical value for
/// it (e.g. beds are all `part == head`); properties that vary across states are *free*
/// and left unconstrained, which expands to all of their values at registration.
fn derive_matchers(poi: &PoiTypeJson) -> Vec<BlockMatcher> {
    // Group entries by block, preserving first-seen order for deterministic output.
    let mut order: Vec<String> = Vec::new();
    let mut groups: BTreeMap<String, Vec<&BlockStateEntry>> = BTreeMap::new();
    for entry in &poi.block_states {
        if !groups.contains_key(&entry.block) {
            order.push(entry.block.clone());
        }
        groups.entry(entry.block.clone()).or_default().push(entry);
    }

    order
        .into_iter()
        .map(|block| {
            let entries = &groups[&block];

            let mut values: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
            for entry in entries {
                for (name, value) in &entry.properties {
                    let observed = values.entry(name.as_str()).or_default();
                    if !observed.contains(&value.as_str()) {
                        observed.push(value.as_str());
                    }
                }
            }

            let mut properties: Vec<(String, String)> = Vec::new();
            let mut free_product: usize = 1;
            for (name, observed) in &values {
                if observed.len() == 1 {
                    properties.push(((*name).to_owned(), observed[0].to_owned()));
                } else {
                    free_product *= observed.len();
                }
            }
            properties.sort();

            // Rectangularity guard: the enumerated states must be exactly the cartesian
            // product of the free properties' observed values. A mismatch means this POI
            // needs a richer model than equality filters — fail the build loudly.
            //
            // Whether the observed values also cover each free property's *complete*
            // value set is checked against `blocks.json` at runtime by the generated
            // test below, because the property value counts only exist in Rust.
            assert_eq!(
                entries.len(),
                free_product,
                "POI `{}` block `{}` is not expressible as a property filter \
                 ({} states != {} from free properties)",
                poi.name,
                block,
                entries.len(),
                free_product,
            );

            BlockMatcher { block, properties }
        })
        .collect()
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=build_assets/poi_types.json");

    let content =
        fs::read_to_string("build_assets/poi_types.json").expect("Failed to read poi_types.json");
    let file: PoiTypesFile =
        serde_json::from_str(&content).expect("Failed to parse poi_types.json");
    let poi_types = file.poi_types.clone();

    let mut stream = TokenStream::new();

    stream.extend(quote! {
        use crate::poi::{PointOfInterestType, PoiBlockMatcher, PoiTypeRegistry};
        use crate::vanilla_blocks;
        use steel_utils::Identifier;
    });

    for poi_type in &poi_types {
        let poi_ident = Ident::new(&poi_type.name.to_shouty_snake_case(), Span::call_site());
        let poi_name = &poi_type.name;
        let ticket_count = poi_type.ticket_count;
        let search_distance = poi_type.valid_range;

        let matchers = derive_matchers(poi_type).into_iter().map(|matcher| {
            let block_ident = Ident::new(&matcher.block.to_shouty_snake_case(), Span::call_site());
            let properties = matcher.properties.iter().map(|(name, value)| {
                quote! { (#name, #value) }
            });
            quote! {
                PoiBlockMatcher {
                    block: &vanilla_blocks::#block_ident,
                    properties: &[#(#properties),*],
                }
            }
        });

        stream.extend(quote! {
            pub static #poi_ident: PointOfInterestType = PointOfInterestType {
                key: Identifier::vanilla_static(#poi_name),
                blocks: &[#(#matchers),*],
                ticket_count: #ticket_count,
                search_distance: #search_distance,
            };
        });
    }

    // Generate registration function (order matters - must match vanilla IDs)
    let mut register_stream = TokenStream::new();

    let mut sorted_poi_types = poi_types;
    sorted_poi_types.sort_by_key(|p| p.id);

    for poi_type in &sorted_poi_types {
        let poi_ident = Ident::new(&poi_type.name.to_shouty_snake_case(), Span::call_site());
        register_stream.extend(quote! {
            registry.register(&#poi_ident);
        });
    }

    stream.extend(quote! {
        pub fn register_poi_types(registry: &mut PoiTypeRegistry) {
            #register_stream
        }
    });

    stream.extend(matcher_test_tokens(&file));
    stream
}

/// Generates a registry test asserting every POI matcher selects exactly the
/// states the extractor enumerated.
///
/// This closes the gap the build-time rectangularity guard cannot see: the
/// matcher expands free properties to their full value set, so a property whose
/// observed values are a strict subset of the block's real value set would
/// silently match extra states.
fn matcher_test_tokens(file: &PoiTypesFile) -> TokenStream {
    let mut cases = Vec::new();
    for poi in &file.poi_types {
        let poi_ident = Ident::new(&poi.name.to_shouty_snake_case(), Span::call_site());
        let poi_name = &poi.name;
        for matcher in derive_matchers(poi) {
            let block_ident = Ident::new(&matcher.block.to_shouty_snake_case(), Span::call_site());
            let block_name = &matcher.block;
            let filter_len = matcher.properties.len();
            let properties = matcher.properties.iter().map(|(name, value)| {
                quote! { (#name, #value) }
            });
            let expected = poi
                .block_states
                .iter()
                .filter(|entry| &entry.block == &matcher.block)
                .count();
            cases.push(quote! {
                {
                    let filter: &[(&str, &str)] = &[#(#properties),*];
                    assert_eq!(filter.len(), #filter_len);
                    let matched = REGISTRY
                        .blocks
                        .matching_states(&vanilla_blocks::#block_ident, filter);
                    assert_eq!(
                        matched.len(),
                        #expected,
                        "POI `{}` block `{}` matcher {:?} selects {} states, extractor enumerated {}",
                        #poi_name,
                        #block_name,
                        filter,
                        matched.len(),
                        #expected,
                    );
                }
            });
        }
    }

    quote! {
        #[cfg(test)]
        #[test]
        fn poi_matchers_select_exactly_the_enumerated_states() {
            use crate::REGISTRY;
            use crate::init_vanilla_registry;
            init_vanilla_registry();
            #(#cases)*
        }
    }
}
