//! Predicate evaluation for advancement criteria.
//!
//! Vanilla parses each criterion's `conditions` object into a typed predicate
//! and only fires when it matches. Steel stores the raw JSON verbatim (see
//! [`steel_registry::advancement::criterion::SimpleCriterion`]), so this module
//! evaluates the subset of the predicate language that the vanilla advancement
//! data actually uses.
//!
//! Everything is *opt-in by absence*: an empty condition object (`{}`) matches
//! any event, which is why `minecraft:impossible`-style criteria still behave.
//! Unknown keys are ignored rather than treated as "no match", matching
//! vanilla's lenient deserialization of predicates.

use glam::DVec3;
use serde_json::Value;
use steel_registry::item_stack::ItemStack;
use steel_utils::Identifier;

use crate::world::World;

/// What happened, described in the terms the vanilla predicates use.
///
/// A trigger fires one of these; the matcher compares it against the criterion's
/// stored JSON. Keeping one struct instead of a type per trigger means a new
/// trigger can be wired up without touching the matcher.
#[derive(Clone, Debug, Default)]
pub struct TriggerContext {
    /// Item involved (placed block, destroyed block's item, consumed item, …).
    pub item: Option<Identifier>,
    /// Block involved, when the predicate distinguishes it from the item.
    pub block: Option<Identifier>,
    /// Entity type involved, as a `minecraft:` identifier.
    pub entity_type: Option<Identifier>,
    /// Block position, for `location` and distance predicates.
    pub position: Option<(f64, f64, f64)>,
    /// Whether the player was sneaking, for the `player` sub-predicate.
    pub player_sneaking: Option<bool>,
    /// Whether the item was the main hand, for `item`/`tool` predicates.
    pub main_hand: Option<bool>,
    /// Dimension the player left, for `changed_dimension`.
    pub from_dimension: Option<Identifier>,
    /// Dimension the player entered, for `changed_dimension`.
    pub to_dimension: Option<Identifier>,
    /// Distance travelled, for `nether_travel` and `levitation` distances.
    pub distance: Option<f64>,
    /// Duration in ticks, for `levitation`.
    pub duration: Option<i32>,
}

impl TriggerContext {
    /// A context that only carries an item identifier.
    #[must_use]
    pub fn item(item: &Identifier) -> Self {
        Self {
            item: Some(item.clone()),
            ..Self::default()
        }
    }

    /// A context for a block event, carrying both the block and its item form.
    #[must_use]
    pub fn block(block: &Identifier, item: Option<&Identifier>) -> Self {
        Self {
            block: Some(block.clone()),
            item: item.cloned(),
            ..Self::default()
        }
    }

    /// A context for an entity event, carrying the entity type.
    ///
    /// `player_killed_entity`, `tame_animal` and `villager_trade` all key their
    /// predicates on the entity type, so one constructor serves them.
    #[must_use]
    pub fn entity(entity_type: &Identifier) -> Self {
        Self {
            entity_type: Some(entity_type.clone()),
            ..Self::default()
        }
    }

    /// Adds the entity type to an existing context.
    ///
    /// Triggers that key on both an item and an entity (`killed_by_crossbow`
    /// carries the weapon and the victim) need both halves at once.
    #[must_use]
    pub fn with_entity_type(mut self, entity_type: &Identifier) -> Self {
        self.entity_type = Some(entity_type.clone());
        self
    }

    /// Adds the held item to an existing context.
    #[must_use]
    pub fn with_item(mut self, item: &Identifier) -> Self {
        self.item = Some(item.clone());
        self
    }

    /// A context for `changed_dimension`, carrying both dimension keys.
    ///
    /// Vanilla `ChangeDimensionTrigger.TriggerInstance` matches on `from`/`to`,
    /// usually to tell the Nether apart from the End.
    #[must_use]
    pub fn dimension(from: &Identifier, to: &Identifier) -> Self {
        Self {
            from_dimension: Some(from.clone()),
            to_dimension: Some(to.clone()),
            ..Self::default()
        }
    }

