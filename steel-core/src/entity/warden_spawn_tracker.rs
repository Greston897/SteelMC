//! Vanilla `WardenSpawnTracker`.
//!
//! The tracker is per player, not per block: a shriek asks every nearby player's
//! tracker whether it may warn, picks the highest warning level among them, bumps
//! it, and copies the result back to all of them. That is why two players
//! shrieking the same block reach level four in half the time.
//!
//! Steel stores the tracker on `Player` (vanilla stores it in the player's
//! optional attachment) and keeps the tick, the cooldown and the twelve
//! thousand-tick decay identical.

use std::sync::Arc;

use steel_utils::{BlockPos, WorldAabb};

use crate::entity::Entity as _;
use crate::player::Player;
use crate::world::World;

/// Vanilla `WardenSpawnTracker.MAX_WARNING_LEVEL`.
pub const MAX_WARNING_LEVEL: i32 = 4;
/// Vanilla `WardenSpawnTracker.PLAYER_SEARCH_RADIUS`: players that share a shriek.
const PLAYER_SEARCH_RADIUS: f64 = 16.0;
/// Vanilla `WardenSpawnTracker.WARNING_CHECK_DIAMETER`: no warden within this box.
const WARNING_CHECK_DIAMETER: f64 = 48.0;
/// Vanilla `WardenSpawnTracker.DECREASE_WARNING_LEVEL_EVERY_INTERVAL`.
const DECREASE_WARNING_LEVEL_EVERY_INTERVAL: i32 = 12_000;
/// Vanilla `WardenSpawnTracker.WARNING_LEVEL_INCREASE_COOLDOWN`.
const WARNING_LEVEL_INCREASE_COOLDOWN: i32 = 200;

/// Vanilla `WardenSpawnTracker`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WardenSpawnTracker {
    ticks_since_last_warning: i32,
    warning_level: i32,
    cooldown_ticks: i32,
}

impl WardenSpawnTracker {
    /// Creates an empty tracker.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            ticks_since_last_warning: 0,
            warning_level: 0,
            cooldown_ticks: 0,
        }
    }

    /// Creates a tracker with explicit state, as the vanilla codec does.
    #[must_use]
    pub const fn with_state(
        ticks_since_last_warning: i32,
        warning_level: i32,
        cooldown_ticks: i32,
    ) -> Self {
        Self {
            ticks_since_last_warning,
            warning_level,
            cooldown_ticks,
        }
    }

    /// Vanilla `WardenSpawnTracker.tick`: decay one level per twelve thousand
    /// ticks of quiet and count the warning cooldown down.
    pub fn tick(&mut self) {
        if self.ticks_since_last_warning >= DECREASE_WARNING_LEVEL_EVERY_INTERVAL {
            self.decrease_warning_level();
            self.ticks_since_last_warning = 0;
        } else {
            self.ticks_since_last_warning += 1;
        }
        if self.cooldown_ticks > 0 {
            self.cooldown_ticks -= 1;
        }
    }

    /// Vanilla `WardenSpawnTracker.reset`.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Vanilla `getWarningLevel`.
    #[must_use]
    pub const fn warning_level(self) -> i32 {
        self.warning_level
    }

    /// Vanilla `setWarningLevel`, clamped to `0..=4`.
    pub fn set_warning_level(&mut self, warning_level: i32) {
        self.warning_level = warning_level.clamp(0, MAX_WARNING_LEVEL);
    }

    /// Vanilla `ticks_since_last_warning`.
    #[must_use]
    pub const fn ticks_since_last_warning(self) -> i32 {
        self.ticks_since_last_warning
    }

    /// Vanilla `cooldown_ticks`.
    #[must_use]
    pub const fn cooldown_ticks(self) -> i32 {
        self.cooldown_ticks
    }

    /// Vanilla `onCooldown`.
    #[must_use]
    pub const fn on_cooldown(self) -> bool {
        self.cooldown_ticks > 0
    }

    /// Vanilla `WardenSpawnTracker.tryWarn`: returns the new warning level when a
    /// shriek at `pos` may raise it, `None` when it may not.
    pub fn try_warn(world: &Arc<World>, pos: BlockPos, trigger: &Arc<Player>) -> Option<i32> {
        if has_nearby_warden(world, pos) {
            return None;
        }

        let mut players = nearby_players(world, pos);
        if !players.iter().any(|player| Arc::ptr_eq(player, trigger)) {
            players.push(Arc::clone(trigger));
        }

        if players
            .iter()
            .any(|player| player.warden_spawn_tracker().on_cooldown())
        {
            return None;
        }

        let highest = players
            .iter()
            .map(|player| player.warden_spawn_tracker())
            .max_by_key(|tracker| tracker.warning_level())?;

        let mut tracker = highest;
        tracker.increase_warning_level();
        let warned = tracker.warning_level();
        for player in &players {
            player.set_warden_spawn_tracker(tracker);
        }
        Some(warned)
    }

    /// Vanilla `increaseWarningLevel`: ignored while on cooldown.
    fn increase_warning_level(&mut self) {
        if self.on_cooldown() {
            return;
        }
        self.ticks_since_last_warning = 0;
        self.cooldown_ticks = WARNING_LEVEL_INCREASE_COOLDOWN;
        self.set_warning_level(self.warning_level + 1);
    }

    /// Vanilla `decreaseWarningLevel`.
    fn decrease_warning_level(&mut self) {
        self.set_warning_level(self.warning_level - 1);
    }
}

