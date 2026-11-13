//! Vanilla `WaypointManager`: per-player locator-bar transmission.
//!
//! Vanilla tracks every other player as a waypoint whose visibility is bounded by
//! the observer's `waypoint_transmit_range` attribute. Steel keeps the same shape:
//! each world owns the transmitted set, and a tick refreshes it so attribute
//! changes (armour, effects, commands) take effect without extra plumbing.

use std::collections::BTreeMap;
use std::sync::Arc;

use steel_protocol::packets::game::{CWaypoint, WaypointData, WaypointOperation};
use steel_registry::vanilla_attributes;
use steel_utils::Identifier;
use steel_utils::locks::SyncMutex;

use crate::entity::{Entity, LivingEntity as _};
use crate::player::Player;
use crate::world::World;

/// Transmission source key vanilla uses for player-tracked waypoints.
pub(super) const PLAYER_SOURCE: Identifier = Identifier::vanilla_static("player");

/// Vanilla default `waypoint_transmit_range`.
const DEFAULT_TRANSMIT_RANGE: f64 = 6.0e7;

/// One transmitted waypoint as last sent to an observer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SentWaypoint {
    x: f64,
    z: f64,
}

/// Tracks which waypoint each observer already received.
#[derive(Debug, Default)]
pub struct WaypointManager {
    /// Observer `UUID` to the waypoints currently displayed for them.
    pub(super) sent: SyncMutex<BTreeMap<uuid::Uuid, BTreeMap<uuid::Uuid, SentWaypoint>>>,
}

impl WaypointManager {
    /// Creates an empty manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Forgets every waypoint transmitted to a leaving observer.
    pub fn forget_observer(&self, observer: uuid::Uuid) {
        self.sent.lock().remove(&observer);
    }

    /// Refreshes the locator bar of every player in `world`.
    ///
    /// Called once per world tick; the diff keeps packet volume at vanilla levels
    /// by only emitting `track`/`update`/`untrack` when the state actually moved.
    pub fn tick(&self, world: &Arc<World>) {
        let mut observers = Vec::new();
        world.players.iter_players(|_uuid, player| {
            observers.push(Arc::clone(player));
            true
        });

        for observer in &observers {
            self.tick_observer(world, observer);
        }

        // Observers that left the world keep no state, matching vanilla's
        // per-connection manager lifetime.
        let live: Vec<uuid::Uuid> = observers.iter().map(|player| player.uuid()).collect();
        self.sent
            .lock()
            .retain(|observer, _| live.contains(observer));
    }

    fn tick_observer(&self, world: &Arc<World>, observer: &Arc<Player>) {
        let observer_uuid = observer.base().uuid();
        let observer_pos = observer.position();
        let range = observer
            .attributes()
            .lock()
            .get_value(vanilla_attributes::WAYPOINT_TRANSMIT_RANGE)
            .unwrap_or(DEFAULT_TRANSMIT_RANGE);

        let mut targets = Vec::new();
        world.players.iter_players(|uuid, player| {
            if player.base().uuid() != observer_uuid && Entity::is_alive(player.as_ref()) {
                targets.push((*uuid, player.position()));
            }
            true
        });

        let mut state = self.sent.lock();
        let tracked = state.entry(observer_uuid).or_default();
        let mut still_valid = BTreeMap::new();

        for (uuid, position) in targets {
            let distance_sq = (position.x - observer_pos.x).powi(2)
                + (position.y - observer_pos.y).powi(2)
                + (position.z - observer_pos.z).powi(2);
            if distance_sq > range * range {
                continue;
            }

            let waypoint = WaypointData {
                uuid,
                x: position.x,
                z: position.z,
                chunk_x: (position.x.floor() / 16.0) as i32,
                chunk_z: (position.z.floor() / 16.0) as i32,
            };
            let sent = SentWaypoint {
                x: position.x,
                z: position.z,
            };

            let packet = match tracked.get(&uuid) {
                None => CWaypoint {
                    operation: WaypointOperation::Track {
                        source: PLAYER_SOURCE,
                        icon: 0,
                        color: 0xFFFF_FFFFu32 as i32,
                        waypoint,
                    },
                },
                // Vanilla throttles updates to a moved block position; the locator
                // bar reads block granularity, so sub-block motion is skipped.
                Some(previous)
                    if previous.x.floor() == position.x.floor()
                        && previous.z.floor() == position.z.floor() =>
                {
                    still_valid.insert(uuid, *previous);
                    continue;
                }
                Some(_) => CWaypoint {
                    operation: WaypointOperation::Update {
                        source: PLAYER_SOURCE,
                        waypoint,
                    },
                },
            };

            observer.send_packet(packet);
            still_valid.insert(uuid, sent);
        }

        for uuid in tracked.keys() {
            if !still_valid.contains_key(uuid) {
                observer.send_packet(CWaypoint {
                    operation: WaypointOperation::Untrack {
                        source: PLAYER_SOURCE,
                    },
                });
            }
        }

        *tracked = still_valid;
    }
}

#[cfg(test)]
mod tests;
