//! Tests for locator-bar waypoint transmission.

use std::collections::BTreeMap;

use steel_utils::Identifier;
use uuid::Uuid;

use super::{PLAYER_SOURCE, SentWaypoint, WaypointManager};

#[test]
fn new_manager_tracks_nothing() {
    let manager = WaypointManager::new();
    assert!(manager.sent.lock().is_empty());
}

#[test]
fn forgetting_an_observer_drops_its_tracking_state() {
    let manager = WaypointManager::new();
    let observer = Uuid::from_u128(1);
    let target = Uuid::from_u128(2);

    manager
        .sent
        .lock()
        .entry(observer)
        .or_default()
        .insert(target, SentWaypoint { x: 0.0, z: 0.0 });

    manager.forget_observer(observer);

    assert!(manager.sent.lock().get(&observer).is_none());
}

#[test]
fn forgetting_an_unknown_observer_is_a_no_op() {
    let manager = WaypointManager::new();
    manager.forget_observer(Uuid::from_u128(42));
    assert!(manager.sent.lock().is_empty());
}

#[test]
fn player_source_identifier_is_vanilla_player_key() {
    assert_eq!(
        PLAYER_SOURCE,
        Identifier::vanilla_static("player"),
        "locator bar sources must match vanilla so clients group them"
    );
}

#[test]
fn block_granularity_update_check_ignores_sub_block_motion() {
    // Vanilla only refreshes a waypoint when it crosses a block boundary; the
    // comparison drives the `update` packet suppression.
    let previous = SentWaypoint { x: 10.2, z: -4.7 };
    let same_block = SentWaypoint { x: 10.9, z: -4.1 };
    let next_block = SentWaypoint { x: 11.1, z: -4.7 };

    assert_eq!(previous.x.floor(), same_block.x.floor());
    assert_eq!(previous.z.floor(), same_block.z.floor());
    assert_ne!(previous.x.floor(), next_block.x.floor());
}

#[test]
fn tracked_state_preserves_ordering() {
    let manager = WaypointManager::new();
    let observer = Uuid::from_u128(7);
    let mut tracked = BTreeMap::new();
    tracked.insert(Uuid::from_u128(9), SentWaypoint { x: 1.0, z: 2.0 });
    tracked.insert(Uuid::from_u128(8), SentWaypoint { x: 3.0, z: 4.0 });
    manager.sent.lock().insert(observer, tracked);

    let state = manager.sent.lock();
    let keys: Vec<Uuid> = state[&observer].keys().copied().collect();
    assert_eq!(keys, vec![Uuid::from_u128(8), Uuid::from_u128(9)]);
}