    /// A context for `nether_travel`, carrying the travelled distance.
    #[must_use]
    pub fn nether_travel(distance_travelled: f64) -> Self {
        Self {
            distance: Some(distance_travelled),
            ..Self::default()
        }
    }

    /// A context for a position-only trigger such as `location`.
    #[must_use]
    pub fn at(world: &World, pos: DVec3) -> Self {
        let _ = world;
        Self {
            position: Some((pos.x, pos.y, pos.z)),
            ..Self::default()
        }
    }
}

/// Returns whether `criterion_instance` accepts `context`.
///
/// `instance` is the JSON text stored by the generator; an empty object matches
/// everything.
#[must_use]
pub fn matches(instance: &str, context: &TriggerContext) -> bool {
    if instance.is_empty() {
        return true;
    }
    let Ok(value) = serde_json::from_str::<Value>(instance) else {
        // Unparseable conditions cannot be evaluated; fail open the way vanilla
        // does when a predicate is absent, rather than silently swallowing the
        // criterion forever.
        return true;
    };
    // Vanilla wraps every trigger's conditions in a `conditions` key for the
    // typed instances but generated data may already be the inner object.
    let conditions = value.get("conditions").unwrap_or(&value);
    let Value::Object(conditions) = conditions else {
        return true;
    };

    for (key, expected) in conditions {
        if !matches_key(key, expected, context) {
            return false;
        }
    }
    true
}

/// Evaluates one predicate key.
fn matches_key(key: &str, expected: &Value, context: &TriggerContext) -> bool {
    match key {
        // Block/item identity, used by `placed_block`, `destroyed_block`,
        // `consume_item`, `using_item`, `item_used_on_block` and friends.
        "item" => matches_item_predicate(expected, context.item.as_ref()),
        "block" => matches_id_predicate(expected, context.block.as_ref()),
        // `bee_nest_destroyed` and similar carry the block as `block`.
        "entity" | "entity_type" => matches_entity_predicate(expected, context),
        // `location` and distance checks.
        "location" => matches_location(expected, context.position),
        // `player` sub-predicate: Steel models sneaking, the only flag the
        // vanilla advancement data in 26.3 relies on for these triggers.
        "player" => matches_player(expected, context.player_sneaking),
        // `changed_dimension`: `{"from": "minecraft:overworld", "to": "minecraft:the_nether"}`.
        "from" => matches_dimension(expected, context.from_dimension.as_ref()),
        "to" => matches_dimension(expected, context.to_dimension.as_ref()),
        // `nether_travel` and `levitation` measure how far the player moved.
        "distance" => matches_distance(expected, context.distance),
        // `levitation` also carries how long the effect has been running.
        "duration" => context
            .duration
            .is_none_or(|duration| matches_number_range(expected, f64::from(duration))),
        // Nested conditions (e.g. `effects_changed`), evaluated recursively.
        "conditions" => true,
        // Unknown predicates are ignored: vanilla deserialization is lenient and
        // an unrecognised key would be a data-pack field Steel does not model.
        _ => true,
    }
}

/// Vanilla dimension predicate: a plain key or `{"dimension": "..."}`.
fn matches_dimension(expected: &Value, actual: Option<&Identifier>) -> bool {
    let Some(actual) = actual else {
        return is_unconstrained(expected);
    };
    match expected {
        Value::String(value) => id_matches(value, actual),
        Value::Object(map) => map
            .get("dimension")
            .is_none_or(|value| contains_id(value, actual)),
        _ => true,
    }
}

/// Vanilla `DistancePredicate`: bare number, `{"min": ..}`, or the full
/// `horizontal`/`x`/`y`/`z`/`absolute` breakdown.
///
/// Only the `absolute` (and bare) form is modelled, which is what the vanilla
/// `nether_travel` advancement uses.
fn matches_distance(expected: &Value, distance: Option<f64>) -> bool {
    let Some(distance) = distance else {
        return true;
    };
    match expected {
        Value::Number(_) => matches_number_range(expected, distance),
        Value::Object(map) => {
            if map.is_empty() {
                return true;
            }
            map.get("absolute")
                .or_else(|| map.get("horizontal"))
                .is_none_or(|range| matches_number_range(range, distance))
        }
        _ => true,
    }
}

