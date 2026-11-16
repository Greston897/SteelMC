//! Vanilla `ServerWaypointManager` + `WaypointTransmitter`: locator-bar transmission.
//!
//! Matches vanilla 26.3's connection rules:
//! - Gated on `minecraft:locator_bar`; turning it off untracks active waypoints.
//! - Visibility respects `min(source.waypoint_transmit_range, receiver.waypoint_receive_range)`,
//!   sneaking/spectator suppression on the source, and spectator omniscience on the receiver.
//! - Encodes `Vec3i` at close range, `Chunk` outside close view (`<= 332` blocks),
//!   and `Azimuth` beyond `REALLY_FAR_DISTANCE` (`332` blocks).
//! - Supports per-transmitter `WaypointIcon` overrides (style + RGB color) for `/waypoint`.

use std::collections::BTreeMap;
use std::sync::Arc;

use glam::DVec3;
use rustc_hash::FxHashSet;
use steel_protocol::packets::game::{CWaypoint, TrackedWaypointData, WaypointIcon};
use steel_registry::vanilla_attributes;
use steel_utils::locks::SyncMutex;
use steel_utils::types::GameType;
use uuid::Uuid;

use crate::entity::{Entity, LivingEntity as _};
use crate::player::Player;
use crate::world::World;

/// Vanilla `WaypointTransmitter.REALLY_FAR_DISTANCE` (in blocks).
pub const REALLY_FAR_DISTANCE: f64 = 332.0;

/// Minimum azimuth change (radians, ~0.5°) before vanilla sends an `UPDATE` packet.
const AZIMUTH_UPDATE_THRESHOLD: f32 = 0.008_726_646;

/// Vanilla default `waypoint_transmit_range` / `waypoint_receive_range`.
const DEFAULT_WAYPOINT_RANGE: f64 = 6.0e7;

/// Computes the exact positional payload vanilla sends from `source_pos` to `receiver_pos`.
#[must_use]
pub fn compute_waypoint_data(
    source_pos: DVec3,
    receiver_pos: DVec3,
    view_distance_chunks: i32,
) -> TrackedWaypointData {
    let dx = receiver_pos.x - source_pos.x;
    let dy = receiver_pos.y - source_pos.y;
    let dz = receiver_pos.z - source_pos.z;
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();

    let block_x = source_pos.x.floor() as i32;
    let block_y = source_pos.y.floor() as i32;
    let block_z = source_pos.z.floor() as i32;
    let src_chunk_x = block_x.div_euclid(16);
    let src_chunk_z = block_z.div_euclid(16);
    let recv_chunk_x = (receiver_pos.x.floor() as i32).div_euclid(16);
    let recv_chunk_z = (receiver_pos.z.floor() as i32).div_euclid(16);
    let in_view = (src_chunk_x - recv_chunk_x).abs() <= view_distance_chunks
        && (src_chunk_z - recv_chunk_z).abs() <= view_distance_chunks;

    if !in_view && dist > REALLY_FAR_DISTANCE {
        // Vanilla: `Vec3 direction = receiver.position().subtract(source.position()).rotateClockwise90()`
        // where `rotateClockwise90` maps `(x, y, z) -> (-z, y, x)`, then `atan2(z, x)`.
        let rot_x = -dz;
        let rot_z = dx;
        TrackedWaypointData::Azimuth {
            angle: (rot_z.atan2(rot_x)) as f32,
        }
    } else if !in_view {
        TrackedWaypointData::Chunk {
            chunk_x: src_chunk_x,
            chunk_z: src_chunk_z,
        }
    } else {
        TrackedWaypointData::Vec3i {
            x: block_x,
            y: block_y,
            z: block_z,
        }
    }
}

/// Returns whether a waypoint changed enough to emit an `UPDATE` packet.
#[must_use]
pub(super) fn needs_update(
    prev_data: TrackedWaypointData,
    prev_icon: &WaypointIcon,
    next_data: TrackedWaypointData,
    next_icon: &WaypointIcon,
) -> bool {
    if prev_icon != next_icon {
        return true;
    }
    match (prev_data, next_data) {
        (
            TrackedWaypointData::Azimuth { angle: a },
            TrackedWaypointData::Azimuth { angle: b },
        ) => {
            let mut diff = (b - a).rem_euclid(std::f32::consts::TAU);
            if diff > std::f32::consts::PI {
                diff = std::f32::consts::TAU - diff;
            }
            diff > AZIMUTH_UPDATE_THRESHOLD
        }
        (a, b) => a != b,
    }
}