/// Vanilla `hasNearbyWarden`: no warden inside a forty-eight block cube.
fn has_nearby_warden(world: &Arc<World>, pos: BlockPos) -> bool {
    use steel_registry::vanilla_entities;

    let center = glam::DVec3::new(
        f64::from(pos.x()) + 0.5,
        f64::from(pos.y()) + 0.5,
        f64::from(pos.z()) + 0.5,
    );
    let area = WorldAabb::of_size(
        center,
        WARNING_CHECK_DIAMETER,
        WARNING_CHECK_DIAMETER,
        WARNING_CHECK_DIAMETER,
    );
    world
        .get_entities_in_aabb(&area)
        .into_iter()
        .any(|entity| entity.entity_type() == &vanilla_entities::WARDEN)
}

/// Vanilla `getNearbyPlayers`: alive, non-spectator players within sixteen
/// blocks of the shrieker's centre.
fn nearby_players(world: &Arc<World>, pos: BlockPos) -> Vec<Arc<Player>> {
    let center = glam::DVec3::new(
        f64::from(pos.x()) + 0.5,
        f64::from(pos.y()) + 0.5,
        f64::from(pos.z()) + 0.5,
    );
    let mut uuids = Vec::new();
    world.players.iter_players(|uuid, player| {
        if !player.is_spectator()
            && player.is_alive()
            && player.position().distance(center) <= PLAYER_SEARCH_RADIUS
        {
            uuids.push(*uuid);
        }
        true
    });
    uuids
        .into_iter()
        .filter_map(|uuid| world.players.get_by_uuid(&uuid))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_tracker_starts_at_zero() {
        let tracker = WardenSpawnTracker::new();
        assert_eq!(tracker.warning_level(), 0);
        assert_eq!(tracker.cooldown_ticks(), 0);
        assert!(!tracker.on_cooldown());
    }

    #[test]
    fn the_warning_level_saturates_at_four() {
        let mut tracker = WardenSpawnTracker::new();
        for _ in 0..10 {
            tracker.ticks_since_last_warning = 0;
            tracker.cooldown_ticks = 0;
            tracker.increase_warning_level();
        }
        assert_eq!(tracker.warning_level(), MAX_WARNING_LEVEL);
    }

    #[test]
    fn the_cooldown_refuses_a_second_warning() {
        let mut tracker = WardenSpawnTracker::new();
        tracker.increase_warning_level();
        assert_eq!(tracker.warning_level(), 1);
        assert!(tracker.on_cooldown());
        tracker.increase_warning_level();
        assert_eq!(tracker.warning_level(), 1, "still inside the cooldown");
    }

    #[test]
    fn a_level_decays_after_twelve_thousand_quiet_ticks() {
        let mut tracker = WardenSpawnTracker::with_state(0, 3, 0);
        for _ in 0..DECREASE_WARNING_LEVEL_EVERY_INTERVAL {
            tracker.tick();
        }
        // The interval is reached on the tick after the counter passes it.
        tracker.tick();
        assert_eq!(tracker.warning_level(), 2);
        assert_eq!(tracker.ticks_since_last_warning(), 1);
    }

    #[test]
    fn the_cooldown_counts_down() {
        let mut tracker = WardenSpawnTracker::new();
        tracker.increase_warning_level();
        for _ in 0..WARNING_LEVEL_INCREASE_COOLDOWN {
            tracker.tick();
        }
        assert!(!tracker.on_cooldown());
    }
}
