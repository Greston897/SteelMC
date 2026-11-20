//! Build-time codegen for `minecraft:context_int_provider` (26/26) and
//! `minecraft:context_float_provider` (4/4).
//!
//! Also hosts the shared loot-condition parser reused by `predicates`.

use heck::ToShoutySnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::datapack::{quote_id, read_entries, strip_tag, strip_vanilla};

/// Parses a loot condition: registry reference string, or a typed object.
///
/// Covers the condition shapes vanilla data uses (`match_block`,
/// `match_tool`) plus the combinators (`inverted`, `all_of`, `any_of`,
/// `random_chance`) for datapack parity.
pub(crate) fn generate_condition(value: &serde_json::Value) -> TokenStream {
    if let Some(reference) = value.as_str() {
        let id = quote_id(&format!("minecraft:{}", strip_vanilla(reference)));
        return quote! { ContextCondition::Reference(#id) };
    }
    if let Some(flag) = value.as_bool() {
        return quote! { ContextCondition::Constant(#flag) };
    }
    let object = value
        .as_object()
        .unwrap_or_else(|| panic!("condition must be a string, bool or object: {value}"));
    let kind = object
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("condition object is missing type: {value}"));
    match kind {
        "minecraft:inverted" => {
            let term = object
                .get("term")
                .unwrap_or_else(|| panic!("inverted condition is missing term: {value}"));
            let inner = generate_condition(term);
            quote! { ContextCondition::Inverted(Box::new(#inner)) }
        }
        "minecraft:all_of" | "minecraft:any_of" => {
            let terms = object
                .get("terms")
                .and_then(serde_json::Value::as_array)
                .unwrap_or_else(|| panic!("{kind} condition is missing terms: {value}"))
                .iter()
                .map(generate_condition);
            if kind == "minecraft:all_of" {
                quote! { ContextCondition::AllOf(vec![#(#terms),*]) }
            } else {
                quote! { ContextCondition::AnyOf(vec![#(#terms),*]) }
            }
        }
        "minecraft:random_chance" => {
            let chance = object
                .get("chance")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or_else(|| panic!("random_chance condition needs a bare chance: {value}"))
                as f32;
            quote! { ContextCondition::RandomChance(#chance) }
        }
        "minecraft:match_block" => {
            let mut blocks = Vec::new();
            let mut block_tag = quote! { None };
            if let Some(holder) = object.get("blocks") {
                if let Some(single) = holder.as_str() {
                    if single.starts_with('#') {
                        let tag = quote_id(strip_tag(single));
                        block_tag = quote! { Some(#tag) };
                    } else {
                        blocks.push(quote_id(single));
                    }
                } else if let Some(list) = holder.as_array() {
                    for entry in list {
                        let id = entry.as_str().unwrap_or_else(|| {
                            panic!("match_block blocks entry must be a string: {entry}")
                        });
                        assert!(
                            !id.starts_with('#'),
                            "tag references cannot appear inside match_block blocks lists: {id}"
                        );
                        blocks.push(quote_id(id));
                    }
                } else {
                    panic!("match_block blocks must be a string or list: {holder}");
                }
            }
            let mut state = Vec::new();
            if let Some(map) = object.get("state") {
                for (prop, expected) in map
                    .as_object()
                    .unwrap_or_else(|| panic!("match_block state must be an object: {map}"))
                {
                    let expected = expected.as_str().unwrap_or_else(|| {
                        panic!("match_block state value must be a string: {expected}")
                    });
                    state.push(quote! { (#prop.to_string(), #expected.to_string()) });
                }
            }
            assert!(
                object.get("nbt").is_none(),
                "match_block nbt is not supported by context codegen: {value}"
            );
            quote! {
                ContextCondition::MatchBlock {
                    blocks: vec![#(#blocks),*],
                    block_tag: #block_tag,
                    state: vec![#(#state),*],
                }
            }
        }
        "minecraft:match_tool" => {
            let mut items = Vec::new();
            let mut item_tag = quote! { None };
            let mut enchantments = Vec::new();
            if let Some(predicate) = object.get("predicate") {
                let pred = predicate.as_object().unwrap_or_else(|| {
                    panic!("match_tool predicate must be an object: {predicate}")
                });
                if let Some(holder) = pred.get("items") {
                    if let Some(single) = holder.as_str() {
                        if single.starts_with('#') {
                            let tag = quote_id(strip_tag(single));
                            item_tag = quote! { Some(#tag) };
                        } else {
                            items.push(quote_id(single));
                        }
                    } else if let Some(list) = holder.as_array() {
                        for entry in list {
                            let id = entry.as_str().unwrap_or_else(|| {
                                panic!("match_tool items entry must be a string: {entry}")
                            });
                            assert!(
                                !id.starts_with('#'),
                                "tag references cannot appear inside match_tool items lists: {id}"
                            );
                            items.push(quote_id(id));
                        }
                    } else {
                        panic!("match_tool items must be a string or list: {holder}");
                    }
                }
                if let Some(component_map) = pred.get("predicates") {
                    let components = component_map.as_object().unwrap_or_else(|| {
                        panic!("match_tool predicates must be an object: {component_map}")
                    });
                    for (component, checks) in components {
                        assert_eq!(
                            component, "minecraft:enchantments",
                            "unsupported match_tool component predicate: {component}"
                        );
                        for check in checks.as_array().unwrap_or_else(|| {
                            panic!("enchantments checks must be a list: {checks}")
                        }) {
                            let holder = check.get("enchantments").unwrap_or_else(|| {
                                panic!("enchantment check is missing enchantments: {check}")
                            });
                            let ids: Vec<&str> = if let Some(single) = holder.as_str() {
                                vec![single]
                            } else if let Some(list) = holder.as_array() {
                                list.iter()
                                    .map(|entry| {
                                        entry.as_str().unwrap_or_else(|| {
                                            panic!("enchantment entry must be a string: {entry}")
                                        })
                                    })
                                    .collect()
                            } else {
                                panic!("enchantments must be a string or list: {holder}")
                            };
                            let levels = check.get("levels");
                            let min_level = levels
                                .and_then(|l| l.get("min"))
                                .and_then(serde_json::Value::as_i64)
                                .and_then(|v| i32::try_from(v).ok())
                                .map_or(quote! { None }, |v| quote! { Some(#v) });
                            let max_level = levels
                                .and_then(|l| l.get("max"))
                                .and_then(serde_json::Value::as_i64)
                                .and_then(|v| i32::try_from(v).ok())
                                .map_or(quote! { None }, |v| quote! { Some(#v) });
                            for id in ids {
                                let (enchantment, is_tag) = if id.starts_with('#') {
                                    (quote_id(strip_tag(id)), true)
                                } else {
                                    (quote_id(id), false)
                                };
                                enchantments.push(quote! {
                                    EnchantmentRequirement {
                                        enchantment: #enchantment,
                                        is_tag: #is_tag,
                                        min_level: #min_level,
                                        max_level: #max_level,
                                    }
                                });
                            }
                        }
                    }
                }
            }
            quote! {
                ContextCondition::MatchTool {
                    items: vec![#(#items),*],
                    item_tag: #item_tag,
                    enchantments: vec![#(#enchantments),*],
                }
            }
        }
        other => panic!("unsupported loot condition type: {other} in {value}"),
    }
}

fn generate_int_expr(value: &serde_json::Value) -> TokenStream {
    if let Some(number) = value.as_i64().and_then(|v| i32::try_from(v).ok()) {
        return quote! { ContextIntExpr::Constant(#number) };
    }
    if let Some(reference) = value.as_str() {
        let id = quote_id(&format!("minecraft:{}", strip_vanilla(reference)));
        return quote! { ContextIntExpr::Reference(#id) };
    }
    let object = value
        .as_object()
        .unwrap_or_else(|| panic!("int provider must be a number, string or object: {value}"));
    let kind = object
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("int provider object is missing type: {value}"));
    match kind {
        "minecraft:uniform" => {
            let min = generate_int_expr(
                object
                    .get("min")
                    .unwrap_or_else(|| panic!("uniform int provider is missing min: {value}")),
            );
            let max = generate_int_expr(
                object
                    .get("max")
                    .unwrap_or_else(|| panic!("uniform int provider is missing max: {value}")),
            );
            quote! {
                ContextIntExpr::Uniform {
                    min_inclusive: Box::new(#min),
                    max_inclusive: Box::new(#max),
                }
            }
        }
        "minecraft:weighted_list" => {
            let entries = object
                .get("distribution")
                .and_then(serde_json::Value::as_array)
                .unwrap_or_else(|| panic!("weighted_list is missing distribution: {value}"))
                .iter()
                .map(|entry| {
                    let data = generate_int_expr(
                        entry
                            .get("data")
                            .unwrap_or_else(|| panic!("weighted entry is missing data: {entry}")),
                    );
                    let weight = entry
                        .get("weight")
                        .and_then(serde_json::Value::as_u64)
                        .and_then(|v| u32::try_from(v).ok())
                        .unwrap_or_else(|| panic!("weighted entry is missing weight: {entry}"));
                    quote! { (#data, #weight) }
                });
            quote! { ContextIntExpr::WeightedList(vec![#(#entries),*]) }
        }
        "minecraft:add" | "minecraft:sub" | "minecraft:mul" | "minecraft:div" => {
            let left = generate_int_expr(
                object
                    .get("left")
                    .unwrap_or_else(|| panic!("{kind} int provider is missing left: {value}")),
            );
            let right = generate_int_expr(
                object
                    .get("right")
                    .unwrap_or_else(|| panic!("{kind} int provider is missing right: {value}")),
            );
            match kind {
                "minecraft:add" => {
                    quote! { ContextIntExpr::Add(Box::new(#left), Box::new(#right)) }
                }
                "minecraft:sub" => {
                    quote! { ContextIntExpr::Sub(Box::new(#left), Box::new(#right)) }
                }
                "minecraft:mul" => {
                    quote! { ContextIntExpr::Mul(Box::new(#left), Box::new(#right)) }
                }
                _ => quote! { ContextIntExpr::Div(Box::new(#left), Box::new(#right)) },
            }
        }
        "minecraft:conditional" => {
            let condition = generate_condition(
                object
                    .get("condition")
                    .unwrap_or_else(|| panic!("conditional is missing condition: {value}")),
            );
            let on_true = generate_int_expr(
                object
                    .get("on_true")
                    .unwrap_or_else(|| panic!("conditional is missing on_true: {value}")),
            );
            let on_false = generate_int_expr(
                object
                    .get("on_false")
                    .unwrap_or_else(|| panic!("conditional is missing on_false: {value}")),
            );
            quote! {
                ContextIntExpr::Conditional {
                    condition: #condition,
                    on_true: Box::new(#on_true),
                    on_false: Box::new(#on_false),
                }
            }
        }
        "minecraft:number_dispatcher" => {
            let cases = object
                .get("cases")
                .and_then(serde_json::Value::as_array)
                .unwrap_or_else(|| panic!("number_dispatcher is missing cases: {value}"))
                .iter()
                .map(|case| {
                    let condition =
                        generate_condition(case.get("condition").unwrap_or_else(|| {
                            panic!("dispatcher case is missing condition: {case}")
                        }));
                    let result = generate_int_expr(
                        case.get("value")
                            .unwrap_or_else(|| panic!("dispatcher case is missing value: {case}")),
                    );
                    quote! { (#condition, #result) }
                });
            let default = generate_int_expr(
                object
                    .get("default")
                    .unwrap_or_else(|| panic!("number_dispatcher is missing default: {value}")),
            );
            quote! {
                ContextIntExpr::NumberDispatcher {
                    cases: vec![#(#cases),*],
                    default: Box::new(#default),
                }
            }
        }
        other => panic!("unsupported context int provider type: {other} in {value}"),
    }
}

fn generate_float_expr(value: &serde_json::Value) -> TokenStream {
    if let Some(number) = value.as_f64() {
        let number = number as f32;
        return quote! { ContextFloatExpr::Constant(#number) };
    }
    if let Some(reference) = value.as_str() {
        let id = quote_id(&format!("minecraft:{}", strip_vanilla(reference)));
        return quote! { ContextFloatExpr::Reference(#id) };
    }
    let object = value
        .as_object()
        .unwrap_or_else(|| panic!("float provider must be a number, string or object: {value}"));
    let kind = object
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("float provider object is missing type: {value}"));
    match kind {
        "minecraft:uniform" => {
            let min = generate_float_expr(
                object
                    .get("min")
                    .unwrap_or_else(|| panic!("uniform float provider is missing min: {value}")),
            );
            let max = generate_float_expr(
                object
                    .get("max")
                    .unwrap_or_else(|| panic!("uniform float provider is missing max: {value}")),
            );
            quote! {
                ContextFloatExpr::Uniform {
                    min_inclusive: Box::new(#min),
                    max_exclusive: Box::new(#max),
                }
            }
        }
        "minecraft:add" | "minecraft:sub" | "minecraft:mul" | "minecraft:div" => {
            let left = generate_float_expr(
                object
                    .get("left")
                    .unwrap_or_else(|| panic!("{kind} float provider is missing left: {value}")),
            );
            let right = generate_float_expr(
                object
                    .get("right")
                    .unwrap_or_else(|| panic!("{kind} float provider is missing right: {value}")),
            );
            match kind {
                "minecraft:add" => {
                    quote! { ContextFloatExpr::Add(Box::new(#left), Box::new(#right)) }
                }
                "minecraft:sub" => {
                    quote! { ContextFloatExpr::Sub(Box::new(#left), Box::new(#right)) }
                }
                "minecraft:mul" => {
                    quote! { ContextFloatExpr::Mul(Box::new(#left), Box::new(#right)) }
                }
                _ => quote! { ContextFloatExpr::Div(Box::new(#left), Box::new(#right)) },
            }
        }
        "minecraft:conditional" => {
            let condition = generate_condition(
                object
                    .get("condition")
                    .unwrap_or_else(|| panic!("conditional is missing condition: {value}")),
            );
            let on_true = generate_float_expr(
                object
                    .get("on_true")
                    .unwrap_or_else(|| panic!("conditional is missing on_true: {value}")),
            );
            let on_false = generate_float_expr(
                object
                    .get("on_false")
                    .unwrap_or_else(|| panic!("conditional is missing on_false: {value}")),
            );
            quote! {
                ContextFloatExpr::Conditional {
                    condition: #condition,
                    on_true: Box::new(#on_true),
                    on_false: Box::new(#on_false),
                }
            }
        }
        "minecraft:number_dispatcher" => {
            let cases = object
                .get("cases")
                .and_then(serde_json::Value::as_array)
                .unwrap_or_else(|| panic!("number_dispatcher is missing cases: {value}"))
                .iter()
                .map(|case| {
                    let condition =
                        generate_condition(case.get("condition").unwrap_or_else(|| {
                            panic!("dispatcher case is missing condition: {case}")
                        }));
                    let result = generate_float_expr(
                        case.get("value")
                            .unwrap_or_else(|| panic!("dispatcher case is missing value: {case}")),
                    );
                    quote! { (#condition, #result) }
                });
            let default = generate_float_expr(
                object
                    .get("default")
                    .unwrap_or_else(|| panic!("number_dispatcher is missing default: {value}")),
            );
            quote! {
                ContextFloatExpr::NumberDispatcher {
                    cases: vec![#(#cases),*],
                    default: Box::new(#default),
                }
            }
        }
        other => panic!("unsupported context float provider type: {other} in {value}"),
    }
}

pub(crate) fn build_int() -> TokenStream {
    let entries = read_entries("context_int_provider");
    assert_eq!(
        entries.len(),
        26,
        "expected 26 vanilla context_int_provider files, got {}",
        entries.len()
    );

    let mut stream = TokenStream::new();
    stream.extend(quote! {
        use std::sync::LazyLock;

        use steel_utils::Identifier;

        // NOTE: `EnchantmentRequirement` is intentionally not imported — no
        // vanilla context_int_provider uses `match_tool`.
        use crate::context_provider::{
            ContextCondition, ContextIntExpr, ContextIntProvider, ContextIntProviderRegistry,
        };
    });

    let mut register_stream = TokenStream::new();
    for (name, json) in &entries {
        let ident = Ident::new(
            &name.replace('/', "_").to_shouty_snake_case(),
            Span::call_site(),
        );
        let key = quote_id(&format!("minecraft:{name}"));
        let expr = generate_int_expr(json);
        stream.extend(quote! {
            pub static #ident: LazyLock<ContextIntProvider> =
                LazyLock::new(|| ContextIntProvider { key: #key, expr: #expr });
        });
        register_stream.extend(quote! {
            registry.register(&#ident);
        });
    }

    stream.extend(quote! {
        pub fn register_context_int_providers(registry: &mut ContextIntProviderRegistry) {
            #register_stream
        }
    });
    stream
}

pub(crate) fn build_float() -> TokenStream {
    let entries = read_entries("context_float_provider");
    assert_eq!(
        entries.len(),
        4,
        "expected 4 vanilla context_float_provider files, got {}",
        entries.len()
    );

    let mut stream = TokenStream::new();
    stream.extend(quote! {
        use std::sync::LazyLock;

        use steel_utils::Identifier;

        // NOTE: `EnchantmentRequirement` is intentionally not imported — no
        // vanilla context_float_provider uses `match_tool`.
        use crate::context_provider::{
            ContextCondition, ContextFloatExpr, ContextFloatProvider,
            ContextFloatProviderRegistry,
        };
    });

    let mut register_stream = TokenStream::new();
    for (name, json) in &entries {
        let ident = Ident::new(
            &name.replace('/', "_").to_shouty_snake_case(),
            Span::call_site(),
        );
        let key = quote_id(&format!("minecraft:{name}"));
        let expr = generate_float_expr(json);
        stream.extend(quote! {
            pub static #ident: LazyLock<ContextFloatProvider> =
                LazyLock::new(|| ContextFloatProvider { key: #key, expr: #expr });
        });
        register_stream.extend(quote! {
            registry.register(&#ident);
        });
    }

    stream.extend(quote! {
        pub fn register_context_float_providers(registry: &mut ContextFloatProviderRegistry) {
            #register_stream
        }
    });
    stream
}
