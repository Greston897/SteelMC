use crate::shared_structs::{
    BiomeCondition, BiomeConditionTarget, ItemStackTemplateJson, SpawnConditionEntry,
    TextComponentJson,
};
use heck::{ToShoutySnakeCase, ToUpperCamelCase};
use proc_macro2::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::Path;
use steel_utils::Identifier;

pub fn read_json_asset<T: serde::de::DeserializeOwned>(path: &str) -> T {
    println!("cargo:rerun-if-changed={path}");
    let content = fs::read_to_string(path).unwrap_or_else(|e| panic!("Failed to read {path}: {e}"));
    serde_json::from_str(&content).unwrap_or_else(|e| panic!("Failed to parse {path}: {e}"))
}

pub fn sort_contiguous_registry_entries<T>(
    entries: &mut [T],
    path: &str,
    id: impl Fn(&T) -> usize,
) {
    entries.sort_by_key(&id);
    for (expected, entry) in entries.iter().enumerate() {
        let actual = id(entry);
        assert_eq!(
            actual, expected,
            "Expected contiguous ids in {path}: entry at position {expected} has id {actual}"
        );
    }
}

pub fn generate_identifier(resource: &Identifier) -> TokenStream {
    let namespace = resource.namespace.as_ref();
    let path = resource.path.as_ref();
    quote! { Identifier { namespace: Cow::Borrowed(#namespace), path: Cow::Borrowed(#path) } }
}

pub fn generate_sound_event_ref(resource: &Identifier) -> TokenStream {
    assert_eq!(
        resource.namespace.as_ref(),
        "minecraft",
        "vanilla sound event references must use the minecraft namespace: {resource}"
    );

    let ident = Ident::new(&resource.path.to_shouty_snake_case(), Span::call_site());
    quote! { &crate::sound_events::#ident }
}

pub fn generate_option<T, F>(opt: &Option<T>, f: F) -> TokenStream
where
    F: FnOnce(&T) -> TokenStream,
{
    if let Some(val) = opt {
        let inner = f(val);
        quote! { Some(#inner) }
    } else {
        quote! { None }
    }
}

pub fn generate_vec<T, F>(vec: &[T], f: F) -> TokenStream
where
    F: Fn(&T) -> TokenStream,
{
    let items: Vec<_> = vec.iter().map(f).collect();
    quote! { vec![#(#items),*] }
}

pub fn generate_biome_condition(condition: &BiomeCondition) -> TokenStream {
    let condition_type = condition.condition_type.as_str();
    let biomes = generate_biome_condition_target(&condition.biomes);

    quote! {
        BiomeCondition {
            condition_type: #condition_type,
            biomes: #biomes,
        }
    }
}

fn generate_biome_condition_target(target: &BiomeConditionTarget) -> TokenStream {
    match target {
        BiomeConditionTarget::Tag(tag) => {
            let tag = generate_identifier(tag);
            quote! { crate::shared_structs::BiomeConditionTarget::Tag(#tag) }
        }
        BiomeConditionTarget::Direct(biome) => {
            let biome = generate_identifier(biome);
            quote! { crate::shared_structs::BiomeConditionTarget::Direct(#biome) }
        }
    }
}

pub fn generate_spawn_condition_entry(entry: &SpawnConditionEntry) -> TokenStream {
    let priority = entry.priority;
    let condition = generate_option(&entry.condition, generate_biome_condition);

    quote! {
        SpawnConditionEntry {
            priority: #priority,
            condition: #condition,
        }
    }
}
pub fn generate_text_component(component: &TextComponentJson) -> TokenStream {
    let translate = component.translate.as_str();
    let Some(color) = component.color.as_deref() else {
        return quote! {
            TextComponent::translated(TranslatedMessage::new(#translate, None))
        };
    };
    let color = generate_text_color(color);
    quote! {
        TextComponent {
            content: text_components::content::Content::Translate(
                TranslatedMessage::new(#translate, None),
            ),
            format: text_components::format::Format {
                color: Some(#color),
                ..Default::default()
            },
            children: vec![],
            interactions: text_components::interactivity::Interactivity::new(),
        }
    }
}

pub fn generate_item_stack_template(template: &ItemStackTemplateJson) -> TokenStream {
    let name = Ident::new(&template.id.path.to_shouty_snake_case(), Span::call_site());
    let count = template.count;
    if let Some(components) = &template.components {
        let patch = generate_component_patch(components, &template.id.to_string());
        return quote! {
            {
                let (components, components_hash) = #patch;
                ItemStackTemplate::from_extracted(
                    &vanilla_items::#name,
                    #count,
                    components,
                    components_hash,
                )
            }
        };
    }
    quote! {
        ItemStackTemplate::with_count(&*vanilla_items::#name,#count)
    }
}

/// Builds a `DataComponentPatch` plus its persistent hash for a build-time
/// validated template.
///
/// Templates carry one component in the common case and several for things like
/// vanilla's ominous banner, so every entry is mapped rather than assuming a
/// single one. Extracted assets only ever carry the components that have a
/// build-time codec here; anything else fails the build instead of silently
/// dropping data.
fn generate_component_patch(components: &serde_json::Value, owner: &str) -> TokenStream {
    let components = components.as_object().unwrap_or_else(|| {
        panic!("Item stack template components for {owner} are not an object: {components}")
    });
    assert!(
        !components.is_empty(),
        "Item stack template {owner} has empty components"
    );

    // One `(component type, value expression)` pair per entry, in the order the
    // asset lists them (a `serde_json` object preserves insertion order).
    let entries: Vec<_> = components
        .iter()
        .map(|(key, component)| {
            let (component_type, component_value) = match key.as_str() {
                "minecraft:suspicious_stew_effects" => (
                    quote! { SUSPICIOUS_STEW_EFFECTS },
                    suspicious_stew_effects_tokens(component),
                ),
                "minecraft:potion_contents" => (
                    quote! { POTION_CONTENTS },
                    potion_contents_tokens(component),
                ),
                "minecraft:pot_decorations" => (
                    quote! { POT_DECORATIONS },
                    pot_decorations_tokens(component),
                ),
                "minecraft:banner_patterns" => (
                    quote! { BANNER_PATTERNS },
                    banner_patterns_tokens(component),
                ),
                "minecraft:rarity" => (quote! { RARITY }, rarity_tokens(component)),
                "minecraft:tooltip_display" => (
                    quote! { TOOLTIP_DISPLAY },
                    tooltip_display_tokens(component),
                ),
                _ => panic!("Unsupported item stack template components {components:?} in {owner}"),
            };
            (component_type, component_value)
        })
        .collect();

    let names: Vec<_> = entries.iter().map(|(name, _)| name).collect();
    let values: Vec<_> = entries.iter().map(|(_, value)| value).collect();
    let entry_count = entries.len();
    quote! {{
        let mut components = DataComponentPatch::new();
        let mut hashed: Vec<(Identifier, i32)> = Vec::with_capacity(#entry_count);
        #(
            {
                let value = #values;
                hashed.push((
                    vanilla_components::#names.key().clone(),
                    steel_utils::hash::HashComponent::compute_hash(&value),
                ));
                components.set(vanilla_components::#names, value);
            }
        )*
        let components_hash = DataComponentPatch::compute_extracted_hash(&hashed);
        (components, components_hash)
    }}
}

/// A banner's pattern layers, each a pattern id plus dye colour.
fn banner_patterns_tokens(value: &serde_json::Value) -> TokenStream {
    let layers = value
        .as_array()
        .unwrap_or_else(|| panic!("Banner patterns are not an array: {value}"));
    let layers: Vec<_> = layers
        .iter()
        .map(|layer| {
            let pattern = layer
                .get("pattern")
                .and_then(|pattern| pattern.as_str())
                .unwrap_or_else(|| panic!("Banner pattern is missing an id: {layer}"));
            let pattern = vanilla_registry_ident(pattern);
            let color = layer
                .get("color")
                .and_then(|color| color.as_str())
                .unwrap_or_else(|| panic!("Banner pattern is missing a color: {layer}"));
            let color = dye_color_tokens(color);
            quote! {
                BannerPatternLayer::new(
                    RegistryHolder::reference(&vanilla_banner_patterns::#pattern),
                    #color,
                )
            }
        })
        .collect();
    quote! { BannerPatternLayers::new(vec![#(#layers),*]) }
}

/// Vanilla `Rarity`, whose serialized form is the lowercase name.
fn rarity_tokens(value: &serde_json::Value) -> TokenStream {
    let rarity = value
        .as_str()
        .unwrap_or_else(|| panic!("Rarity is not a string: {value}"));
    match rarity {
        "common" => quote! { Rarity::Common },
        "uncommon" => quote! { Rarity::Uncommon },
        "rare" => quote! { Rarity::Rare },
        "epic" => quote! { Rarity::Epic },
        _ => panic!("Unknown rarity {rarity}"),
    }
}

/// The tooltip display rules: whether the tooltip is hidden and which
/// components are suppressed inside it.
fn tooltip_display_tokens(value: &serde_json::Value) -> TokenStream {
    let display = value
        .as_object()
        .unwrap_or_else(|| panic!("Tooltip display is not an object: {value}"));
    let hide_tooltip = display
        .get("hide_tooltip")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let hidden: Vec<_> = display
        .get("hidden_components")
        .and_then(|components| components.as_array())
        .map(|components| {
            components
                .iter()
                .map(|component| {
                    let key = component
                        .as_str()
                        .unwrap_or_else(|| panic!("Hidden component is not a string: {component}"));
                    let key = key.parse::<Identifier>().unwrap_or_else(|error| {
                        panic!("Hidden component {key} is not an identifier: {error}")
                    });
                    quote! { Identifier::vanilla_static(#key) }
                })
                .collect()
        })
        .unwrap_or_default();
    quote! {{
        let display = TooltipDisplay::new(#hide_tooltip);
        #(let display = display.with_hidden_key(Identifier::vanilla_static(#hidden), true);)*
        display
    }}
}

/// Maps a dye colour name from the extracted assets to `DyeColor`.
fn dye_color_tokens(color: &str) -> TokenStream {
    let variant = Ident::new(&color.to_upper_camel_case(), Span::call_site());
    quote! { DyeColor::#variant }
}

fn suspicious_stew_effects_tokens(value: &serde_json::Value) -> TokenStream {
    let effects = value
        .as_array()
        .unwrap_or_else(|| panic!("Suspicious stew effects are not an array: {value}"));
    let effects: Vec<_> = effects
        .iter()
        .map(|effect| {
            let raw_id = effect
                .get("id")
                .and_then(|id| id.as_str())
                .unwrap_or_else(|| panic!("Stew effect is missing an id: {effect}"));
            let effect_id = vanilla_registry_ident(raw_id);
            let duration = effect
                .get("duration")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(160) as i32;
            quote! { SuspiciousStewEffect::new(&vanilla_mob_effects::#effect_id, #duration) }
        })
        .collect();
    quote! { SuspiciousStewEffects::new(vec![#(#effects),*]) }
}

fn potion_contents_tokens(value: &serde_json::Value) -> TokenStream {
    let contents = value
        .as_object()
        .unwrap_or_else(|| panic!("Potion contents are not an object: {value}"));
    assert!(
        contents.len() == 1 && contents.contains_key("potion"),
        "Unsupported extracted potion contents {contents:?}"
    );
    let potion = contents
        .get("potion")
        .and_then(|potion| potion.as_str())
        .unwrap_or_else(|| panic!("Potion contents are missing a potion: {value}"));
    let potion = vanilla_registry_ident(potion);
    quote! {
        PotionContents::new(Some(RegistryReference::new(&vanilla_potions::#potion)))
    }
}

/// One decoration side: either absent or a single item stack.
///
/// The extracted JSON names each present side, and vanilla's codec leaves absent
/// sides out entirely, so a missing key is `None` rather than a failure.
fn pot_decorations_tokens(value: &serde_json::Value) -> TokenStream {
    let sides = value
        .as_object()
        .unwrap_or_else(|| panic!("Pot decorations are not an object: {value}"));
    let side = |name: &str| {
        sides.get(name).map_or_else(
            || quote! { None },
            |stack| {
                let template = generate_item_stack_template(
                    &serde_json::from_value(stack.clone())
                        .unwrap_or_else(|error| panic!("Invalid pot decoration {name}: {error}")),
                );
                quote! { Some(#template) }
            },
        )
    };
    let (back, left, right, front) = (side("back"), side("left"), side("right"), side("front"));
    quote! {
        PotDecorations::new(#back, #left, #right, #front)
    }
}

fn generate_text_color(color: &str) -> TokenStream {
    match color {
        "black" => quote! { text_components::format::Color::Black },
        "dark_blue" => quote! { text_components::format::Color::DarkBlue },
        "dark_green" => quote! { text_components::format::Color::DarkGreen },
        "dark_aqua" => quote! { text_components::format::Color::DarkAqua },
        "dark_red" => quote! { text_components::format::Color::DarkRed },
        "dark_purple" => quote! { text_components::format::Color::DarkPurple },
        "gold" => quote! { text_components::format::Color::Gold },
        "gray" => quote! { text_components::format::Color::Gray },
        "dark_gray" => quote! { text_components::format::Color::DarkGray },
        "blue" => quote! { text_components::format::Color::Blue },
        "green" => quote! { text_components::format::Color::Green },
        "aqua" => quote! { text_components::format::Color::Aqua },
        "red" => quote! { text_components::format::Color::Red },
        "light_purple" => quote! { text_components::format::Color::LightPurple },
        "yellow" => quote! { text_components::format::Color::Yellow },
        "white" => quote! { text_components::format::Color::White },
        _ => generate_rgb_text_color(color),
    }
}

fn generate_rgb_text_color(color: &str) -> TokenStream {
    let Some(hex) = color.strip_prefix('#').filter(|hex| {
        hex.len() == 6 && hex.is_ascii() && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
    }) else {
        panic!("Unknown text color: {color}");
    };
    let red = u8::from_str_radix(&hex[0..2], 16)
        .unwrap_or_else(|_| panic!("Invalid red channel in text color: {color}"));
    let green = u8::from_str_radix(&hex[2..4], 16)
        .unwrap_or_else(|_| panic!("Invalid green channel in text color: {color}"));
    let blue = u8::from_str_radix(&hex[4..6], 16)
        .unwrap_or_else(|_| panic!("Invalid blue channel in text color: {color}"));
    quote! { text_components::format::Color::Rgb(#red, #green, #blue) }
}

pub fn read_variants_from_dir<T: DeserializeOwned>(subdir: &str) -> Vec<(String, T)> {
    let dir = format!("../steel-utils/build_assets/builtin_datapacks/minecraft/{subdir}");
    println!("cargo:rerun-if-changed={dir}/");
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("Failed to read {dir}: {e}")) {
        let path = entry
            .unwrap_or_else(|e| panic!("Failed to read entry in {dir}: {e}"))
            .path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_else(|| panic!("Invalid variant file name in {dir}: {}", path.display()))
            .to_string();
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
        let value: T = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {name}: {e}"));
        out.push((name, value));
    }
    let order = extracted_variant_order(subdir);
    out.sort_by_key(|(name, _)| {
        order
            .iter()
            .position(|ordered| ordered == name)
            .unwrap_or_else(|| panic!("Unknown vanilla {subdir} variant in extracted data: {name}"))
    });
    assert_eq!(
        out.len(),
        order.len(),
        "Expected {} vanilla {subdir} variants, got {}",
        order.len(),
        out.len()
    );
    out
}

pub(crate) trait ResourceData<T> {
    fn new_data(key: String, value: T) -> Self;

    fn name() -> &'static str;
}
pub fn read_resources<G: DeserializeOwned, T: ResourceData<G>>(dir: &Path) -> Vec<T> {
    let mut resources = Vec::new();
    read_resources_from_dir(dir, dir, &mut resources);
    resources
}

pub fn read_resources_from_dir<G: DeserializeOwned, T: ResourceData<G>>(
    root: &Path,
    dir: &Path,
    advancements: &mut Vec<T>,
) {
    let entries = fs::read_dir(dir).unwrap_or_else(|error| {
        panic!(
            "Cannot read {} directory {}: {error}",
            T::name(),
            dir.display()
        )
    });
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| {
            panic!(
                "Cannot read an entry in {} directory {}: {error}",
                T::name(),
                dir.display()
            )
        });
        let path = entry.path();
        if path.is_dir() {
            read_resources_from_dir(root, &path, advancements);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let key = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .with_extension("")
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/");
        let source = fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!("Cannot read {} {}: {error}", T::name(), path.display())
        });
        let value: G = serde_json::from_str(&source)
            .unwrap_or_else(|e| panic!("Cannot parse {} {}: {e}", T::name(), path.display()));

        advancements.push(T::new_data(key, value));
    }
}

#[derive(Deserialize)]
struct ExtractedVariantRegistryEntry {
    id: usize,
    key: String,
}

fn extracted_variant_order(subdir: &str) -> Vec<String> {
    const ASSET: &str = "build_assets/entity_variant_registries.json";

    let mut registries: FxHashMap<String, Vec<ExtractedVariantRegistryEntry>> =
        read_json_asset(ASSET);
    let mut entries = registries
        .remove(subdir)
        .unwrap_or_else(|| panic!("Missing vanilla {subdir} registry in {ASSET}"));
    sort_contiguous_registry_entries(&mut entries, ASSET, |entry| entry.id);

    entries
        .into_iter()
        .map(|entry| {
            let Some(key) = entry.key.strip_prefix("minecraft:") else {
                panic!(
                    "Expected vanilla {subdir} registry key in {ASSET}, got {}",
                    entry.key
                );
            };
            key.to_owned()
        })
        .collect()
}

/// Maps a `minecraft:` registry identifier onto its generated constant name.
fn vanilla_registry_ident(identifier: &str) -> Ident {
    let path = identifier.strip_prefix("minecraft:").unwrap_or(identifier);
    assert!(
        !path.contains('/') && !path.contains(':'),
        "Expected a vanilla registry identifier, got {identifier}"
    );
    Ident::new(&path.to_shouty_snake_case(), Span::call_site())
}
