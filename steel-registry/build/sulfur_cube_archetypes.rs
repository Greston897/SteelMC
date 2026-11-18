#![expect(
    clippy::unwrap_used,
    reason = "build script must fail immediately on invalid extracted archetype data"
)]

//! Build-time codegen for the `minecraft:sulfur_cube_archetype` registry (12/12).

use heck::ToShoutySnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use steel_utils::value_providers::FloatProvider;

use crate::datapack::{quote_id, read_entries};
use crate::features::generate_float_provider;

fn generate_item_set(items: &serde_json::Value) -> TokenStream {
    if let Some(tag) = items.as_str() {
        if let Some(stripped) = tag.strip_prefix('#') {
            let tag_id = quote_id(stripped);
            return quote! { SulfurCubeItemSet::Tag(#tag_id) };
        }
        let item = quote_id(tag);
        return quote! { SulfurCubeItemSet::Items(vec![#item]) };
    }
    let list = items
        .as_array()
        . unwrap_or_else(|| panic!("sulfur items must be a string or list: {items}"));
    let ids = list.iter().map(|entry| {
        let id = entry
            .as_str()
            .unwrap_or_else(|| panic!("sulfur item entry must be a string: {entry}"));
        assert!(
            !id.starts_with('#'),
            "tag references cannot appear inside sulfur items lists: {id}"
        );
        quote_id(id)
    });
    quote! { SulfurCubeItemSet::Items(vec![#(#ids),*]) }
}

fn generate_operation(operation: &str) -> TokenStream {
    match operation {
        "add_value" => quote! { AttributeModifierOperation::AddValue },
        "add_multiplied_base" => quote! { AttributeModifierOperation::AddMultipliedBase },
        "add_multiplied_total" => quote! { AttributeModifierOperation::AddMultipliedTotal },
        other => panic!("unknown sulfur attribute operation: {other}"),
    }
}

fn generate_archetype_value(name: &str, json: &serde_json::Value) -> TokenStream {
    let object = json
        .as_object()
        .unwrap_or_else(|| panic!("sulfur archetype {name} must be an object"));
    let items = generate_item_set(
        object
            .get("items")
            .unwrap_or_else(|| panic!("sulfur archetype {name} is missing items")),
    );
    let modifiers = object
        .get("attribute_modifiers")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("sulfur archetype {name} is missing attribute_modifiers"))
        .iter()
        .map(|entry| {
            let amount = entry
                .get("amount")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or_else(|| panic!("sulfur modifier in {name} is missing amount: {entry}"));
            let attribute = quote_id(
                entry
                    .get("attribute")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_else(|| panic!("sulfur modifier in {name} is missing attribute")),
            );
            let id = quote_id(
                entry
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_else(|| panic!("sulfur modifier in {name} is missing id")),
            );
            let operation = generate_operation(
                entry
                    .get("operation")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_else(|| panic!("sulfur modifier in {name} is missing operation")),
            );
            quote! {
                SulfurCubeAttributeModifier {
                    attribute: #attribute,
                    id: #id,
                    amount: #amount,
                    operation: #operation,
                }
            }
        });
    let buoyant = object
        .get("buoyant")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let explosion = match object.get("explosion") {
        None => quote! { None },
        Some(value) => {
            let power = value
                .get("power")
                .and_then(serde_json::Value::as_i64)
                .and_then(|v| i32::try_from(v).ok())
                .unwrap_or_else(|| panic!("sulfur explosion in {name} is missing power"));
            let causes_fire = value
                .get("causes_fire")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or_else(|| panic!("sulfur explosion in {name} is missing causes_fire"));
            let fuse = value
                .get("fuse")
                .and_then(serde_json::Value::as_i64)
                .and_then(|v| i32::try_from(v).ok())
                .unwrap_or_else(|| panic!("sulfur explosion in {name} is missing fuse"));
            quote! {
                Some(SulfurCubeExplosion { fuse: #fuse, power: #power, causes_fire: #causes_fire })
            }
        }
    };
    let contact_damage = match object.get("contact_damage") {
        None => quote! { None },
        Some(value) => {
            let damage_type = quote_id(
                value
                    .get("damage_type")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_else(|| {
                        panic!("sulfur contact_damage in {name} is missing damage_type")
                    }),
            );
            let amount: FloatProvider = serde_json::from_value(
                value
                    .get("amount")
                    .cloned()
                    .unwrap_or_else(|| {
                        panic!("sulfur contact_damage in {name} is missing amount")
                    }),
            )
            .unwrap_or_else(|e| panic!("sulfur contact_damage amount in {name}: {e}"));
            let amount = generate_float_provider(amount);
            let attribute_to_source = value
                .get("attribute_to_source")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or_else(|| {
                    panic!("sulfur contact_damage in {name} is missing attribute_to_source")
                });
            quote! {
                Some(SulfurCubeContactDamage {
                    damage_type: #damage_type,
                    amount: #amount,
                    attribute_to_source: #attribute_to_source,
                })
            }
        }
    };
    let knockback = object
        .get("knockback_modifiers")
        .unwrap_or_else(|| panic!("sulfur archetype {name} is missing knockback_modifiers"));
    let horizontal_power = knockback
        .get("horizontal_power")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or_else(|| panic!("sulfur knockback in {name} is missing horizontal_power"))
        as f32;
    let vertical_power = knockback
        .get("vertical_power")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or_else(|| panic!("sulfur knockback in {name} is missing vertical_power"))
        as f32;
    let sound = object
        .get("sound_settings")
        .unwrap_or_else(|| panic!("sulfur archetype {name} is missing sound_settings"));
    let sound_setting = |key: &str| {
        quote_id(
            sound
                .get(key)
                .and_then(serde_json::Value::as_str)
                .unwrap_or_else(|| panic!("sulfur sound_settings in {name} is missing {key}")),
        )
    };
    let hit_sound = sound_setting("hit_sound");
    let push_sound = sound_setting("push_sound");
    let impulse_threshold = sound
        .get("push_sound_impulse_threshold")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or_else(|| panic!("sulfur sound_settings in {name} is missing impulse threshold"))
        as f32;
    let cooldown = sound
        .get("push_sound_cooldown")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or_else(|| panic!("sulfur sound_settings in {name} is missing cooldown"))
        as f32;

    quote! {
        SulfurCubeArchetypeValue {
            items: #items,
            buoyant: #buoyant,
            explosion: #explosion,
            contact_damage: #contact_damage,
            knockback_modifiers: SulfurCubeKnockbackModifiers {
                horizontal_power: #horizontal_power,
                vertical_power: #vertical_power,
            },
            attribute_modifiers: vec![#(#modifiers),*],
            sound_settings: SulfurCubeSoundSettings {
                hit_sound: #hit_sound,
                push_sound: #push_sound,
                push_sound_impulse_threshold: #impulse_threshold,
                push_sound_cooldown: #cooldown,
            },
        }
    }
}

pub(crate) fn build() -> TokenStream {
    let entries = read_entries("sulfur_cube_archetype");
    assert_eq!(
        entries.len(),
        12,
        "expected 12 vanilla sulfur_cube_archetype files, got {}",
        entries.len()
    );

    let mut stream = TokenStream::new();
    stream.extend(quote! {
        use std::sync::LazyLock;

        use steel_utils::Identifier;
        use steel_utils::value_providers::FloatProvider;

        use crate::attribute::AttributeModifierOperation;
        use crate::sulfur_cube_archetype::{
            SulfurCubeArchetype, SulfurCubeArchetypeRegistry, SulfurCubeArchetypeValue,
            SulfurCubeAttributeModifier, SulfurCubeContactDamage, SulfurCubeExplosion,
            SulfurCubeItemSet, SulfurCubeKnockbackModifiers, SulfurCubeSoundSettings,
        };
    });

    let mut register_stream = TokenStream::new();
    for (name, json) in &entries {
        let ident = Ident::new(&name.to_shouty_snake_case(), Span::call_site());
        let key = quote_id(&format!("minecraft:{name}"));
        let value = generate_archetype_value(name, json);
        stream.extend(quote! {
            pub static #ident: LazyLock<SulfurCubeArchetype> = LazyLock::new(|| SulfurCubeArchetype {
                key: #key,
                value: #value,
            });
        });
        register_stream.extend(quote! {
            registry.register(&#ident);
        });
    }

    stream.extend(quote! {
        pub fn register_sulfur_cube_archetypes(registry: &mut SulfurCubeArchetypeRegistry) {
            #register_stream
        }
    });
    stream
}
