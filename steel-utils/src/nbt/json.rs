//! Bidirectional Mojang `DynamicOps` conversion between JSON (`JsonOps`) and
//! NBT (`NbtOps`).
//!
//! Used by registry-sync codecs, runtime datapack loaders (custom dialogs,
//! dynamic registry entries), and protocol conformance tests. Follows vanilla's
//! `JsonOps.INSTANCE.convertTo(NbtOps.INSTANCE, ...)` rules:
//!
//! * JSON booleans become `Byte(0 | 1)`.
//! * JSON integers become `Int` when in `i32::MIN..=i32::MAX`, `Long` when in
//!   `i64::MIN..=i64::MAX`, otherwise `Double`.
//! * Floating-point numbers become `Double` (or `Float` via
//!   [`json_to_nbt_float_preferred`] when encoding float-typed registry fields).
//! * Homogeneous arrays become typed `NbtList` variants; heterogeneous arrays
//!   wrap each element in vanilla's `NbtOps` marker compound `{"": element}` so
//!   no element is lost or rejected by binary NBT list rules.
//! * JSON `null` values inside objects are skipped (matching `NbtOps.createMap`
//!   ignoring `EndTag` values); a top-level `null` returns [`JsonNbtError::Null`].

use std::error::Error;
use std::fmt;

use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};
use simdnbt::owned::{NbtCompound, NbtList, NbtTag};

/// Errors produced when converting a JSON value to an [`NbtTag`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonNbtError {
    /// Top-level or list element was JSON `null`, which has no standalone NBT tag.
    Null,
    /// Floating-point value was non-finite (`NaN` or `Infinity`).
    NonFiniteNumber,
    /// Nesting exceeded vanilla's 512-level NBT recursion limit.
    DepthLimitExceeded,
}

impl fmt::Display for JsonNbtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => f.write_str("JSON null cannot be encoded as a standalone NBT tag"),
            Self::NonFiniteNumber => f.write_str("non-finite JSON number cannot be encoded as NBT"),
            Self::DepthLimitExceeded => f.write_str("JSON nesting exceeds maximum NBT depth (512)"),
        }
    }
}

impl Error for JsonNbtError {}

/// Vanilla `NbtAccounter.MAX_STACK_DEPTH`.
const MAX_NBT_DEPTH: usize = 512;

/// Converts a [`serde_json::Value`] into an owned [`NbtTag`] following Mojang's
/// `JsonOps -> NbtOps` rules.
///
/// # Errors
///
/// Returns [`JsonNbtError`] if the root value (or an array element) is `null`,
/// contains a non-finite number, or nests deeper than 512 levels.
pub fn json_to_nbt(value: &JsonValue) -> Result<NbtTag, JsonNbtError> {
    convert_json(value, 0, false)
}

/// Converts a JSON object directly into an [`NbtCompound`].
///
/// # Errors
///
/// Returns [`JsonNbtError`] on invalid nested values, or `None`-equivalent
/// error when `value` is not a JSON object.
pub fn json_object_to_nbt_compound(
    object: &JsonMap<String, JsonValue>,
) -> Result<NbtCompound, JsonNbtError> {
    convert_object(object, 0, false)
}

/// Converts a [`serde_json::Value`] into an owned [`NbtTag`], encoding finite
/// `f32`-range decimals as `NbtTag::Float` when they fit losslessly in `f32` or
/// come from float-typed codec trees (e.g. environment attributes).
///
/// # Errors
///
/// Returns [`JsonNbtError`] under the same conditions as [`json_to_nbt`].
pub fn json_to_nbt_float_preferred(value: &JsonValue) -> Result<NbtTag, JsonNbtError> {
    convert_json(value, 0, true)
}

fn convert_json(
    value: &JsonValue,
    depth: usize,
    prefer_float: bool,
) -> Result<NbtTag, JsonNbtError> {
    if depth > MAX_NBT_DEPTH {
        return Err(JsonNbtError::DepthLimitExceeded);
    }
    match value {
        JsonValue::Null => Err(JsonNbtError::Null),
        JsonValue::Bool(flag) => Ok(NbtTag::Byte(i8::from(*flag))),
        JsonValue::Number(num) => convert_number(num, prefer_float),
        JsonValue::String(text) => Ok(NbtTag::String(text.as_str().into())),
        JsonValue::Array(items) => {
            let mut tags = Vec::with_capacity(items.len());
            for item in items {
                tags.push(convert_json(item, depth + 1, prefer_float)?);
            }
            Ok(NbtTag::List(pack_nbt_list(tags)))
        }
        JsonValue::Object(map) => Ok(NbtTag::Compound(convert_object(
            map,
            depth + 1,
            prefer_float,
        )?)),
    }
}

