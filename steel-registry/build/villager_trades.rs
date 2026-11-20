//! Build-time codegen for `minecraft:villager_trade` (391/391) and
//! `minecraft:trade_set` (68/68).
//!
//! Defaults mirror the vanilla codecs (`VillagerTrade.CODEC`,
//! `TradeSet.CODEC`, `TradeCost.CODEC`, `ItemStackTemplate.MAP_CODEC`):
//! `wants.count = 1`, `gives.count = 1`, `max_uses = 4`, `xp = 1`,
//! `reputation_discount = 0.0`, `allow_duplicates = false`.

use heck::ToShoutySnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::datapack::{quote_id, read_entries, strip_tag};

/// Parses a trade `NumberProvider`: bare constant, `uniform`, `binomial`,
/// or `add` (26.3 renamed `sum`/`summands` to `add`/`inputs`).
fn generate_number_provider(value: &serde_json::Value) -> TokenStream {
    if let Some(number) = value.as_f64() {
        let number = number as f32;
        return quote! { TradeNumberProvider::Constant(#number) };
    }
    let object = value
        .as_object()
        .unwrap_or_else(|| panic!("trade number provider must be a number or object: {value}"));
    let kind = object
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("trade number provider is missing type: {value}"));
    match kind {
        "minecraft:uniform" => {
            let min = generate_number_provider(
                object
                    .get("min")
                    .unwrap_or_else(|| panic!("uniform trade provider is missing min: {value}")),
            );
            let max = generate_number_provider(
                object
                    .get("max")
                    .unwrap_or_else(|| panic!("uniform trade provider is missing max: {value}")),
            );
            quote! {
                TradeNumberProvider::Uniform {
                    min: Box::new(#min),
                    max: Box::new(#max),
                }
            }
        }
        "minecraft:binomial" => {
            let n = generate_number_provider(
                object
                    .get("n")
                    .unwrap_or_else(|| panic!("binomial trade provider is missing n: {value}")),
            );
            let p = generate_number_provider(
                object
                    .get("p")
                    .unwrap_or_else(|| panic!("binomial trade provider is missing p: {value}")),
            );
            quote! {
                TradeNumberProvider::Binomial {
                    n: Box::new(#n),
                    p: Box::new(#p),
                }
            }
        }
        "minecraft:add" => {
            let inputs = object
                .get("inputs")
                .and_then(serde_json::Value::as_array)
                .unwrap_or_else(|| panic!("add trade provider is missing inputs: {value}"))
                .iter()
                .map(generate_number_provider);
            quote! { TradeNumberProvider::Add(vec![#(#inputs),*]) }
        }
        other => panic!("unsupported trade number provider type: {other} in {value}"),
    }
}

fn generate_cost(name: &str, side: &str, json: &serde_json::Value) -> TokenStream {
    let item = quote_id(
        json.get("id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("trade {name} {side} is missing id")),
    );
    let count = match json.get("count") {
        None => quote! { TradeNumberProvider::Constant(1.0) },
        Some(value) => generate_number_provider(value),
    };
    let potion = match json.get("components") {
        None => quote! { None },
        Some(components) => {
            let map = components
                .as_object()
                .unwrap_or_else(|| panic!("trade {name} {side} components must be an object"));
            assert_eq!(
                map.len(),
                1,
                "trade {name} {side} has unexpected components: {components}"
            );
            let contents = map.get("minecraft:potion_contents").unwrap_or_else(|| {
                panic!("trade {name} {side} has unsupported components: {components}")
            });
            let potion = quote_id(
                contents
                    .get("potion")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_else(|| {
                        panic!("trade {name} {side} potion_contents is missing potion")
                    }),
            );
            quote! { Some(#potion) }
        }
    };
    quote! {
        TradeCostSpec { item: #item, count: #count, potion: #potion }
    }
}

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
        .unwrap_or_else(|| panic!("enchantment holder set must be a string or list: {holder}"));
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

/// Parses a potion holder set (`#tag`, single id, or id list).
fn generate_potion_set(holder: &serde_json::Value) -> TokenStream {
    if let Some(single) = holder.as_str() {
        if single.starts_with('#') {
            let tag = quote_id(strip_tag(single));
            return quote! { TradeHolderSet::Tag(#tag) };
        }
        let id = quote_id(single);
        return quote! { TradeHolderSet::List(vec![#id]) };
    }
    let list = holder
        .as_array()
        .unwrap_or_else(|| panic!("potion holder set must be a string or list: {holder}"));
    let ids = list.iter().map(|entry| {
        let id = entry
            .as_str()
            .unwrap_or_else(|| panic!("potion entry must be a string: {entry}"));
        assert!(
            !id.starts_with('#'),
            "tag references cannot appear inside potion lists: {id}"
        );
        quote_id(id)
    });
    quote! { TradeHolderSet::List(vec![#(#ids),*]) }
}

fn generate_modifier(name: &str, json: &serde_json::Value) -> TokenStream {
    let kind = json
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("trade {name} modifier is missing type: {json}"));
    match kind {
        "minecraft:enchant_randomly" => {
            let options = generate_enchantment_set(
                json.get("options")
                    .unwrap_or_else(|| panic!("trade {name} enchant_randomly is missing options")),
            );
            let only_compatible = json
                .get("only_compatible")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true);
            let include_additional_cost_component = json
                .get("include_additional_cost_component")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            quote! {
                TradeItemModifier::EnchantRandomly {
                    options: #options,
                    only_compatible: #only_compatible,
                    include_additional_cost_component: #include_additional_cost_component,
                }
            }
        }
        "minecraft:enchant_with_levels" => {
            let levels =
                generate_number_provider(json.get("levels").unwrap_or_else(|| {
                    panic!("trade {name} enchant_with_levels is missing levels")
                }));
            let options = match json.get("options") {
                None => quote! { None },
                Some(holder) => {
                    let set = generate_enchantment_set(holder);
                    quote! { Some(#set) }
                }
            };
            let include_additional_cost_component = json
                .get("include_additional_cost_component")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            quote! {
                TradeItemModifier::EnchantWithLevels {
                    options: #options,
                    levels: #levels,
                    include_additional_cost_component: #include_additional_cost_component,
                }
            }
        }
        "minecraft:exploration_map" => {
            let destination = json
                .get("destination")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_else(|| panic!("trade {name} exploration_map is missing destination"));
            assert!(
                destination.starts_with('#'),
                "trade {name} exploration_map destination must be a structure tag: {destination}"
            );
            let destination = quote_id(strip_tag(destination));
            let decoration = quote_id(
                json.get("decoration")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("minecraft:mansion"),
            );
            let zoom = json
                .get("zoom")
                .and_then(serde_json::Value::as_i64)
                .and_then(|v| i32::try_from(v).ok())
                .unwrap_or(2);
            let search_radius = json
                .get("search_radius")
                .and_then(serde_json::Value::as_i64)
                .and_then(|v| i32::try_from(v).ok())
                .unwrap_or(50);
            let skip_existing_chunks = json
                .get("skip_existing_chunks")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true);
            quote! {
                TradeItemModifier::ExplorationMap {
                    destination: #destination,
                    decoration: #decoration,
                    zoom: #zoom,
                    search_radius: #search_radius,
                    skip_existing_chunks: #skip_existing_chunks,
                }
            }
        }
        "minecraft:set_stew_effect" => {
            let effects = json
                .get("effects")
                .and_then(serde_json::Value::as_array)
                .map_or_else(Vec::new, |list| {
                    list.iter()
                        .map(|entry| {
                            let effect = quote_id(
                                entry
                                    .get("type")
                                    .and_then(serde_json::Value::as_str)
                                    .unwrap_or_else(|| {
                                        panic!("trade {name} stew effect is missing type")
                                    }),
                            );
                            let duration = entry
                                .get("duration")
                                .and_then(serde_json::Value::as_i64)
                                .and_then(|v| i32::try_from(v).ok())
                                .unwrap_or_else(|| {
                                    panic!("trade {name} stew effect is missing duration")
                                });
                            quote! { (#effect, #duration) }
                        })
                        .collect::<Vec<_>>()
                });
            quote! { TradeItemModifier::SetStewEffect { effects: vec![#(#effects),*] } }
        }
        "minecraft:set_potion" => {
            let potion = quote_id(
                json.get("id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_else(|| panic!("trade {name} set_potion is missing id")),
            );
            quote! { TradeItemModifier::SetPotion { potion: #potion } }
        }
        "minecraft:set_random_potion" => {
            let options = match json.get("options") {
                None => quote! { None },
                Some(holder) => {
                    let set = generate_potion_set(holder);
                    quote! { Some(#set) }
                }
            };
            quote! { TradeItemModifier::SetRandomPotion { options: #options } }
        }
        "minecraft:set_random_dyes" => {
            let number_of_dyes =
                generate_number_provider(json.get("number_of_dyes").unwrap_or_else(|| {
                    panic!("trade {name} set_random_dyes is missing number_of_dyes")
                }));
            quote! { TradeItemModifier::SetRandomDyes { number_of_dyes: #number_of_dyes } }
        }
        "minecraft:filtered" => {
            let filter = json
                .get("item_filter")
                .and_then(serde_json::Value::as_object)
                .unwrap_or_else(|| panic!("trade {name} filtered is missing item_filter"));
            let predicates = filter
                .get("predicates")
                .and_then(serde_json::Value::as_object)
                .unwrap_or_else(|| panic!("trade {name} filtered is missing filter predicates"));
            assert_eq!(
                predicates.len(),
                1,
                "trade {name} filtered has unexpected predicates: {predicates:?}"
            );
            let guard = match predicates.keys().next().map(String::as_str) {
                Some("minecraft:enchantments") => quote! { TradeItemModifier::RequireEnchantments },
                Some("minecraft:stored_enchantments") => {
                    quote! { TradeItemModifier::RequireStoredEnchantments }
                }
                Some("minecraft:map_id") => quote! { TradeItemModifier::RequireMapId },
                Some("minecraft:dyed_color") => quote! { TradeItemModifier::RequireDyedColor },
                other => panic!("trade {name} filtered has unsupported predicate: {other:?}"),
            };
            let on_fail = json
                .get("on_fail")
                .and_then(|v| v.get("type"))
                .and_then(serde_json::Value::as_str);
            assert_eq!(
                on_fail,
                Some("minecraft:discard"),
                "trade {name} filtered must discard on fail"
            );
            assert!(
                json.get("on_pass").is_none(),
                "trade {name} filtered must not set on_pass"
            );
            guard
        }
        other => panic!("trade {name} has unsupported modifier type: {other}"),
    }
}

fn generate_trade(name: &str, json: &serde_json::Value, key: &TokenStream) -> TokenStream {
    let wants = generate_cost(
        name,
        "wants",
        json.get("wants")
            .unwrap_or_else(|| panic!("trade {name} is missing wants")),
    );
    let additional_wants = match json.get("additional_wants") {
        None => quote! { None },
        Some(value) => {
            let cost = generate_cost(name, "additional_wants", value);
            quote! { Some(#cost) }
        }
    };
    let gives = json
        .get("gives")
        .unwrap_or_else(|| panic!("trade {name} is missing gives"));
    let gives_item = quote_id(
        gives
            .get("id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("trade {name} gives is missing id")),
    );
    let gives_count = gives
        .get("count")
        .and_then(serde_json::Value::as_i64)
        .and_then(|v| i32::try_from(v).ok())
        .unwrap_or(1);
    assert!(
        gives.get("components").is_none(),
        "trade {name} gives must not set components"
    );
    let max_uses = match json.get("max_uses") {
        None => quote! { TradeNumberProvider::Constant(4.0) },
        Some(value) => generate_number_provider(value),
    };
    let xp = match json.get("xp") {
        None => quote! { TradeNumberProvider::Constant(1.0) },
        Some(value) => generate_number_provider(value),
    };
    let reputation_discount = match json.get("reputation_discount") {
        None => quote! { TradeNumberProvider::Constant(0.0) },
        Some(value) => generate_number_provider(value),
    };
    let merchant_villager_variants = match json.get("merchant_predicate") {
        None => Vec::new(),
        Some(predicate) => {
            assert_eq!(
                predicate.get("type").and_then(serde_json::Value::as_str),
                Some("minecraft:entity_properties"),
                "trade {name} merchant_predicate must be entity_properties"
            );
            // Vanilla accepts a single variant id or a list (6 trades use the
            // bare-string form: cartographer banners, fisherman boats).
            let variant_value = predicate
                .get("predicate")
                .and_then(|v| v.get("minecraft:predicates"))
                .and_then(|v| v.get("minecraft:villager/variant"))
                .unwrap_or_else(|| {
                    panic!("trade {name} merchant_predicate must check villager/variant")
                });
            let quote_variant =
                |entry: &serde_json::Value| {
                    quote_id(entry.as_str().unwrap_or_else(|| {
                        panic!("trade {name} villager variant must be a string")
                    }))
                };
            match variant_value {
                serde_json::Value::String(_) => vec![quote_variant(variant_value)],
                serde_json::Value::Array(list) => list.iter().map(quote_variant).collect(),
                other => panic!("trade {name} villager variant must be a string or list: {other}"),
            }
        }
    };
    let given_item_modifiers = match json.get("given_item_modifier") {
        None => Vec::new(),
        Some(value) => {
            if value.is_object() {
                vec![generate_modifier(name, value)]
            } else if let Some(list) = value.as_array() {
                list.iter()
                    .map(|entry| generate_modifier(name, entry))
                    .collect()
            } else {
                panic!("trade {name} given_item_modifier must be an object or list")
            }
        }
    };
    let double_trade_price_enchantments = match json.get("double_trade_price_enchantments") {
        None => quote! { None },
        Some(holder) => {
            let set = generate_enchantment_set(holder);
            quote! { Some(#set) }
        }
    };

    quote! {
        VillagerTrade {
            key: #key,
            wants: #wants,
            additional_wants: #additional_wants,
            gives: TradeResultSpec { item: #gives_item, count: #gives_count },
            max_uses: #max_uses,
            xp: #xp,
            reputation_discount: #reputation_discount,
            merchant_villager_variants: vec![#(#merchant_villager_variants),*],
            given_item_modifiers: vec![#(#given_item_modifiers),*],
            double_trade_price_enchantments: #double_trade_price_enchantments,
        }
    }
}

pub(crate) fn build_trades() -> TokenStream {
    let entries = read_entries("villager_trade");
    assert_eq!(
        entries.len(),
        391,
        "expected 391 vanilla villager_trade files, got {}",
        entries.len()
    );

    let mut stream = TokenStream::new();
    stream.extend(quote! {
        use std::sync::LazyLock;

        use steel_utils::Identifier;

        use crate::enchantment_provider::EnchantmentProviderSet;
        use crate::villager_trade::{
            TradeCostSpec, TradeHolderSet, TradeItemModifier, TradeNumberProvider,
            TradeResultSpec, VillagerTrade, VillagerTradeRegistry,
        };
    });

    let mut register_stream = TokenStream::new();
    for (name, json) in &entries {
        let ident = Ident::new(
            &name.replace('/', "_").to_shouty_snake_case(),
            Span::call_site(),
        );
        let key = quote_id(&format!("minecraft:{name}"));
        let trade = generate_trade(name, json, &key);
        stream.extend(quote! {
            pub static #ident: LazyLock<VillagerTrade> = LazyLock::new(|| #trade);
        });
        register_stream.extend(quote! {
            registry.register(&#ident);
        });
    }

    stream.extend(quote! {
        pub fn register_villager_trades(registry: &mut VillagerTradeRegistry) {
            #register_stream
        }
    });
    stream
}

pub(crate) fn build_sets() -> TokenStream {
    let entries = read_entries("trade_set");
    assert_eq!(
        entries.len(),
        68,
        "expected 68 vanilla trade_set files, got {}",
        entries.len()
    );

    let mut stream = TokenStream::new();
    stream.extend(quote! {
        use std::sync::LazyLock;

        use steel_utils::Identifier;

        use crate::villager_trade::{
            TradeNumberProvider, TradeSet, TradeSetEntries, TradeSetRegistry,
        };
    });

    let mut register_stream = TokenStream::new();
    for (name, json) in &entries {
        let ident = Ident::new(
            &name.replace('/', "_").to_shouty_snake_case(),
            Span::call_site(),
        );
        let key = quote_id(&format!("minecraft:{name}"));
        let holder = json
            .get("trades")
            .unwrap_or_else(|| panic!("trade_set {name} is missing trades"));
        let trades = if let Some(single) = holder.as_str() {
            if single.starts_with('#') {
                let tag = quote_id(strip_tag(single));
                quote! { TradeSetEntries::Tag(#tag) }
            } else {
                let id = quote_id(single);
                quote! { TradeSetEntries::Direct(vec![#id]) }
            }
        } else if let Some(list) = holder.as_array() {
            let ids = list.iter().map(|entry| {
                let id = entry.as_str().unwrap_or_else(|| {
                    panic!("trade_set {name} trades entry must be a string: {entry}")
                });
                assert!(
                    !id.starts_with('#'),
                    "tag references cannot appear inside trade_set trades lists: {id}"
                );
                quote_id(id)
            });
            quote! { TradeSetEntries::Direct(vec![#(#ids),*]) }
        } else {
            panic!("trade_set {name} trades must be a string or list")
        };
        let amount = generate_number_provider(
            json.get("amount")
                .unwrap_or_else(|| panic!("trade_set {name} is missing amount")),
        );
        let allow_duplicates = json
            .get("allow_duplicates")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let random_sequence = if let Some(id) = json.get("random_sequence").and_then(serde_json::Value::as_str) {
                let quoted = quote_id(id);
                quote! { Some(#quoted) }
            } else {
                quote! { None }
            };
        stream.extend(quote! {
            pub static #ident: LazyLock<TradeSet> = LazyLock::new(|| TradeSet {
                key: #key,
                trades: #trades,
                amount: #amount,
                allow_duplicates: #allow_duplicates,
                random_sequence: #random_sequence,
            });
        });
        register_stream.extend(quote! {
            registry.register(&#ident);
        });
    }

    stream.extend(quote! {
        pub fn register_trade_sets(registry: &mut TradeSetRegistry) {
            #register_stream
        }
    });
    stream
}
