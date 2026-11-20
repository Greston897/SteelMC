//! Shared helpers for datapack-registry codegen (`sulfur_cube_archetype`,
//! `test_environment`, `predicate`, `context_*_provider`, `trial_spawner`,
//! `villager_trade`, `dimension`, worldgen presets, ...).
//!
//! All readers walk `steel-utils/build_assets/builtin_datapacks/minecraft/`
//! (extracted from the target vanilla server jar) recursively and return
//! `(resource_name, json)` pairs sorted by resource name, where the resource
//! name is the slash-separated path relative to the registry dir
//! (e.g. `trial_chamber/melee/husk/normal`).

use std::{fs, path::Path};

use proc_macro2::TokenStream;
use quote::quote;

/// Reads every `.json` file under a builtin-datapack registry dir, recursively.
pub fn read_entries(subdir: &str) -> Vec<(String, serde_json::Value)> {
    let dir = format!("../steel-utils/build_assets/builtin_datapacks/minecraft/{subdir}");
    println!("cargo:rerun-if-changed={dir}/");

    fn walk(dir: &Path, base: &Path, out: &mut Vec<(String, serde_json::Value)>) {
        let entries =
            fs::read_dir(dir).unwrap_or_else(|e| panic!("Failed to read {}: {e}", dir.display()));
        for entry in entries {
            let path = entry
                .unwrap_or_else(|e| panic!("Failed to read entry in {}: {e}", dir.display()))
                .path();
            if path.is_dir() {
                walk(&path, base, out);
            } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let name = path
                    .with_extension("")
                    .strip_prefix(base)
                    .unwrap_or_else(|_| {
                        panic!("{} is not under {}", path.display(), base.display())
                    })
                    .to_str()
                    .unwrap_or_else(|| panic!("Invalid file name: {}", path.display()))
                    .replace('\\', "/");
                let content = fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
                let value: serde_json::Value = serde_json::from_str(&content)
                    .unwrap_or_else(|e| panic!("Failed to parse {name}: {e}"));
                out.push((name, value));
            }
        }
    }

    let base = Path::new(&dir);
    let mut out = Vec::new();
    walk(base, base, &mut out);
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Reads one JSON file under `builtin_datapacks/minecraft/`.
pub fn read_json_file(relative_path: &str) -> serde_json::Value {
    let path = format!("../steel-utils/build_assets/builtin_datapacks/minecraft/{relative_path}");
    println!("cargo:rerun-if-changed={path}");
    let content =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path}: {e}"));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {relative_path}: {e}"))
}

/// Emits an `Identifier` literal for a (possibly namespaced) id string.
///
/// Vanilla datapack ids always use the `minecraft` namespace; anything else is
/// emitted via `Identifier::new_static`.
pub fn quote_id(id: &str) -> TokenStream {
    if let Some(path) = id.strip_prefix("minecraft:") {
        quote! { Identifier::vanilla_static(#path) }
    } else if let Some((namespace, path)) = id.split_once(':') {
        quote! { Identifier::new_static(#namespace, #path) }
    } else {
        quote! { Identifier::vanilla_static(#id) }
    }
}

/// Strips the `minecraft:` namespace prefix, panicking on foreign namespaces.
///
/// Registry keys in generated code are always vanilla ids; datapack string
/// references (`"minecraft:block/fast_cooking"`) map to keys (`block/fast_cooking`).
pub fn strip_vanilla(id: &str) -> &str {
    if let Some(path) = id.strip_prefix("minecraft:") {
        path
    } else if id.contains(':') {
        panic!("Non-vanilla registry reference is not supported by vanilla codegen: {id}")
    } else {
        id
    }
}

/// Strips a `#minecraft:` tag reference down to the tag path.
pub fn strip_tag(tag: &str) -> &str {
    let raw = tag
        .strip_prefix('#')
        .unwrap_or_else(|| panic!("Expected a #tag reference, got: {tag}"));
    strip_vanilla(raw)
}