fn convert_object(
    map: &JsonMap<String, JsonValue>,
    depth: usize,
    prefer_float: bool,
) -> Result<NbtCompound, JsonNbtError> {
    if depth > MAX_NBT_DEPTH {
        return Err(JsonNbtError::DepthLimitExceeded);
    }
    let mut compound = NbtCompound::new();
    for (key, value) in map {
        if value.is_null() {
            continue;
        }
        compound.insert(key.as_str(), convert_json(value, depth + 1, prefer_float)?);
    }
    Ok(compound)
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "prefer_float deliberately narrows JSON f64 literals to NBT float"
)]
fn convert_number(num: &JsonNumber, prefer_float: bool) -> Result<NbtTag, JsonNbtError> {
    if let Some(int) = num.as_i64() {
        if let Ok(narrow) = i32::try_from(int) {
            return Ok(NbtTag::Int(narrow));
        }
        return Ok(NbtTag::Long(int));
    }
    let Some(float) = num.as_f64() else {
        return Err(JsonNbtError::NonFiniteNumber);
    };
    if !float.is_finite() {
        return Err(JsonNbtError::NonFiniteNumber);
    }
    if prefer_float {
        let narrowed = float as f32;
        if narrowed.is_finite() {
            return Ok(NbtTag::Float(narrowed));
        }
    }
    Ok(NbtTag::Double(float))
}

/// Packs a sequence of arbitrary [`NbtTag`]s into a valid [`NbtList`].
///
/// Homogeneous sequences produce the matching typed `NbtList` variant (and
/// mixed integer widths widen to `Int`/`Long`, mixed float/double widen to
/// `Double`); truly heterogeneous sequences wrap every element inside vanilla's
/// `NbtOps` marker compound `{"": tag}`.
#[must_use]
pub fn pack_nbt_list(tags: Vec<NbtTag>) -> NbtList {
    if tags.is_empty() {
        return NbtList::Empty;
    }

    if tags.iter().all(|tag| matches!(tag, NbtTag::Byte(_))) {
        return NbtList::Byte(
            tags.into_iter()
                .filter_map(|tag| match tag {
                    NbtTag::Byte(v) => Some(v),
                    _ => None,
                })
                .collect(),
        );
    }
    if tags.iter().all(|tag| matches!(tag, NbtTag::Short(_))) {
        return NbtList::Short(
            tags.into_iter()
                .filter_map(|tag| match tag {
                    NbtTag::Short(v) => Some(v),
                    _ => None,
                })
                .collect(),
        );
    }
    if tags.iter().all(|tag| matches!(tag, NbtTag::Int(_))) {
        return NbtList::Int(
            tags.into_iter()
                .filter_map(|tag| match tag {
                    NbtTag::Int(v) => Some(v),
                    _ => None,
                })
                .collect(),
        );
    }
    if tags
        .iter()
        .all(|tag| matches!(tag, NbtTag::Int(_) | NbtTag::Long(_)))
    {
        return NbtList::Long(
            tags.into_iter()
                .filter_map(|tag| match tag {
                    NbtTag::Int(v) => Some(i64::from(v)),
                    NbtTag::Long(v) => Some(v),
                    _ => None,
                })
                .collect(),
        );
    }
    if tags.iter().all(|tag| matches!(tag, NbtTag::Float(_))) {
        return NbtList::Float(
            tags.into_iter()
                .filter_map(|tag| match tag {
                    NbtTag::Float(v) => Some(v),
                    _ => None,
                })
                .collect(),
        );
    }
    if tags
        .iter()
        .all(|tag| matches!(tag, NbtTag::Float(_) | NbtTag::Double(_)))
    {
        return NbtList::Double(
            tags.into_iter()
                .filter_map(|tag| match tag {
                    NbtTag::Float(v) => Some(f64::from(v)),
                    NbtTag::Double(v) => Some(v),
                    _ => None,
                })
                .collect(),
        );
    }
    if tags.iter().all(|tag| matches!(tag, NbtTag::String(_))) {
        return NbtList::String(
            tags.into_iter()
                .filter_map(|tag| match tag {
                    NbtTag::String(v) => Some(v),
                    _ => None,
                })
                .collect(),
        );
    }
    if tags.iter().all(|tag| matches!(tag, NbtTag::List(_))) {
        return NbtList::List(
            tags.into_iter()
                .filter_map(|tag| match tag {
                    NbtTag::List(v) => Some(v),
                    _ => None,
                })
                .collect(),
        );
    }
    pack_compound_or_heterogeneous_list(tags)
}

