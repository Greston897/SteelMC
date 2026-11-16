//! Tests for locator-bar waypoint transmission.

use std::collections::BTreeMap;

use glam::DVec3;
use steel_protocol::packets::game::{TrackedWaypointData, WaypointIcon};
use steel_utils::Identifier;
use uuid::Uuid;

use super::{SentWaypoint, WaypointManager, compute_waypoint_data, needs_update};

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

    manager.sent.lock().entry(observer).or_default().insert(
        target,
        SentWaypoint {
            data: TrackedWaypointData::Vec3i { x: 0, y: 64, z: 0 },
            icon: WaypointIcon::default(),
        },
    );

    manager.forget_observer(observer);
    assert!(!manager.sent.lock().contains_key(&observer));
}

#[test]
fn distance_tiers_select_vec3i_chunk_and_azimuth_with_floor_chunk_division() {
    // Nearby in view distance -> exact Vec3i
    assert_eq!(
        compute_waypoint_data(DVec3::new(10.8, 64.2, -3.2), DVec3::new(0.0, 64.0, 0.0), 8),
        TrackedWaypointData::Vec3i {
            x: 10,
            y: 64,
            z: -4
        }
    );

    // Outside 8-chunk view distance, <= 332 blocks, negative coords -> Euclidean chunk (-13, -1)
    assert_eq!(
        compute_waypoint_data(DVec3::new(-200.1, 64.0, -0.5), DVec3::new(0.0, 64.0, 0.0), 8),
        TrackedWaypointData::Chunk {
            chunk_x: -13,
            chunk_z: -1
        }
    );

    // Beyond 332 blocks -> Azimuth
    assert!(matches!(
        compute_waypoint_data(DVec3::new(500.0, 64.0, 0.0), DVec3::new(0.0, 64.0, 0.0), 8),
        TrackedWaypointData::Azimuth { .. }
    ));
}

#[test]
fn update_suppression_handles_block_azimuth_and_icon_changes() {
    let icon = WaypointIcon::default();
    let p = TrackedWaypointData::Vec3i { x: 1, y: 64, z: 2 };
    assert!(!needs_update(p, &icon, p, &icon));
    assert!(needs_update(
        p,
        &icon,
        TrackedWaypointData::Vec3i { x: 2, y: 64, z: 2 },
        &icon
    ));

    let tinted = WaypointIcon {
        style: Identifier::vanilla_static("bowtie"),
        color: Some([255, 0, 0]),
    };
    assert!(needs_update(p, &icon, p, &tinted));

    let a1 = TrackedWaypointData::Azimuth { angle: 1.0 };
    let a2 = TrackedWaypointData::Azimuth { angle: 1.001 };
    let a3 = TrackedWaypointData::Azimuth { angle: 1.1 };
    assert!(!needs_update(a1, &icon, a2, &icon));
    assert!(needs_update(a1, &icon, a3, &icon));
}

#[test]
fn icon_color_and_style_overrides_clean_up_default_entries() {
    let manager = WaypointManager::new();
    let id = Uuid::from_u128(99);
    assert_eq!(manager.icon_for(id), WaypointIcon::default());

    manager.set_icon_color(id, Some([10, 20, 30]));
    manager.set_icon_style(id, Identifier::vanilla_static("bowtie"));
    assert_eq!(
        manager.icon_for(id),
        WaypointIcon {
            style: Identifier::vanilla_static("bowtie"),
            color: Some([10, 20, 30]),
        }
    );

    manager.set_icon_color(id, None);
    manager.set_icon_style(id, Identifier::vanilla_static("default"));
    assert_eq!(manager.icon_for(id), WaypointIcon::default());
}

#[test]
fn tracked_state_preserves_ordering() {
    let manager = WaypointManager::new();
    let observer = Uuid::from_u128(7);
    let mut tracked = BTreeMap::new();
    let sample = SentWaypoint {
        data: TrackedWaypointData::Vec3i { x: 1, y: 0, z: 2 },
        icon: WaypointIcon::default(),
    };
    tracked.insert(Uuid::from_u128(9), sample.clone());
    tracked.insert(Uuid::from_u128(8), sample);
    manager.sent.lock().insert(observer, tracked);

    let state = manager.sent.lock();
    let keys: Vec<Uuid> = state[&observer].keys().copied().collect();
    assert_eq!(keys, vec![Uuid::from_u128(8), Uuid::from_u128(9)]);
}