/// Snapshot of one waypoint as last transmitted to an observer.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct SentWaypoint {
    pub(super) data: TrackedWaypointData,
    pub(super) icon: WaypointIcon,
}

/// Per-tick snapshot of an online player so `tick` reads attributes once (`O(N)`).
struct PlayerSnapshot {
    player: Arc<Player>,
    uuid: Uuid,
    pos: DVec3,
    is_alive: bool,
    is_spectator: bool,
    is_crouching: bool,
    transmit_range: f64,
    receive_range: f64,
    view_distance: i32,
    icon: WaypointIcon,
}

/// Tracks active locator-bar connections and custom waypoint icons in a world.
#[derive(Debug, Default)]
pub struct WaypointManager {
    /// Observer `UUID` → currently tracked source `UUID`s.
    pub(super) sent: SyncMutex<BTreeMap<Uuid, BTreeMap<Uuid, SentWaypoint>>>,
    /// Optional custom icon overrides configured via `/waypoint modify`.
    icons: SyncMutex<BTreeMap<Uuid, WaypointIcon>>,
}

impl WaypointManager {
    /// Creates an empty waypoint manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Forgets every waypoint transmitted to a disconnecting or transferring observer.
    pub fn forget_observer(&self, observer: Uuid) {
        self.sent.lock().remove(&observer);
    }

    /// Returns the effective [`WaypointIcon`] for `target`.
    #[must_use]
    pub fn icon_for(&self, target: Uuid) -> WaypointIcon {
        self.icons
            .lock()
            .get(&target)
            .cloned()
            .unwrap_or_default()
    }

    /// Sets or clears the RGB color override on `target`'s locator-bar icon.
    pub fn set_icon_color(&self, target: Uuid, color: Option<[u8; 3]>) {
        let mut icons = self.icons.lock();
        let entry = icons.entry(target).or_default();
        entry.color = color;
        if *entry == WaypointIcon::default() {
            icons.remove(&target);
        }
    }

    /// Sets the style asset identifier on `target`'s locator-bar icon.
    pub fn set_icon_style(&self, target: Uuid, style: steel_utils::Identifier) {
        let mut icons = self.icons.lock();
        let entry = icons.entry(target).or_default();
        entry.style = style;
        if *entry == WaypointIcon::default() {
            icons.remove(&target);
        }
    }

    /// Returns snapshots of `(uuid, icon)` for players currently transmitting waypoints.
    #[must_use]
    pub fn active_transmitters(&self, world: &World) -> Vec<(Uuid, String, WaypointIcon)> {
        if !world.get_game_rule(&steel_registry::vanilla_game_rules::LOCATOR_BAR) {
            return Vec::new();
        }
        let icons = self.icons.lock();
        let mut list = Vec::new();
        world.players.iter_players(|uuid, player| {
            if Entity::is_alive(player.as_ref())
                && player.game_mode() != GameType::Spectator
                && !player.is_crouching()
            {
                let transmit = player
                    .attributes()
                    .lock()
                    .get_value(vanilla_attributes::WAYPOINT_TRANSMIT_RANGE)
                    .unwrap_or(DEFAULT_WAYPOINT_RANGE);
                if transmit > 0.0 {
                    let icon = icons.get(uuid).cloned().unwrap_or_default();
                    list.push((*uuid, player.gameprofile.name().to_owned(), icon));
                }
            }
            true
        });
        list
    }