fn pack_compound_or_heterogeneous_list(tags: Vec<NbtTag>) -> NbtList {
    if tags.iter().all(|tag| matches!(tag, NbtTag::Compound(_))) {
        let needs_wrap = tags
            .iter()
            .any(|tag| matches!(tag, NbtTag::Compound(c) if is_wrapper_compound(c)));
        if !needs_wrap {
            return NbtList::Compound(
                tags.into_iter()
                    .filter_map(|tag| match tag {
                        NbtTag::Compound(v) => Some(v),
                        _ => None,
                    })
                    .collect(),
            );
        }
    }

    // Vanilla `NbtOps.HeterogenousListCollector`: wrap every entry as `{"": v}`
    // (leaving non-wrapper compounds unwrapped unless they collide with `{"": _}`).
    let wrapped = tags
        .into_iter()
        .map(|tag| match tag {
            NbtTag::Compound(compound) if !is_wrapper_compound(&compound) => compound,
            other => {
                let mut wrapper = NbtCompound::new();
                wrapper.insert("", other);
                wrapper
            }
        })
        .collect();
    NbtList::Compound(wrapped)
}

fn is_wrapper_compound(compound: &NbtCompound) -> bool {
    compound.len() == 1 && compound.contains("")
}

/// Converts an owned [`NbtTag`] into a [`serde_json::Value`] following Mojang's
/// `NbtOps.INSTANCE.convertTo(JsonOps.INSTANCE, ...)` rules.
///
/// Unwraps `NbtOps` heterogeneous list wrapper compounds (`{"": value}`).
#[must_use]
pub fn nbt_to_json(tag: &NbtTag) -> JsonValue {
    match tag {
        NbtTag::Byte(v) => JsonValue::Number((*v).into()),
        NbtTag::Short(v) => JsonValue::Number((*v).into()),
        NbtTag::Int(v) => JsonValue::Number((*v).into()),
        NbtTag::Long(v) => JsonValue::Number((*v).into()),
        NbtTag::Float(v) => JsonNumber::from_f64(f64::from(*v))
            .map_or(JsonValue::Null, JsonValue::Number),
        NbtTag::Double(v) => JsonNumber::from_f64(*v).map_or(JsonValue::Null, JsonValue::Number),
        NbtTag::ByteArray(bytes) => {
            #[expect(
                clippy::cast_possible_wrap,
                reason = "NBT byte arrays store signed two's-complement bytes"
            )]
            JsonValue::Array(
                bytes
                    .iter()
                    .map(|&b| JsonValue::Number((b as i8).into()))
                    .collect(),
            )
        }
        NbtTag::String(text) => JsonValue::String(text.to_str().into_owned()),
        NbtTag::List(list) => nbt_list_to_json(list),
        NbtTag::Compound(compound) => nbt_compound_to_json(compound),
        NbtTag::IntArray(ints) => {
            JsonValue::Array(ints.iter().map(|&v| JsonValue::Number(v.into())).collect())
        }
        NbtTag::LongArray(longs) => {
            JsonValue::Array(longs.iter().map(|&v| JsonValue::Number(v.into())).collect())
        }
    }
}

/// Converts an owned [`NbtCompound`] into a JSON object.
#[must_use]
pub fn nbt_compound_to_json(compound: &NbtCompound) -> JsonValue {
    let mut map = JsonMap::with_capacity(compound.len());
    for (key, value) in compound.iter() {
        map.insert(key.to_str().into_owned(), nbt_to_json(value));
    }
    JsonValue::Object(map)
}