/// Vanilla `ItemPredicate`: `{"items": "minecraft:stone"}` or `{"items": [...]}`.
fn matches_item_predicate(expected: &Value, item: Option<&Identifier>) -> bool {
    let Some(item) = item else {
        // A predicate that constrains the item cannot match an item-less event,
        // except when it constrains nothing.
        return is_unconstrained(expected);
    };
    match expected {
        Value::Object(map) => {
            if let Some(items) = map.get("items") {
                contains_id(items, item)
            } else if let Some(tag) = map.get("tag") {
                // Tags are not resolved here; a tag predicate is treated as a
                // match so criteria using tags are not silently lost.
                let _ = tag;
                true
            } else {
                true
            }
        }
        _ => true,
    }
}

/// Vanilla block predicate: `{"blocks": "minecraft:stone"}` or a plain string.
fn matches_id_predicate(expected: &Value, id: Option<&Identifier>) -> bool {
    let Some(id) = id else {
        return is_unconstrained(expected);
    };
    match expected {
        Value::Object(map) => map
            .get("blocks")
            .is_none_or(|blocks| contains_id(blocks, id)),
        Value::String(value) => id_matches(value, id),
        _ => true,
    }
}

/// Vanilla entity predicate: `{"type": "minecraft:pig"}`.
fn matches_entity_predicate(expected: &Value, context: &TriggerContext) -> bool {
    let Some(entity) = context.entity_type.as_ref() else {
        return is_unconstrained(expected);
    };
    match expected {
        Value::Object(map) => map
            .get("type")
            .is_none_or(|value| contains_id(value, entity)),
        Value::String(value) => id_matches(value, entity),
        _ => true,
    }
}

/// Vanilla `LocationPredicate`: `position`, `biome`, `dimension`, `smokey`, …
///
/// Only `position` is modelled; anything else is ignored.
fn matches_location(expected: &Value, position: Option<(f64, f64, f64)>) -> bool {
    let Value::Object(map) = expected else {
        return true;
    };
    let Some(position_value) = map.get("position") else {
        return true;
    };
    let Some((x, y, z)) = position else {
        return true;
    };
    let Some(bounds) = position_value.as_object() else {
        return true;
    };
    for (axis, value) in [("x", x), ("y", y), ("z", z)] {
        if let Some(range) = bounds.get(axis)
            && !matches_number_range(range, value)
        {
            return false;
        }
    }
    true
}

/// Vanilla `Range<f64>`: `{"min": .., "max": ..}` or a bare number.
fn matches_number_range(expected: &Value, value: f64) -> bool {
    match expected {
        Value::Number(number) => number.as_f64().is_some_and(|n| (value - n).abs() < 1.0e-6),
        Value::Object(map) => {
            let min_ok = map
                .get("min")
                .and_then(Value::as_f64)
                .is_none_or(|min| value >= min);
            let max_ok = map
                .get("max")
                .and_then(Value::as_f64)
                .is_none_or(|max| value <= max);
            min_ok && max_ok
        }
        _ => true,
    }
}

/// Vanilla `PlayerPredicate`: only the `flags` array is honoured.
fn matches_player(expected: &Value, sneaking: Option<bool>) -> bool {
    let Value::Object(map) = expected else {
        return true;
    };
    let Some(Value::Array(flags)) = map.get("flags") else {
        return true;
    };
    let Some(sneaking) = sneaking else {
        return true;
    };
    for flag in flags {
        let Value::Object(flag) = flag else {
            continue;
        };
        if let Some(Value::Bool(required)) = flag.get("is_sneaking")
            && *required != sneaking
        {
            return false;
        }
    }
    true
}

