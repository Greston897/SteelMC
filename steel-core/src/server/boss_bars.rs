//! Server-side boss bar registry backing the `/bossbar` command.
//!
//! Vanilla keeps these in `ServerBossEvent`s owned by whoever created them and
//! relies on `MinecraftServer.getAllLevels()` to find viewers. Steel has a flat
//! online-player map, so bars are registered here by name and the command
//! resolves their viewers through entity selectors.

use std::collections::BTreeMap;
use std::sync::Arc;

use steel_protocol::packets::game::{BossBarColor, BossBarOverlay, CBossEvent};
use steel_utils::locks::SyncMutex;
use text_components::TextComponent;
use uuid::Uuid;

use crate::player::Player;

/// One registered boss bar.
#[derive(Debug, Clone)]
pub struct BossBar {
    /// Wire identity used by every packet for this bar.
    pub uuid: Uuid,
    /// Displayed title.
    pub name: TextComponent,
    /// Progress in `0.0..=1.0`.
    pub progress: f32,
    /// Bar colour.
    pub color: BossBarColor,
    /// Bar overlay style.
    pub overlay: BossBarOverlay,
    /// Vanilla `BossEvent` flags.
    pub flags: u8,
    /// Whether the bar is currently visible to players.
    pub visible: bool,
}

impl BossBar {
    /// Builds the `add` packet for a player that starts seeing this bar.
    #[must_use]
    pub fn add_packet(&self) -> CBossEvent {
        CBossEvent::add(
            self.uuid,
            self.name.clone(),
            self.progress,
            self.color,
            self.overlay,
            self.flags,
        )
    }

    /// Builds the `remove` packet for a player that stops seeing this bar.
    #[must_use]
    pub fn remove_packet(&self) -> CBossEvent {
        CBossEvent::remove(self.uuid)
    }
}

/// Registry of every live boss bar, keyed by command-visible id.
#[derive(Debug, Default)]
pub struct BossBarRegistry {
    bars: SyncMutex<BTreeMap<String, BossBar>>,
}

impl BossBarRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new bar, or replaces the definition of an existing id.
    ///
    /// Returns the previous definition when the id was already in use, so the
    /// caller can decide whether existing viewers need re-syncing.
    pub fn insert(&self, id: String, bar: BossBar) -> Option<BossBar> {
        self.bars.lock().insert(id, bar)
    }

    /// Removes a bar and returns its definition.
    pub fn remove(&self, id: &str) -> Option<BossBar> {
        self.bars.lock().remove(id)
    }

    /// Returns a copy of the named bar.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<BossBar> {
        self.bars.lock().get(id).cloned()
    }

    /// Returns the definitions of all registered bars, sorted by id.
    #[must_use]
    pub fn all(&self) -> Vec<(String, BossBar)> {
        self.bars
            .lock()
            .iter()
            .map(|(id, bar)| (id.clone(), bar.clone()))
            .collect()
    }

    /// Returns the ids of all registered bars, sorted.
    #[must_use]
    pub fn ids(&self) -> Vec<String> {
        self.bars.lock().keys().cloned().collect()
    }

    /// Mutates the bar with `id` in place and returns a copy of the result.
    ///
    /// Returns `None` when the id is unknown.
    pub fn update<F>(&self, id: &str, f: F) -> Option<BossBar>
    where
        F: FnOnce(&mut BossBar),
    {
        let mut bars = self.bars.lock();
        let bar = bars.get_mut(id)?;
        f(bar);
        Some(bar.clone())
    }

    /// Sends the `add` action for `bar` to every player in `targets` that does
    /// not already show it, and the `remove` action to players that should no
    /// longer see it.
    ///
    /// `showing` is the caller-owned set of players that currently display the
    /// bar; this method keeps it in sync so later updates only send deltas.
    pub fn sync_viewers(bar: &BossBar, targets: &[Arc<Player>], showing: &mut Vec<Uuid>) {
        for player in targets {
            let uuid = player.base().uuid();
            let already = showing.contains(&uuid);
            if bar.visible && !already {
                player.send_packet(bar.add_packet());
                showing.push(uuid);
            } else if !bar.visible && already {
                player.send_packet(bar.remove_packet());
                showing.retain(|shown| *shown != uuid);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(name: &str) -> BossBar {
        BossBar {
            uuid: Uuid::from_u128(7),
            name: TextComponent::plain(name.to_string()),
            progress: 0.5,
            color: BossBarColor::Blue,
            overlay: BossBarOverlay::Progress,
            flags: 0,
            visible: true,
        }
    }

    #[test]
    fn insert_get_and_remove_round_trip() {
        let registry = BossBarRegistry::new();
        assert!(registry.insert("demo".to_string(), bar("Demo")).is_none());
        assert_eq!(registry.ids(), vec!["demo".to_string()]);

        let stored = registry.get("demo").expect("bar should exist");
        assert_eq!(stored.progress, 0.5);

        let replaced = registry.insert("demo".to_string(), bar("Second"));
        assert!(replaced.is_some(), "replacing must report the old definition");

        assert!(registry.remove("demo").is_some());
        assert!(registry.get("demo").is_none());
    }

    #[test]
    fn update_reports_unknown_ids() {
        let registry = BossBarRegistry::new();
        assert!(registry.update("missing", |bar| bar.progress = 1.0).is_none());

        registry.insert("demo".to_string(), bar("Demo"));
        let updated = registry
            .update("demo", |bar| bar.progress = 1.0)
            .expect("known id");
        assert_eq!(updated.progress, 1.0);
    }

    #[test]
    fn all_lists_bars_sorted_by_id() {
        let registry = BossBarRegistry::new();
        registry.insert("zeta".to_string(), bar("Z"));
        registry.insert("alpha".to_string(), bar("A"));

        let ids: Vec<String> = registry.all().into_iter().map(|(id, _)| id).collect();
        assert_eq!(ids, vec!["alpha".to_string(), "zeta".to_string()]);
    }
}