    /// Refreshes the locator bar of every player in `world`.
    pub fn tick(&self, world: &Arc<World>) {
        if !world.get_game_rule(&steel_registry::vanilla_game_rules::LOCATOR_BAR) {
            // Untrack any previously visible waypoints when the rule is turned off.
            let drained: Vec<(Uuid, Vec<Uuid>)> = {
                let mut sent = self.sent.lock();
                if sent.is_empty() {
                    return;
                }
                std::mem::take(&mut *sent)
                    .into_iter()
                    .map(|(obs, map)| (obs, map.into_keys().collect()))
                    .collect()
            };
            for (observer_uuid, targets) in drained {
                if let Some(observer) = world.players.get_by_uuid(&observer_uuid) {
                    for target in targets {
                        observer.send_packet(CWaypoint::untrack_uuid(target));
                    }
                }
            }
            return;
        }

        // Fast path: with 0 or 1 players online nobody has a peer to track.
        let mut snapshots = Vec::new();
        {
            let icons = self.icons.lock();
            world.players.iter_players(|uuid, player| {
                let (transmit_range, receive_range) = {
                    let attrs = player.attributes().lock();
                    (
                        attrs
                            .get_value(vanilla_attributes::WAYPOINT_TRANSMIT_RANGE)
                            .unwrap_or(DEFAULT_WAYPOINT_RANGE),
                        attrs
                            .get_value(vanilla_attributes::WAYPOINT_RECEIVE_RANGE)
                            .unwrap_or(DEFAULT_WAYPOINT_RANGE),
                    )
                };
                snapshots.push(PlayerSnapshot {
                    player: Arc::clone(player),
                    uuid: *uuid,
                    pos: player.position(),
                    is_alive: Entity::is_alive(player.as_ref()),
                    is_spectator: player.game_mode() == GameType::Spectator,
                    is_crouching: player.is_crouching(),
                    transmit_range,
                    receive_range,
                    view_distance: i32::from(player.view_distance()).max(2),
                    icon: icons.get(uuid).cloned().unwrap_or_default(),
                });
                true
            });
        }

        if snapshots.is_empty() {
            let mut sent = self.sent.lock();
            sent.clear();
            return;
        }

        let live: FxHashSet<Uuid> = snapshots.iter().map(|s| s.uuid).collect();
        let mut outgoing: Vec<(&Arc<Player>, CWaypoint)> = Vec::new();

        {
            let mut state = self.sent.lock();
            state.retain(|observer, _| live.contains(observer));

            for observer in &snapshots {
                let tracked = state.entry(observer.uuid).or_default();
                let mut still_valid = BTreeMap::new();

                for source in &snapshots {
                    if source.uuid == observer.uuid || !source.is_alive {
                        continue;
                    }
                    // Vanilla `WaypointTransmitter.doesSourceIgnoreReceiver` +
                    // `LivingEntity.isTransmittingWaypoint` (sneaking hides non-spectator targets).
                    if !observer.is_spectator {
                        if source.is_spectator || source.is_crouching || source.transmit_range <= 0.0
                        {
                            continue;
                        }
                        let max_range = source.transmit_range.min(observer.receive_range);
                        if max_range <= 0.0 {
                            continue;
                        }
                        let dist_sq = (source.pos - observer.pos).length_squared();
                        if dist_sq >= max_range * max_range {
                            continue;
                        }
                    }

                    let data = compute_waypoint_data(
                        source.pos,
                        observer.pos,
                        observer.view_distance,
                    );

                    match tracked.get(&source.uuid) {
                        None => {
                            outgoing.push((
                                &observer.player,
                                CWaypoint::track_uuid(source.uuid, source.icon.clone(), data),
                            ));
                            still_valid.insert(
                                source.uuid,
                                SentWaypoint {
                                    data,
                                    icon: source.icon.clone(),
                                },
                            );
                        }
                        Some(prev)
                            if needs_update(prev.data, &prev.icon, data, &source.icon) =>
                        {
                            // Switching representation type (Vec3i <-> Chunk <-> Azimuth)
                            // mirrors vanilla's connection transition cleanly via `Track`.
                            let same_kind =
                                std::mem::discriminant(&prev.data) == std::mem::discriminant(&data);
                            let packet = if same_kind {
                                CWaypoint::update_uuid(source.uuid, source.icon.clone(), data)
                            } else {
                                CWaypoint::track_uuid(source.uuid, source.icon.clone(), data)
                            };
                            outgoing.push((&observer.player, packet));
                            still_valid.insert(
                                source.uuid,
                                SentWaypoint {
                                    data,
                                    icon: source.icon.clone(),
                                },
                            );
                        }
                        Some(prev) => {
                            still_valid.insert(source.uuid, prev.clone());
                        }
                    }
                }

                for old_uuid in tracked.keys() {
                    if !still_valid.contains_key(old_uuid) {
                        outgoing.push((&observer.player, CWaypoint::untrack_uuid(*old_uuid)));
                    }
                }

                *tracked = still_valid;
            }
        }

        // Emit all packets after releasing `self.sent` to avoid lock contention.
        for (observer, packet) in outgoing {
            observer.send_packet(packet);
        }
    }
}

#[cfg(test)]
mod tests;
