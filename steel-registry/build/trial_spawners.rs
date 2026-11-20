//! Build-time codegen for the `minecraft:trial_spawner` registry (28/28).
//!
//! Defaults mirror `TrialSpawnerConfig.Builder`: `spawn_range = 4`,
//! `total = 6.0`, `simultaneous = 2.0`, `+2.0/+1.0` per player,
//! `ticks = 40`, eject `[consumables, key]`, ominous drops
//! `spawners/trial_chamber/items_to_drop_when_ominous`.

use heck::ToShoutySnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::datapack::{quote_id, read_entries};

fn generate_entity_tag_value(key: &str, value: &serde_json::Value) -> TokenStream {
    if key == "IsBaby" {
        if let Some(number) = value.as_i64() {
            let flag = number != 0;
            return quote! { TrialEntityTagValue::Byte(#flag) };
        }
        if let Some(flag) = value.as_bool() {
            return quote! { TrialEntityTagValue::Byte(#flag) };
        }
        panic!("IsBaby entity tag must be a bool or int: {value}");
    }
    if let Some(number) = value.as_i64().and_then(|v| i32::try_from(v).ok()) {
        quote! { TrialEntityTagValue::Int(#number) }
    } else if let Some(number) = value.as_f64() {
        let number = number as f32;
        quote! { TrialEntityTagValue::Float(#number) }
    } else if let Some(text) = value.as_str() {
        quote! { TrialEntityTagValue::Text(#text.to_string()) }
    } else {
        panic!("unsupported trial_spawner entity tag value: {value}")
    }
}

fn generate_potential(name: &str, json: &serde_json::Value) -> TokenStream {
    let data = json
        .get("data")
        .unwrap_or_else(|| panic!("trial_spawner {name} potential is missing data"));
    let entity = data
        .get("entity")
        .and_then(serde_json::Value::as_object)
        .unwrap_or_else(|| panic!("trial_spawner {name} potential is missing entity"));
    let entity_id = quote_id(
        entity
            .get("id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("trial_spawner {name} entity is missing id")),
    );
    let extra_entity_tags = entity
        .iter()
        .filter(|(key, _)| *key != "id")
        .map(|(key, value)| {
            let tag_value = generate_entity_tag_value(key, value);
            quote! { (#key.to_string(), #tag_value) }
        });
    let equipment = match data.get("equipment") {
        None => quote! { None },
        Some(value) => {
            let loot_table = quote_id(
                value
                    .get("loot_table")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_else(|| {
                        panic!("trial_spawner {name} equipment is missing loot_table")
                    }),
            );
            let slot_drop_chances = value
                .get("slot_drop_chances")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or_else(|| {
                    panic!("trial_spawner {name} equipment is missing slot_drop_chances")
                }) as f32;
            quote! {
                Some(TrialSpawnEquipment {
                    loot_table: #loot_table,
                    slot_drop_chances: #slot_drop_chances,
                })
            }
        }
    };
    let weight = json
        .get("weight")
        .and_then(serde_json::Value::as_i64)
        .and_then(|v| i32::try_from(v).ok())
        .unwrap_or_else(|| panic!("trial_spawner {name} potential is missing weight"));
    quote! {
        TrialSpawnPotential {
            entity_id: #entity_id,
            weight: #weight,
            equipment: #equipment,
            extra_entity_tags: vec![#(#extra_entity_tags),*],
        }
    }
}

fn generate_config(name: &str, json: &serde_json::Value, key: &TokenStream) -> TokenStream {
    let spawn_range = json
        .get("spawn_range")
        .and_then(serde_json::Value::as_i64)
        .and_then(|v| i32::try_from(v).ok())
        .unwrap_or(4);
    let total_mobs = json
        .get("total_mobs")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(6.0) as f32;
    let simultaneous_mobs = json
        .get("simultaneous_mobs")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(2.0) as f32;
    let total_mobs_added_per_player = json
        .get("total_mobs_added_per_player")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(2.0) as f32;
    let simultaneous_mobs_added_per_player = json
        .get("simultaneous_mobs_added_per_player")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(1.0) as f32;
    let ticks_between_spawn = json
        .get("ticks_between_spawn")
        .and_then(serde_json::Value::as_i64)
        .and_then(|v| i32::try_from(v).ok())
        .unwrap_or(40);
    let spawn_potentials = json
        .get("spawn_potentials")
        .and_then(serde_json::Value::as_array)
        .map_or_else(Vec::new, |list| {
            list.iter()
                .map(|entry| generate_potential(name, entry))
                .collect::<Vec<_>>()
        });
    let loot_tables_to_eject = match json.get("loot_tables_to_eject") {
        None => {
            let consumables = quote_id("minecraft:spawners/trial_chamber/consumables");
            let key = quote_id("minecraft:spawners/trial_chamber/key");
            quote! {
                vec![
                    TrialWeightedLootTable { loot_table: #consumables, weight: 1 },
                    TrialWeightedLootTable { loot_table: #key, weight: 1 },
                ]
            }
        }
        Some(value) => {
            let entries = value
                .as_array()
                .unwrap_or_else(|| {
                    panic!("trial_spawner {name} loot_tables_to_eject must be a list")
                })
                .iter()
                .map(|entry| {
                    let loot_table = quote_id(
                        entry
                            .get("data")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or_else(|| {
                                panic!("trial_spawner {name} loot entry is missing data")
                            }),
                    );
                    let weight = entry
                        .get("weight")
                        .and_then(serde_json::Value::as_i64)
                        .and_then(|v| i32::try_from(v).ok())
                        .unwrap_or_else(|| {
                            panic!("trial_spawner {name} loot entry is missing weight")
                        });
                    quote! {
                        TrialWeightedLootTable { loot_table: #loot_table, weight: #weight }
                    }
                });
            quote! { vec![#(#entries),*] }
        }
    };
    let items_to_drop_when_ominous = quote_id(
        json.get("items_to_drop_when_ominous")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("minecraft:spawners/trial_chamber/items_to_drop_when_ominous"),
    );

    quote! {
        TrialSpawnerConfig {
            key: #key,
            spawn_range: #spawn_range,
            total_mobs: #total_mobs,
            simultaneous_mobs: #simultaneous_mobs,
            total_mobs_added_per_player: #total_mobs_added_per_player,
            simultaneous_mobs_added_per_player: #simultaneous_mobs_added_per_player,
            ticks_between_spawn: #ticks_between_spawn,
            spawn_potentials: vec![#(#spawn_potentials),*],
            loot_tables_to_eject: #loot_tables_to_eject,
            items_to_drop_when_ominous: #items_to_drop_when_ominous,
        }
    }
}

pub(crate) fn build() -> TokenStream {
    let entries = read_entries("trial_spawner");
    assert_eq!(
        entries.len(),
        28,
        "expected 28 vanilla trial_spawner files, got {}",
        entries.len()
    );

    let mut stream = TokenStream::new();
    stream.extend(quote! {
        use std::sync::LazyLock;

        use steel_utils::Identifier;

        use crate::trial_spawner::{
            TrialEntityTagValue, TrialSpawnEquipment, TrialSpawnPotential, TrialSpawnerConfig,
            TrialSpawnerConfigRegistry, TrialWeightedLootTable,
        };
    });

    let mut register_stream = TokenStream::new();
    for (name, json) in &entries {
        let ident = Ident::new(
            &name.replace('/', "_").to_shouty_snake_case(),
            Span::call_site(),
        );
        let key = quote_id(&format!("minecraft:{name}"));
        let config = generate_config(name, json, &key);
        stream.extend(quote! {
            pub static #ident: LazyLock<TrialSpawnerConfig> = LazyLock::new(|| #config);
        });
        register_stream.extend(quote! {
            registry.register(&#ident);
        });
    }

    stream.extend(quote! {
        pub fn register_trial_spawners(registry: &mut TrialSpawnerConfigRegistry) {
            #register_stream
        }
    });
    stream
}