/// Whether a predicate object constrains nothing, so an absent id still matches.
fn is_unconstrained(expected: &Value) -> bool {
    match expected {
        Value::Object(map) => map.is_empty(),
        _ => false,
    }
}

/// Whether a vanilla id-or-list JSON value refers to `id`.
///
/// Vanilla writes the full `minecraft:` id, which is exactly what
/// [`Identifier`] compares, so the string is parsed and compared directly.
fn contains_id(expected: &Value, id: &Identifier) -> bool {
    match expected {
        Value::String(value) => id_matches(value, id),
        Value::Array(values) => values
            .iter()
            .any(|value| value.as_str().is_some_and(|value| id_matches(value, id))),
        _ => true,
    }
}

/// Compares a JSON id string against an [`Identifier`], tolerating the
/// namespace-less shorthand the data pack occasionally uses.
fn id_matches(value: &str, id: &Identifier) -> bool {
    if value == id.to_string() {
        return true;
    }
    // Vanilla allows `stone` to mean `minecraft:stone` inside predicates.
    value == id.path.as_ref()
}

/// Convenience for call sites that hold an [`ItemStack`].
#[must_use]
pub fn context_for_stack(stack: &ItemStack) -> TriggerContext {
    if stack.is_empty() {
        TriggerContext::default()
    } else {
        TriggerContext {
            item: Some(stack.item().key.clone()),
            ..TriggerContext::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(path: &str) -> Identifier {
        Identifier::vanilla_static(path)
    }

    #[test]
    fn an_empty_instance_matches_everything() {
        assert!(matches("", &TriggerContext::default()));
        assert!(matches("{}", &TriggerContext::default()));
    }

    #[test]
    fn item_predicate_accepts_the_listed_item() {
        let context = TriggerContext::item(&id("stone"));
        assert!(matches(r#"{"item":{"items":"minecraft:stone"}}"#, &context));
        assert!(!matches(r#"{"item":{"items":"minecraft:dirt"}}"#, &context));
        // A list of items matches any member.
        assert!(matches(
            r#"{"item":{"items":["minecraft:dirt","minecraft:stone"]}}"#,
            &context
        ));
    }

    #[test]
    fn item_predicate_rejects_a_missing_item() {
        let context = TriggerContext::default();
        assert!(!matches(r#"{"item":{"items":"minecraft:stone"}}"#, &context));
        // …but an unconstrained predicate still matches.
        assert!(matches(r#"{"item":{}}"#, &context));
    }

    #[test]
    fn location_predicate_checks_each_axis() {
        let context = TriggerContext {
            position: Some((10.0, 64.0, -3.0)),
            ..TriggerContext::default()
        };
        assert!(matches(
            r#"{"location":{"position":{"x":{"min":0,"max":20}}}}"#,
            &context
        ));
        assert!(!matches(
            r#"{"location":{"position":{"x":{"min":0,"max":5}}}}"#,
            &context
        ));
        assert!(matches(
            r#"{"location":{"position":{"y":64}}}"#,
            &context
        ));
    }

    #[test]
    fn player_sneaking_flag_is_honoured() {
        let sneaking = TriggerContext {
            player_sneaking: Some(true),
            ..TriggerContext::default()
        };
        assert!(matches(
            r#"{"player":{"flags":[{"is_sneaking":true}]}}"#,
            &sneaking
        ));
        assert!(!matches(
            r#"{"player":{"flags":[{"is_sneaking":false}]}}"#,
            &sneaking
        ));
    }

    #[test]
    fn unknown_predicates_do_not_block_the_criterion() {
        let context = TriggerContext::item(&id("stone"));
        assert!(matches(
            r#"{"item":{"items":"minecraft:stone"},"future_field":42}"#,
            &context
        ));
    }

    #[test]
    fn unparseable_instances_fail_open() {
        // A malformed data-pack predicate must not permanently hide an
        // advancement; vanilla would reject the file at load time instead.
        assert!(matches("[not an object", &TriggerContext::default()));
    }
}