fn nbt_list_to_json(list: &NbtList) -> JsonValue {
    match list {
        NbtList::Empty => JsonValue::Array(Vec::new()),
        NbtList::Byte(values) => JsonValue::Array(
            values
                .iter()
                .map(|&v| JsonValue::Number(v.into()))
                .collect(),
        ),
        NbtList::Short(values) => JsonValue::Array(
            values
                .iter()
                .map(|&v| JsonValue::Number(v.into()))
                .collect(),
        ),
        NbtList::Int(values) => JsonValue::Array(
            values
                .iter()
                .map(|&v| JsonValue::Number(v.into()))
                .collect(),
        ),
        NbtList::Long(values) => JsonValue::Array(
            values
                .iter()
                .map(|&v| JsonValue::Number(v.into()))
                .collect(),
        ),
        NbtList::Float(values) => JsonValue::Array(
            values
                .iter()
                .map(|&v| {
                    JsonNumber::from_f64(f64::from(v)).map_or(JsonValue::Null, JsonValue::Number)
                })
                .collect(),
        ),
        NbtList::Double(values) => JsonValue::Array(
            values
                .iter()
                .map(|&v| JsonNumber::from_f64(v).map_or(JsonValue::Null, JsonValue::Number))
                .collect(),
        ),
        NbtList::ByteArray(values) => JsonValue::Array(
            values
                .iter()
                .map(|arr| nbt_to_json(&NbtTag::ByteArray(arr.clone())))
                .collect(),
        ),
        NbtList::String(values) => JsonValue::Array(
            values
                .iter()
                .map(|v| JsonValue::String(v.to_str().into_owned()))
                .collect(),
        ),
        NbtList::List(values) => JsonValue::Array(values.iter().map(nbt_list_to_json).collect()),
        NbtList::Compound(values) => JsonValue::Array(
            values
                .iter()
                .map(|compound| {
                    if compound.len() == 1
                        && let Some(inner) = compound.get("")
                    {
                        return nbt_to_json(inner);
                    }
                    nbt_compound_to_json(compound)
                })
                .collect(),
        ),
        NbtList::IntArray(values) => JsonValue::Array(
            values
                .iter()
                .map(|arr| nbt_to_json(&NbtTag::IntArray(arr.clone())))
                .collect(),
        ),
        NbtList::LongArray(values) => JsonValue::Array(
            values
                .iter()
                .map(|arr| nbt_to_json(&NbtTag::LongArray(arr.clone())))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use simdnbt::owned::{NbtList, NbtTag};

    use super::{JsonNbtError, json_to_nbt, json_to_nbt_float_preferred, nbt_to_json};

    #[test]
    fn converts_primitives_and_nested_objects() {
        let input = json!({
            "flag": true,
            "count": 42,
            "big": 9_000_000_000_i64,
            "ratio": 0.25,
            "name": "minecraft:sulfur_cube",
            "ignored": null,
            "tags": ["a", "b", "c"],
            "ints": [1, 2, 3],
        });
        let tag = json_to_nbt(&input).expect("valid JSON should convert");
        let NbtTag::Compound(compound) = &tag else {
            panic!("expected compound");
        };
        assert_eq!(compound.get("flag"), Some(&NbtTag::Byte(1)));
        assert_eq!(compound.get("count"), Some(&NbtTag::Int(42)));
        assert_eq!(compound.get("big"), Some(&NbtTag::Long(9_000_000_000)));
        assert_eq!(compound.get("ratio"), Some(&NbtTag::Double(0.25)));
        assert!(compound.get("ignored").is_none());
        assert!(matches!(
            compound.get("tags"),
            Some(NbtTag::List(NbtList::String(_)))
        ));
        assert!(matches!(
            compound.get("ints"),
            Some(NbtTag::List(NbtList::Int(_)))
        ));

        let back = nbt_to_json(&tag);
        assert_eq!(back["count"], json!(42));
        assert_eq!(back["big"], json!(9_000_000_000_i64));
        assert_eq!(back["ratio"], json!(0.25));
        assert_eq!(back["name"], json!("minecraft:sulfur_cube"));
        assert_eq!(back["tags"], json!(["a", "b", "c"]));
        assert_eq!(back["ints"], json!([1, 2, 3]));
    }

    #[test]
    fn wraps_and_unwraps_heterogeneous_lists_like_vanilla_nbt_ops() {
        let input = json!([
            "plain_text",
            {"translate": "gui.ok", "color": "green"},
            7
        ]);
        let tag = json_to_nbt(&input).expect("heterogeneous list should wrap");
        assert!(matches!(tag, NbtTag::List(NbtList::Compound(_))));
        let round_trip = nbt_to_json(&tag);
        assert_eq!(round_trip, input);
    }

    #[test]
    fn float_preferred_mode_produces_float_tags_and_rejects_null() {
        let tag = json_to_nbt_float_preferred(&json!(0.5)).expect("float converts");
        assert_eq!(tag, NbtTag::Float(0.5));
        assert_eq!(json_to_nbt(&json!(null)), Err(JsonNbtError::Null));
    }
}
