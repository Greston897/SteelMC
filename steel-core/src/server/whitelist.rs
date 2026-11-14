//! Server whitelist backing the `/whitelist` command.
//!
//! Vanilla `Whitelist` keeps a set of game profiles in `whitelist.json` and asks
//! it at login when `white-list=true` is set in `server.properties`. Steel keeps
//! the same shape: UUIDs with their last known name, so `/whitelist list` can
//! print names without looking them up, and the join gate can answer by UUID
//! alone.
//!
//! The list is only consulted while it is enabled, which is the `enabled` flag
//! here. Vanilla's `enforce-whitelist` option, which also disconnects players who
//! were removed from the list while online, is not modelled.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use steel_utils::locks::SyncMutex;
use uuid::Uuid;

/// One whitelisted profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhitelistEntry {
    /// The player's UUID.
    pub uuid: Uuid,
    /// The name shown by `/whitelist list`.
    pub name: String,
}

/// The server's whitelist.
#[derive(Debug, Default)]
pub struct Whitelist {
    enabled: SyncMutex<bool>,
    entries: SyncMutex<BTreeMap<Uuid, String>>,
}

impl Whitelist {
    /// Creates an empty, disabled whitelist.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether the whitelist currently gates logins.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        *self.enabled.lock()
    }

    /// Turns the login gate on or off. Returns the previous value.
    pub fn set_enabled(&self, enabled: bool) -> bool {
        let mut current = self.enabled.lock();
        let previous = *current;
        *current = enabled;
        previous
    }

    /// Adds a player. Returns whether the UUID was already listed.
    pub fn add(&self, uuid: Uuid, name: impl Into<String>) -> bool {
        self.entries.lock().insert(uuid, name.into()).is_some()
    }

    /// Removes a player. Returns whether the UUID was listed.
    pub fn remove(&self, uuid: Uuid) -> bool {
        self.entries.lock().remove(&uuid).is_some()
    }

    /// Replaces the name stored for a listed player.
    ///
    /// Vanilla updates the stored name on every login, which keeps
    /// `/whitelist list` accurate after a rename. `false` means the player is
    /// not listed, so nothing was updated.
    pub fn update_name(&self, uuid: Uuid, name: impl Into<String>) -> bool {
        let mut entries = self.entries.lock();
        let Some(stored) = entries.get_mut(&uuid) else {
            return false;
        };
        *stored = name.into();
        true
    }

    /// Returns whether `uuid` may join.
    ///
    /// An empty whitelist rejects everyone while enabled, which is vanilla's
    /// behaviour and the reason `/whitelist add` needs the player's name rather
    /// than trusting that a later login will be allowed.
    #[must_use]
    pub fn allows(&self, uuid: Uuid) -> bool {
        if !self.is_enabled() {
            return true;
        }
        self.entries.lock().contains_key(&uuid)
    }

    /// Returns whether `uuid` is listed, regardless of whether the gate is on.
    #[must_use]
    pub fn contains(&self, uuid: Uuid) -> bool {
        self.entries.lock().contains_key(&uuid)
    }

    /// Every entry, sorted by name so `/whitelist list` is stable.
    #[must_use]
    pub fn entries(&self) -> Vec<WhitelistEntry> {
        let mut entries: Vec<WhitelistEntry> = self
            .entries
            .lock()
            .iter()
            .map(|(uuid, name)| WhitelistEntry {
                uuid: *uuid,
                name: name.clone(),
            })
            .collect();
        entries.sort_by(|left, right| left.name.cmp(&right.name));
        entries
    }

    /// The number of listed players.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.lock().len()
    }

    /// Whether nobody is listed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The persisted form: the enabled flag plus every entry.
    ///
    /// Vanilla writes `whitelist.json` as an array of `{uuid, name}` objects and
    /// keeps the enabled flag only in `server.properties`; Steel persists both
    /// together so an operator does not lose the gate on a restart.
    #[must_use]
    pub fn to_saved(&self) -> WhitelistSaved {
        WhitelistSaved {
            enabled: self.is_enabled(),
            entries: self.entries(),
        }
    }

    /// Replaces the contents from a saved snapshot.
    pub fn apply_saved(&self, saved: WhitelistSaved) {
        *self.enabled.lock() = saved.enabled;
        let mut entries = self.entries.lock();
        entries.clear();
        for entry in saved.entries {
            entries.insert(entry.uuid, entry.name);
        }
    }
}

/// On-disk form of the whitelist.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WhitelistSaved {
    /// Whether the whitelist gates logins.
    pub enabled: bool,
    /// Listed profiles.
    pub entries: Vec<WhitelistEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uuid(n: u8) -> Uuid {
        Uuid::from_bytes([n; 16])
    }

    #[test]
    fn a_disabled_whitelist_lets_everyone_in() {
        let whitelist = Whitelist::new();
        assert!(!whitelist.is_enabled());
        assert!(whitelist.allows(uuid(1)));
    }

    #[test]
    fn an_enabled_empty_whitelist_rejects_everyone() {
        let whitelist = Whitelist::new();
        whitelist.set_enabled(true);
        assert!(!whitelist.allows(uuid(1)));
    }

    #[test]
    fn an_enabled_whitelist_admits_listed_players_only() {
        let whitelist = Whitelist::new();
        whitelist.add(uuid(1), "Alice");
        whitelist.set_enabled(true);

        assert!(whitelist.allows(uuid(1)));
        assert!(!whitelist.allows(uuid(2)));
    }

    #[test]
    fn removing_a_player_takes_effect_immediately() {
        let whitelist = Whitelist::new();
        whitelist.add(uuid(1), "Alice");
        whitelist.set_enabled(true);

        assert!(whitelist.remove(uuid(1)));
        assert!(!whitelist.remove(uuid(1)));
        assert!(!whitelist.allows(uuid(1)));
    }

    #[test]
    fn adding_the_same_player_twice_reports_the_duplicate() {
        let whitelist = Whitelist::new();
        assert!(!whitelist.add(uuid(1), "Alice"));
        assert!(whitelist.add(uuid(1), "Alice"));
        assert_eq!(whitelist.len(), 1);
    }

    #[test]
    fn a_rename_updates_the_listed_name() {
        let whitelist = Whitelist::new();
        whitelist.add(uuid(1), "Alice");
        assert!(whitelist.update_name(uuid(1), "Alicia"));
        assert_eq!(whitelist.entries()[0].name, "Alicia");
        assert!(!whitelist.update_name(uuid(2), "Bob"));
    }

    #[test]
    fn entries_are_sorted_by_name() {
        let whitelist = Whitelist::new();
        whitelist.add(uuid(1), "Zoe");
        whitelist.add(uuid(2), "Alice");
        assert_eq!(
            whitelist
                .entries()
                .iter()
                .map(|entry| entry.name.as_str())
                .collect::<Vec<_>>(),
            vec!["Alice", "Zoe"]
        );
    }

    #[test]
    fn toggling_reports_the_previous_state() {
        let whitelist = Whitelist::new();
        assert!(!whitelist.set_enabled(true));
        assert!(whitelist.set_enabled(false));
        assert!(!whitelist.is_enabled());
    }
}
