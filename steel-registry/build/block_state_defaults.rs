//! Build-time lookup of vanilla block property names and default values.
//!
//! Vanilla's `BlockState` codecs (`StateHolder.codec` / `StateDefinition` in
//! MC 26.3) resolve every encoded state against the block's full property
//! set, so the NBT codegen needs the same metadata at build time:
//!
//! * `FULL_CODEC` always writes the **complete** sorted property map — keys
//!   missing from the source file behave as the block's defaults, and the
//!   `properties` key is only omitted for property-less (singleton) blocks.
//! * `BlockState.CODEC` collapses a state whose every value equals the
//!   default to the bare block id string.
//! * Unknown property keys are ignored on decode, so they must not be
//!   emitted either.
//!
//! The property *constants* per block and their default values come from
//! `build_assets/blocks.json` (`properties` zipped with `default_properties`,
//! values as `bool_<b>` / `int_<n>` / `enum_<Type>_<value>` with the value in
//! serialized form). The serialized property *names* behind those constants
//! live in `src/blocks/properties.rs` (`BlockStateProperties::*` first
//! constructor argument) and are scraped at build time so the two files
//! cannot drift apart silently: any constant without a scraped name fails
//! the build with an explicit message.

#![expect(
    clippy::unwrap_used,
    reason = "build script must fail immediately on invalid extracted block data"
)]

use std::collections::BTreeMap;

use rustc_hash::FxHashMap;
use std::fs;
use std::sync::OnceLock;

use serde::Deserialize;

#[derive(Deserialize)]
struct BlocksFile {
    blocks: Vec<BlockEntry>,
}

#[derive(Deserialize)]
struct BlockEntry {
    name: String,
    properties: Vec<String>,
    default_properties: Vec<String>,
}

/// Maps a `BlockStateProperties` constant (`HORIZONTAL_FACING`) to its
/// serialized name (`facing`) by scraping the first constructor argument in
/// `src/blocks/properties.rs`.
fn load_property_names() -> FxHashMap<String, String> {
    println!("cargo:rerun-if-changed=src/blocks/properties.rs");
    let source = fs::read_to_string("src/blocks/properties.rs")
        .expect("block state defaults require src/blocks/properties.rs");

    let mut names = FxHashMap::default();
    let mut lines = source.lines();
    while let Some(line) = lines.next() {
        let trimmed = line.trim_start();
        // `pub const NAME: ...` — the `: Type` guard skips `pub const fn` items.
        let Some(rest) = trimmed.strip_prefix("pub const ") else {
            continue;
        };
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        let after_name = rest[name.len()..].trim_start();
        if name.is_empty() || !after_name.starts_with(':') {
            continue;
        }

        // The serialized name is the first string literal at or after the `=`.
        let mut text = after_name.to_owned();
        loop {
            if let Some(start) = text.find('"') {
                let remainder = &text[start + 1..];
                let end = remainder
                    .find('"')
                    .unwrap_or_else(|| panic!("unterminated property name for `{name}`"));
                names.insert(name, remainder[..end].to_owned());
                break;
            }
            lines
                .next()
                .unwrap_or_else(|| panic!("missing property name for `{name}`"))
                .clone_into(&mut text);
        }
    }
    names
}

/// Extracts the serialized value from a `blocks.json` default entry
/// (`bool_<b>` / `int_<n>` / `enum_<Type>_<value>`).
fn serialized_default(value: &str) -> &str {
    if let Some(value) = value.strip_prefix("bool_") {
        return value;
    }
    if let Some(value) = value.strip_prefix("int_") {
        return value;
    }
    if let Some(rest) = value.strip_prefix("enum_") {
        // Enum type names are `UpperCamelCase` (never contain `_`), so the
        // value is everything after the first underscore (`north_east` stays
        // intact).
        let (_, value) = rest
            .split_once('_')
            .unwrap_or_else(|| panic!("malformed enum default `{value}`"));
        return value;
    }
    panic!("unknown block default value format `{value}`");
}

fn load_block_property_defaults() -> FxHashMap<String, BTreeMap<String, String>> {
    let file: BlocksFile =
        serde_json::from_str(&fs::read_to_string("build_assets/blocks.json").unwrap()).unwrap();
    let names = load_property_names();

    let mut defaults = FxHashMap::with_capacity_and_hasher(file.blocks.len(), Default::default());
    for block in &file.blocks {
        assert_eq!(
            block.properties.len(),
            block.default_properties.len(),
            "block `{}` has mismatched properties/defaults",
            block.name
        );
        let mut properties = BTreeMap::new();
        for (constant, default) in block.properties.iter().zip(block.default_properties.iter()) {
            let name = names
                .get(constant)
                .unwrap_or_else(|| panic!("unknown block property constant `{constant}`"));
            properties.insert(name.clone(), serialized_default(default).to_owned());
        }
        defaults.insert(format!("minecraft:{}", block.name), properties);
    }
    defaults
}

/// Vanilla default property values per block id (`minecraft:...`):
/// serialized property name -> serialized default value, sorted by name like
/// vanilla's `ImmutableSortedMap`.
pub fn block_property_defaults() -> &'static FxHashMap<String, BTreeMap<String, String>> {
    static CACHE: OnceLock<FxHashMap<String, BTreeMap<String, String>>> = OnceLock::new();
    CACHE.get_or_init(load_block_property_defaults)
}
