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

use std::cmp::Ordering;

use glam::DVec3;
use serde_json::Value;
use steel_registry::{
    REGISTRY, TaggedRegistryExt, item_predicate::ItemPredicate, item_stack::ItemStack,
};
use steel_utils::{BlockPos, BlockStateId, Identifier, nbt::json_to_nbt};

use crate::command::execution::registered_item_stack_matches;
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
    /// The full event stack, when available, including count and components.
    pub item_stack: Option<ItemStack>,
    /// Inventory snapshot for inventory-changed criteria, including empty slots.
    pub inventory: Option<Vec<ItemStack>>,
    /// Block involved, when the predicate distinguishes it from the item.
    pub block: Option<Identifier>,
    /// Exact block state involved in a block criterion.
    pub block_state: Option<BlockStateId>,
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
    /// Distance traveled, for `nether_travel` and `levitation` distances.
    pub distance: Option<f64>,
    /// Duration in ticks, for `levitation`.
    pub duration: Option<i32>,
    /// Recipe key, for `recipe_unlocked`.
    pub recipe: Option<Identifier>,
    /// Number of occupants remaining after a nest is destroyed.
    pub bees_inside: Option<i32>,
    /// Pyramid level of a constructed beacon, for `construct_beacon`.
    pub dimension_level: Option<i32>,
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

    /// Adds the exact state for block property predicates.
    #[must_use]
    pub const fn with_block_state(mut self, state: BlockStateId) -> Self {
        self.block_state = Some(state);
        self
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

    /// Captures a block event's current state and vanilla block-center origin.
    #[must_use]
    pub fn block_at(world: &World, pos: BlockPos) -> Self {
        let state = world.get_block_state(pos);
        let block = REGISTRY
            .blocks
            .by_state_id(state)
            .map(|block| block.key.clone());
        let (x, y, z) = pos.get_center();
        Self {
            block,
            block_state: Some(state),
            position: Some((x, y, z)),
            ..Self::default()
        }
    }

    /// A context for `recipe_unlocked`.
    #[must_use]
    pub fn recipe(recipe: &Identifier) -> Self {
        Self {
            recipe: Some(recipe.clone()),
            ..Self::default()
        }
    }

    /// Adds the held item to an existing context.
    #[must_use]
    pub fn with_item(mut self, item: &Identifier) -> Self {
        self.item = Some(item.clone());
        self.item_stack = None;
        self
    }

    /// Adds the actual stack for count and component predicates.
    #[must_use]
    pub fn with_item_stack(mut self, stack: &ItemStack) -> Self {
        self.item = (!stack.is_empty()).then(|| stack.item().key.clone());
        self.item_stack = Some(stack.clone());
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

    /// A context for `nether_travel`, carrying the traveled distance.
    #[must_use]
    pub fn nether_travel(distance_traveled: f64) -> Self {
        Self {
            distance: Some(distance_traveled),
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

/// Matches a player-only trigger without a player predicate.
#[must_use]
pub fn matches_unconditional_player_trigger(instance: &str) -> bool {
    let Ok(value) = serde_json::from_str::<Value>(instance) else {
        return false;
    };
    value
        .get("conditions")
        .unwrap_or(&value)
        .as_object()
        .is_some_and(serde_json::Map::is_empty)
}

/// Matches dimension keys, rejecting player predicates until their loot context is supported.
#[must_use]
pub fn matches_changed_dimension(instance: &str, context: &TriggerContext) -> bool {
    let (Some(from), Some(to)) = (&context.from_dimension, &context.to_dimension) else {
        return false;
    };
    let Ok(value) = serde_json::from_str::<Value>(instance) else {
        return false;
    };
    let Some(conditions) = value.get("conditions").unwrap_or(&value).as_object() else {
        return false;
    };
    if conditions.keys().any(|key| key != "from" && key != "to") {
        return false;
    }
    [("from", from), ("to", to)]
        .into_iter()
        .all(|(key, actual)| {
            conditions.get(key).is_none_or(|expected| {
                expected
                    .as_str()
                    .and_then(|id| id.parse::<Identifier>().ok())
                    .is_some_and(|id| id == *actual)
            })
        })
}

/// Conservative matcher for block-use and block-placement location triggers.
/// Player loot predicates and named condition references are not yet supported;
/// do not award irreversible progress when their result is unknown.
#[must_use]
pub fn matches_block_location(instance: &str, context: &TriggerContext) -> bool {
    let Ok(value) = serde_json::from_str::<Value>(instance) else {
        return false;
    };
    let Some(conditions) = value.get("conditions").unwrap_or(&value).as_object() else {
        return false;
    };
    if conditions.keys().any(|key| key != "location") {
        return false;
    }
    conditions
        .get("location")
        .is_none_or(|location| matches_location_loot_condition(location, context, 0) == Some(true))
}

/// Conservative matcher for triggers whose only event-specific field is `item`.
/// Player loot predicates remain unsupported and cannot grant progress.
#[must_use]
pub fn matches_used_item(instance: &str, context: &TriggerContext) -> bool {
    let Ok(value) = serde_json::from_str::<Value>(instance) else {
        return false;
    };
    let Some(conditions) = value.get("conditions").unwrap_or(&value).as_object() else {
        return false;
    };
    if conditions.keys().any(|key| key != "item") || context.item_stack.is_none() {
        return false;
    }
    conditions.get("item").is_none_or(|item| {
        let Some(fields) = item.as_object() else {
            return false;
        };
        // Nested component codecs are not yet fully strict; limit irreversible
        // grants to holder sets and well-formed integer count bounds for now.
        if fields.keys().any(|key| key != "items" && key != "count") {
            return false;
        }
        if let Some(count) = fields.get("count") {
            let valid_integer = |v: &Value| v.as_i64().is_some_and(|n| i32::try_from(n).is_ok());
            let valid = valid_integer(count)
                || count.as_object().is_some_and(|bounds| {
                    bounds
                        .iter()
                        .all(|(key, value)| (key == "min" || key == "max") && valid_integer(value))
                });
            if !valid {
                return false;
            }
        }
        matches_item_predicate(item, context)
    })
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
        "item" => matches_item_predicate(expected, context),
        "items" => matches_inventory_items(expected, context),
        "slots" => matches_inventory_slots(expected, context.inventory.as_deref()),
        "block" => matches_id_predicate(expected, context.block.as_ref()),
        // Enter-block, slide-down-block and bee-nest triggers use a holder set.
        "blocks" => context
            .block
            .as_ref()
            .is_some_and(|block| contains_holder_set(expected, block, || &REGISTRY.blocks)),
        "state" => context
            .block_state
            .is_some_and(|state| matches_block_state(expected, state)),
        "entity" | "entity_type" => matches_entity_predicate(expected, context),
        // `location` and distance checks.
        "location" => {
            if expected.get("type").is_some() || expected.get("condition").is_some() {
                matches_location_loot_condition(expected, context, 0) == Some(true)
            } else {
                matches_location(expected, context.position)
            }
        }
        // `player` sub-predicate: Steel models sneaking, the only flag the
        // vanilla advancement data in 26.3 relies on for these triggers.
        "player" => matches_player(expected, context.player_sneaking),
        // `changed_dimension`: `{"from": "minecraft:overworld", "to": "minecraft:the_nether"}`.
        "from" => matches_dimension(expected, context.from_dimension.as_ref()),
        "to" => matches_dimension(expected, context.to_dimension.as_ref()),
        // `nether_travel` and `levitation` measure how far the player moved.
        "distance" => matches_distance(expected, context.distance),
        "num_bees_inside" => context
            .bees_inside
            .is_some_and(|count| matches_number_range(expected, f64::from(count))),
        // `construct_beacon` reports the pyramid level that was built.
        "level" => context
            .dimension_level
            .is_none_or(|level| matches_number_range(expected, f64::from(level))),
        // `recipe_unlocked` names the recipe the player just learned.
        "recipe" => context
            .recipe
            .as_ref()
            .is_none_or(|recipe| contains_id(expected, recipe)),
        // `levitation` also carries how long the effect has been running.
        "duration" => context
            .duration
            .is_none_or(|duration| matches_number_range(expected, f64::from(duration))),
        // Nested conditions (e.g. `effects_changed`), evaluated recursively, and
        // unknown predicates are ignored: vanilla deserialization is lenient and
        // an unrecognised key would be a data-pack field Steel does not model.
        _ => true,
    }
}

/// Matches vanilla's exact and inclusive ranged state property predicates.
fn matches_block_state(expected: &Value, state: BlockStateId) -> bool {
    let Some(predicates) = expected.as_object() else {
        return false;
    };
    let Some(block) = REGISTRY.blocks.by_state_id(state) else {
        return false;
    };
    predicates.iter().all(|(name, expected)| {
        let Some(property) = block
            .properties
            .iter()
            .find(|property| property.get_name() == name.as_str())
        else {
            return false;
        };
        let Some(actual) = REGISTRY.blocks.get_property_str(state, name) else {
            return false;
        };
        if let Some(exact) = expected.as_str() {
            return property.compare_value_names(actual, exact) == Some(Ordering::Equal);
        }
        let Some(range) = expected.as_object() else {
            return false;
        };
        range.get("min").is_none_or(|min| {
            min.as_str()
                .and_then(|min| property.compare_value_names(actual, min))
                .is_some_and(|order| order != Ordering::Less)
        }) && range.get("max").is_none_or(|max| {
            max.as_str()
                .and_then(|max| property.compare_value_names(actual, max))
                .is_some_and(|order| order != Ordering::Greater)
        })
    })
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

/// Vanilla item identity, count and registered component matchers.
fn matches_item_predicate(expected: &Value, context: &TriggerContext) -> bool {
    matches_item_view(expected, context.item.as_ref(), context.item_stack.as_ref())
}

fn matches_item_view(
    expected: &Value,
    item: Option<&Identifier>,
    stack: Option<&ItemStack>,
) -> bool {
    let Some(map) = expected.as_object() else {
        return false;
    };
    if map.keys().any(|key| {
        !matches!(
            key.as_str(),
            "items" | "count" | "components" | "predicates"
        )
    }) {
        return false;
    }
    if let Some(items) = map.get("items")
        && !item.is_some_and(|item| contains_holder_set(items, item, || &REGISTRY.items))
    {
        return false;
    }
    if !map.contains_key("count")
        && !map.contains_key("components")
        && !map.contains_key("predicates")
    {
        return item.is_some() || is_unconstrained(expected);
    }
    let Some(stack) = stack else {
        return false;
    };
    // The shared command matcher implements the registered typed component
    // predicates. Identity is already checked above, so decode the remaining
    // fields without re-resolving holder sets.
    let mut fields = map.clone();
    fields.remove("items");
    let Ok(tag) = json_to_nbt(&Value::Object(fields)) else {
        return false;
    };
    ItemPredicate::from_owned_nbt(&tag)
        .is_some_and(|predicate| registered_item_stack_matches(&predicate, stack))
}

/// Inventory predicates use the changed stack for a single condition; when
/// there are several, every predicate must match at least one occupied slot.
fn matches_inventory_items(expected: &Value, context: &TriggerContext) -> bool {
    let Some(predicates) = expected.as_array() else {
        return false;
    };
    if predicates.is_empty() {
        return true;
    }
    if predicates.len() == 1 {
        return context.item_stack.as_ref().is_some_and(|stack| {
            !stack.is_empty()
                && matches_item_view(&predicates[0], Some(&stack.item().key), Some(stack))
        });
    }
    context.inventory.as_ref().is_some_and(|inventory| {
        predicates.iter().all(|predicate| {
            inventory.iter().any(|stack| {
                !stack.is_empty()
                    && matches_item_view(predicate, Some(&stack.item().key), Some(stack))
            })
        })
    })
}

fn matches_inventory_slots(expected: &Value, inventory: Option<&[ItemStack]>) -> bool {
    let Some(bounds) = expected.as_object() else {
        return false;
    };
    if bounds.is_empty() {
        return true;
    }
    let Some(inventory) = inventory else {
        return false;
    };
    let occupied = inventory.iter().filter(|stack| !stack.is_empty()).count();
    let full = inventory
        .iter()
        .filter(|stack| !stack.is_empty() && stack.count() >= stack.max_stack_size())
        .count();
    let empty = inventory.len() - occupied;
    [("occupied", occupied), ("full", full), ("empty", empty)]
        .into_iter()
        .all(|(key, count)| {
            bounds.get(key).is_none_or(|range| {
                i32::try_from(count)
                    .is_ok_and(|count| matches_number_range(range, f64::from(count)))
            })
        })
}

/// Vanilla block predicate: `{"blocks": "minecraft:stone"}` or a plain string.
fn matches_id_predicate(expected: &Value, id: Option<&Identifier>) -> bool {
    let Some(id) = id else {
        return is_unconstrained(expected);
    };
    match expected {
        Value::Object(map) => map
            .get("blocks")
            .is_none_or(|blocks| contains_holder_set(blocks, id, || &REGISTRY.blocks)),
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
            .is_none_or(|value| contains_holder_set(value, entity, || &REGISTRY.entity_types)),
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

/// Evaluates the supported location-loot conditions with the captured event data.
/// `None` means unsupported or unavailable context; keeping it distinct from
/// false prevents an inverted unknown predicate from granting an advancement.
fn matches_location_loot_condition(
    expected: &Value,
    context: &TriggerContext,
    depth: usize,
) -> Option<bool> {
    if depth >= 64 {
        return None;
    }
    let map = expected.as_object()?;
    let name = map.get("type").or_else(|| map.get("condition"))?.as_str()?;
    let name = name.strip_prefix("minecraft:").unwrap_or(name);
    match name {
        "all_of" | "any_of" => {
            let terms = map.get("terms")?.as_array()?;
            let mut unknown = false;
            for term in terms {
                match matches_location_loot_condition(term, context, depth + 1) {
                    Some(value) if value == (name == "any_of") => return Some(value),
                    Some(_) => {}
                    None => unknown = true,
                }
            }
            if unknown {
                None
            } else {
                Some(name == "all_of")
            }
        }
        "inverted" => matches_location_loot_condition(map.get("term")?, context, depth + 1)
            .map(|result| !result),
        "match_tool" => {
            let stack = context.item_stack.as_ref()?;
            Some(map.get("predicate").is_none_or(|predicate| {
                matches_item_view(
                    predicate,
                    (!stack.is_empty()).then_some(&stack.item().key),
                    Some(stack),
                )
            }))
        }
        "match_block" => matches_event_block(expected, context),
        "location_check" => {
            let (mut x, mut y, mut z) = context.position?;
            let mut has_offset = false;
            for (key, value) in [
                ("offsetX", &mut x),
                ("offsetY", &mut y),
                ("offsetZ", &mut z),
            ] {
                if let Some(offset) = map.get(key) {
                    let offset = i32::try_from(offset.as_i64()?).ok()?;
                    has_offset |= offset != 0;
                    *value += f64::from(offset);
                }
            }
            let Some(predicate) = map.get("predicate") else {
                return Some(true);
            };
            let predicate = predicate.as_object()?;
            if predicate
                .keys()
                .any(|key| key != "position" && key != "block")
            {
                return None;
            }
            if let Some(position) = predicate.get("position") {
                let bounds = position.as_object()?;
                for (axis, value) in [("x", x), ("y", y), ("z", z)] {
                    if bounds
                        .get(axis)
                        .is_some_and(|range| !matches_number_range(range, value))
                    {
                        return Some(false);
                    }
                }
            }
            if let Some(block) = predicate.get("block") {
                // Only the event block was captured; other offsets need a world lookup.
                if has_offset {
                    return None;
                }
                return matches_event_block(block, context);
            }
            Some(true)
        }
        _ => None,
    }
}

fn matches_event_block(expected: &Value, context: &TriggerContext) -> Option<bool> {
    let map = expected.as_object()?;
    let state = context.block_state?;
    if map.contains_key("nbt") || map.contains_key("components") || map.contains_key("predicates") {
        return None;
    }
    if let Some(blocks) = map.get("blocks") {
        let block = REGISTRY.blocks.by_state_id(state)?;
        if !contains_holder_set(blocks, &block.key, || &REGISTRY.blocks) {
            return Some(false);
        }
    }
    Some(
        map.get("state")
            .is_none_or(|value| matches_block_state(value, state)),
    )
}

/// Vanilla `Range<f64>`: `{"min": .., "max": ..}` or a bare number.
#[expect(clippy::float_cmp, reason = "a point bound is exact, as in vanilla")]
fn matches_number_range(expected: &Value, value: f64) -> bool {
    match expected {
        Value::Number(number) => number.as_f64().is_some_and(|n| value == n),
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

/// Evaluate a registry holder set: one entry, a tag, or a list of entries.
fn contains_holder_set<R: TaggedRegistryExt + 'static>(
    expected: &Value,
    id: &Identifier,
    registry: impl FnOnce() -> &'static R,
) -> bool {
    match expected {
        Value::String(value) => {
            if let Some(tag) = value.strip_prefix('#') {
                let Ok(tag) = tag.parse::<Identifier>() else {
                    return false;
                };
                let registry = registry();
                registry
                    .by_key(id)
                    .is_some_and(|entry| registry.is_in_tag(entry, &tag))
            } else {
                id_matches(value, id)
            }
        }
        Value::Array(values) => values
            .iter()
            .any(|value| value.as_str().is_some_and(|value| id_matches(value, id))),
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
    let (namespace, path) = value
        .split_once(':')
        .unwrap_or((Identifier::VANILLA_NAMESPACE, value));
    let namespace = if namespace.is_empty() {
        Identifier::VANILLA_NAMESPACE
    } else {
        namespace
    };
    id.namespace == namespace && id.path == path && Identifier::validate(namespace, path)
}

/// Convenience for call sites that hold an [`ItemStack`].
#[must_use]
pub fn context_for_stack(stack: &ItemStack) -> TriggerContext {
    TriggerContext::default().with_item_stack(stack)
}

#[cfg(test)]
mod tests {
    use super::*;
    use steel_registry::items::ItemRegistry;

    fn id(path: &str) -> Identifier {
        Identifier::vanilla(path.to_owned())
    }

    #[test]
    fn unconditional_player_trigger_rejects_unevaluated_predicates() {
        assert!(matches_unconditional_player_trigger("{}"));
        assert!(matches_unconditional_player_trigger(r#"{"conditions":{}}"#));
        assert!(!matches_unconditional_player_trigger(r#"{"player":{}}"#));
        assert!(!matches_unconditional_player_trigger(
            r#"{"unexpected":true}"#
        ));
        assert!(!matches_unconditional_player_trigger("[]"));
        assert!(!matches_unconditional_player_trigger("invalid"));
    }

    #[test]
    fn dimension_change_requires_matching_keys_and_rejects_unknown_player_predicates() {
        let context = TriggerContext::dimension(&id("overworld"), &id("the_nether"));
        assert!(matches_changed_dimension("{}", &context));
        assert!(matches_changed_dimension(
            r#"{"from":"minecraft:overworld","to":"the_nether"}"#,
            &context
        ));
        assert!(!matches_changed_dimension(
            r#"{"to":"minecraft:the_end"}"#,
            &context
        ));
        assert!(!matches_changed_dimension(
            r#"{"from":"minecraft:the_end"}"#,
            &context
        ));
        assert!(!matches_changed_dimension(r#"{"to":5}"#, &context));
        assert!(!matches_changed_dimension(r#"{"player":{}}"#, &context));
        assert!(!matches_changed_dimension("{}", &TriggerContext::default()));
        assert!(!matches_changed_dimension("invalid", &context));
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
    fn direct_holder_sets_do_not_access_registry() {
        let stone = id("stone");
        assert!(contains_holder_set::<ItemRegistry>(
            &serde_json::json!("minecraft:stone"),
            &stone,
            || panic!("direct holder set should not access the registry")
        ));
    }

    #[test]
    fn item_predicate_resolves_holder_set_tags() {
        steel_registry::init_vanilla_registry();
        let sword = TriggerContext::item(&id("iron_sword"));
        let stone = TriggerContext::item(&id("stone"));
        let swords = r##"{"item":{"items":"#minecraft:swords"}}"##;
        assert!(matches(swords, &sword));
        assert!(!matches(swords, &stone));
        assert!(!matches(
            r##"{"item":{"items":"#minecraft:missing_tag"}}"##,
            &sword
        ));
    }

    #[test]
    fn block_holder_sets_match_actual_block_not_just_any_event() {
        steel_registry::init_vanilla_registry();
        let log = TriggerContext::block(&id("oak_log"), None);
        let stone = TriggerContext::block(&id("stone"), None);
        let tag = r##"{"blocks":"#minecraft:logs"}"##;
        assert!(matches(tag, &log));
        assert!(!matches(tag, &stone));
        assert!(!matches(tag, &TriggerContext::default()));
        assert!(matches(
            r#"{"blocks":["minecraft:stone","minecraft:oak_log"]}"#,
            &log
        ));
        assert!(!matches(r#"{"blocks":"minecraft:stone"}"#, &log));
        assert!(matches(r##"{"block":{"blocks":"#minecraft:logs"}}"##, &log));
    }

    #[test]
    fn block_state_properties_require_exact_or_typed_range_matches() {
        use steel_registry::blocks::block_state_ext::BlockStateExt as _;
        use steel_registry::blocks::properties::BlockStateProperties;
        use steel_registry::vanilla_blocks;

        steel_registry::init_vanilla_registry();
        let state = vanilla_blocks::BEE_NEST
            .default_state()
            .set_value(&BlockStateProperties::LEVEL_HONEY, 3);
        let context = TriggerContext::block(&id("bee_nest"), None).with_block_state(state);
        assert!(matches(r#"{"state":{"honey_level":"3"}}"#, &context));
        assert!(!matches(r#"{"state":{"honey_level":"2"}}"#, &context));
        assert!(matches(
            r#"{"state":{"honey_level":{"min":"2","max":"3"}}}"#,
            &context
        ));
        assert!(!matches(
            r#"{"state":{"honey_level":{"min":"4"}}}"#,
            &context
        ));
        assert!(!matches(
            r#"{"state":{"honey_level":{"max":"99"}}}"#,
            &context
        ));
        assert!(!matches(r#"{"state":{"missing":"3"}}"#, &context));
        assert!(!matches(r#"{"state":{"honey_level":3}}"#, &context));
        assert!(!matches(r#"{"state":{}}"#, &TriggerContext::default()));

        let campfire = vanilla_blocks::CAMPFIRE
            .default_state()
            .set_value(&BlockStateProperties::LIT, true);
        let context = TriggerContext::block(&id("campfire"), None).with_block_state(campfire);
        assert!(matches(r#"{"state":{"lit":{"min":"false"}}}"#, &context));
        assert!(!matches(r#"{"state":{"lit":{"max":"false"}}}"#, &context));
    }

    #[test]
    fn entity_type_holder_set_resolves_tags() {
        steel_registry::init_vanilla_registry();
        let bee = TriggerContext::entity(&id("bee"));
        let cow = TriggerContext::entity(&id("cow"));
        let arthropods = r##"{"entity":{"type":"#minecraft:arthropod"}}"##;
        assert!(matches(arthropods, &bee));
        assert!(!matches(arthropods, &cow));
    }

    #[test]
    fn unqualified_ids_only_match_the_minecraft_namespace() {
        let modded = TriggerContext::item(&Identifier::new("example", "stone"));
        let vanilla = TriggerContext::item(&id("stone"));
        let predicate = r#"{"item":{"items":"stone"}}"#;
        assert!(!matches(predicate, &modded));
        assert!(matches(predicate, &vanilla));
        assert!(matches(r#"{"item":{"items":"example:stone"}}"#, &modded));
        assert!(matches(r#"{"item":{"items":":stone"}}"#, &vanilla));
        assert!(!matches(
            r#"{"item":{"items":"example:stone:bad"}}"#,
            &modded
        ));
    }

    #[test]
    fn item_predicates_use_live_stack_count_and_components() {
        use steel_registry::data_components::vanilla_components::DAMAGE;
        use steel_registry::vanilla_items;

        steel_registry::init_vanilla_registry();
        let mut stack = ItemStack::with_count(&vanilla_items::IRON_PICKAXE, 2);
        stack.set(DAMAGE, 5);
        let context = context_for_stack(&stack);
        assert!(matches(
            r#"{"item":{"items":"minecraft:iron_pickaxe","count":{"min":2},"components":{"minecraft:damage":5}}}"#,
            &context
        ));
        assert!(!matches(r#"{"item":{"count":{"min":3}}}"#, &context));
        assert!(!matches(
            r#"{"item":{"components":{"minecraft:damage":4}}}"#,
            &context
        ));
        assert!(matches(
            r#"{"item":{"predicates":{"minecraft:damage":{"damage":{"min":5}}}}}"#,
            &context
        ));
        assert!(!matches(
            r#"{"item":{"predicates":{"minecraft:damage":{"damage":{"max":4}}}}}"#,
            &context
        ));
        assert!(!matches(
            r#"{"item":{"count":2}}"#,
            &TriggerContext::item(&id("iron_pickaxe"))
        ));
    }

    #[test]
    fn used_item_conditions_require_stack_and_reject_unsupported_player_predicates() {
        use steel_registry::vanilla_items;

        steel_registry::init_vanilla_registry();
        let apple = context_for_stack(&ItemStack::with_count(&vanilla_items::APPLE, 2));
        assert!(matches_used_item("{}", &apple));
        assert!(matches_used_item(
            r#"{"item":{"items":"minecraft:apple","count":{"min":2}}}"#,
            &apple
        ));
        assert!(!matches_used_item(
            r#"{"item":{"items":"minecraft:carrot"}}"#,
            &apple
        ));
        assert!(!matches_used_item(
            r#"{"player":{"type":"minecraft:entity_properties"}}"#,
            &apple
        ));
        assert!(!matches_used_item(
            r#"{"item":{"items":"minecraft:apple","unknown":true}}"#,
            &apple
        ));
        assert!(!matches_used_item("{}", &TriggerContext::default()));
    }

    #[test]
    fn inventory_change_requires_changed_item_or_all_requested_slots() {
        use steel_registry::vanilla_items;

        steel_registry::init_vanilla_registry();
        let stone = ItemStack::with_count(&vanilla_items::STONE, 64);
        let dirt = ItemStack::new(&vanilla_items::DIRT);
        let mut context = context_for_stack(&stone);
        context.inventory = Some(vec![stone, dirt, ItemStack::empty()]);
        assert!(matches(
            r#"{"items":[{"items":"minecraft:stone"}]}"#,
            &context
        ));
        assert!(!matches(
            r#"{"items":[{"items":"minecraft:dirt"}]}"#,
            &context
        ));
        assert!(matches(
            r#"{"items":[{"items":"minecraft:stone"},{"items":"minecraft:dirt"}]}"#,
            &context
        ));
        assert!(!matches(
            r#"{"items":[{"items":"minecraft:stone"},{"items":"minecraft:diamond"}]}"#,
            &context
        ));
        assert!(matches(
            r#"{"slots":{"full":1,"occupied":2,"empty":1}}"#,
            &context
        ));
        assert!(!matches(r#"{"slots":{"full":2}}"#, &context));
        assert!(!matches(
            r#"{"items":[{"items":"minecraft:stone"}]}"#,
            &TriggerContext::default()
        ));
    }

    #[test]
    fn item_predicate_rejects_a_missing_item() {
        let context = TriggerContext::default();
        assert!(!matches(
            r#"{"item":{"items":"minecraft:stone"}}"#,
            &context
        ));
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
        assert!(matches(r#"{"location":{"position":{"y":64}}}"#, &context));
    }

    #[test]
    fn location_loot_conditions_check_block_tool_and_offsets() {
        use steel_registry::{vanilla_blocks, vanilla_items};

        steel_registry::init_vanilla_registry();
        let stack = ItemStack::new(&vanilla_items::HONEYCOMB);
        let context = TriggerContext {
            block_state: Some(vanilla_blocks::COPPER_BLOCK.default_state()),
            position: Some((8.5, 64.5, 8.5)),
            ..context_for_stack(&stack)
        };
        let condition = r#"{"location":{"type":"minecraft:all_of","terms":[{"type":"minecraft:location_check","predicate":{"block":{"blocks":"minecraft:copper_block"},"position":{"y":{"min":64}}}},{"type":"minecraft:match_tool","predicate":{"items":"minecraft:honeycomb"}}]}}"#;
        assert!(matches(condition, &context));
        let wrong_tool = context
            .clone()
            .with_item_stack(&ItemStack::new(&vanilla_items::STICK));
        assert!(!matches(condition, &wrong_tool));
        let wrong_block = TriggerContext {
            block_state: Some(vanilla_blocks::STONE.default_state()),
            ..context.clone()
        };
        assert!(!matches(condition, &wrong_block));
        assert!(matches(
            r#"{"location":{"type":"minecraft:location_check","offsetY":1,"predicate":{"position":{"y":65.5}}}}"#,
            &context
        ));
        assert!(!matches(
            r#"{"location":{"type":"minecraft:location_check","offsetY":1,"predicate":{"block":{"blocks":"minecraft:copper_block"}}}}"#,
            &context
        ));
        assert!(!matches(
            r#"{"location":{"type":"minecraft:inverted","term":{"type":"minecraft:weather_check"}}}"#,
            &context
        ));
        assert!(!matches(condition, &TriggerContext::default()));
        assert!(matches_block_location(condition, &context));
        assert!(matches_block_location(
            r#"{"location":{"type":"minecraft:any_of","terms":[{"type":"minecraft:match_block","blocks":"minecraft:stone"},{"type":"minecraft:match_block","blocks":"minecraft:copper_block"}]}}"#,
            &context
        ));
        assert!(!matches_block_location(
            r#"{"location":{"type":"minecraft:match_block","blocks":"minecraft:stone"}}"#,
            &context
        ));
        assert!(!matches_block_location(
            r#"{"player":{"type":"minecraft:entity_properties"}}"#,
            &context
        ));
        assert!(!matches_block_location(
            r#"{"location":"minecraft:external_predicate"}"#,
            &context
        ));
    }

    #[test]
    fn exact_number_ranges_do_not_accept_nearby_values() {
        assert!(matches_number_range(&serde_json::json!(1.0), 1.0));
        assert!(!matches_number_range(&serde_json::json!(1.0), 1.000_000_1));
        assert!(matches_number_range(
            &serde_json::json!({"min": 1.0, "max": 2.0}),
            2.0
        ));
    }

    #[test]
    fn bee_nest_count_obeys_exact_and_bounded_ranges() {
        let mut context = TriggerContext {
            bees_inside: Some(3),
            ..TriggerContext::default()
        };
        assert!(matches(r#"{"num_bees_inside":{"min":3}}"#, &context));
        assert!(!matches(r#"{"num_bees_inside":2}"#, &context));
        context.bees_inside = Some(0);
        assert!(!matches(r#"{"num_bees_inside":{"min":3}}"#, &context));
        assert!(matches(r#"{"num_bees_inside":{"max":0}}"#, &context));
        context.bees_inside = None;
        assert!(!matches(r#"{"num_bees_inside":0}"#, &context));
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
