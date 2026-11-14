//! Player and address bans backing `/ban`, `/banip`, `/pardon` and `/banlist`.
//!
//! Vanilla keeps two lists in `banned-players.json` and `banned-ips.json`, each
//! entry carrying the reason, who issued the ban and when it expires. The lists
//! are consulted before a player is admitted, and a matching ban drops the
//! connection with the ban screen text.
//!
//! Steel keeps the same shape but no persistence yet: the server's domain data
//! does not have a slot for these lists, so bans last until the process exits.
//! Everything else — the reason, the issuer, the expiration and the IP list —
//! behaves like vanilla, including expiry, which is checked when the ban is
//! consulted rather than by a timer.

use std::collections::BTreeMap;
use std::net::IpAddr;

use serde::{Deserialize, Serialize};
use steel_utils::locks::SyncMutex;
use uuid::Uuid;

/// One ban entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BanEntry {
    /// The banned player, or `None` for an address ban.
    pub uuid: Option<Uuid>,
    /// The banned address, or `None` for a player ban.
    pub address: Option<IpAddr>,
    /// The name to show in the ban screen and `/banlist`.
    pub name: String,
    /// Why the player or address was banned.
    pub reason: String,
    /// Who issued the ban.
    pub source: String,
    /// When the ban was issued, as a UNIX timestamp in seconds.
    pub created: i64,
    /// When the ban stops applying, or `None` for a permanent ban.
    pub expires: Option<i64>,
}

impl BanEntry {
    /// Builds a ban for a player.
    #[must_use]
    pub fn player(
        uuid: Uuid,
        name: impl Into<String>,
        reason: impl Into<String>,
        source: impl Into<String>,
        created: i64,
        expires: Option<i64>,
    ) -> Self {
        Self {
            uuid: Some(uuid),
            address: None,
            name: name.into(),
            reason: reason.into(),
            source: source.into(),
            created,
            expires,
        }
    }

    /// Builds a ban for an address.
    #[must_use]
    pub fn address(
        address: IpAddr,
        name: impl Into<String>,
        reason: impl Into<String>,
        source: impl Into<String>,
        created: i64,
        expires: Option<i64>,
    ) -> Self {
        Self {
            uuid: None,
            address: Some(address),
            name: name.into(),
            reason: reason.into(),
            source: source.into(),
            created,
            expires,
        }
    }

    /// Whether the ban has expired at `now`.
    #[must_use]
    pub fn is_expired(&self, now: i64) -> bool {
        self.expires.is_some_and(|expires| now >= expires)
    }

    /// The ban screen's reason line, using vanilla's default when none was given.
    #[must_use]
    pub fn display_reason(&self) -> &str {
        if self.reason.is_empty() {
            "Banned by an operator."
        } else {
            &self.reason
        }
    }
}

/// The server's player and address bans.
#[derive(Debug, Default)]
pub struct BanList {
    players: SyncMutex<BTreeMap<Uuid, BanEntry>>,
    addresses: SyncMutex<BTreeMap<IpAddr, BanEntry>>,
}

impl BanList {
    /// Creates an empty ban list.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Bans a player, replacing any earlier ban for the same UUID.
    pub fn ban_player(&self, entry: BanEntry) {
        if let Some(uuid) = entry.uuid {
            self.players.lock().insert(uuid, entry);
        }
    }

    /// Bans an address, replacing any earlier ban for the same address.
    pub fn ban_address(&self, entry: BanEntry) {
        if let Some(address) = entry.address {
            self.addresses.lock().insert(address, entry);
        }
    }

    /// Lifts a player ban. Returns whether one was present.
    pub fn pardon(&self, uuid: Uuid) -> bool {
        self.players.lock().remove(&uuid).is_some()
    }

    /// Lifts an address ban. Returns whether one was present.
    pub fn pardon_address(&self, address: IpAddr) -> bool {
        self.addresses.lock().remove(&address).is_some()
    }

    /// Returns the ban that applies to `uuid` at `now`, if one does.
    ///
    /// An expired ban is removed rather than merely ignored, so `/banlist`
    /// never shows a ban that no longer applies.
    #[must_use]
    pub fn active_player_ban(&self, uuid: Uuid, now: i64) -> Option<BanEntry> {
        let mut players = self.players.lock();
        let entry = players.get(&uuid)?;
        if entry.is_expired(now) {
            players.remove(&uuid);
            return None;
        }
        Some(entry.clone())
    }

    /// Returns the ban that applies to `address` at `now`, if one does.
    #[must_use]
    pub fn active_address_ban(&self, address: IpAddr, now: i64) -> Option<BanEntry> {
        let mut addresses = self.addresses.lock();
        let entry = addresses.get(&address)?;
        if entry.is_expired(now) {
            addresses.remove(&address);
            return None;
        }
        Some(entry.clone())
    }

    /// Whether `uuid` is banned at `now`.
    #[must_use]
    pub fn is_banned(&self, uuid: Uuid, now: i64) -> bool {
        self.active_player_ban(uuid, now).is_some()
    }

    /// Whether `address` is banned at `now`.
    #[must_use]
    pub fn is_address_banned(&self, address: IpAddr, now: i64) -> bool {
        self.active_address_ban(address, now).is_some()
    }

    /// Every player ban, sorted by name.
    #[must_use]
    pub fn player_bans(&self) -> Vec<BanEntry> {
        let mut entries: Vec<BanEntry> = self.players.lock().values().cloned().collect();
        entries.sort_by(|left, right| left.name.cmp(&right.name));
        entries
    }

    /// Every address ban, sorted by address.
    #[must_use]
    pub fn address_bans(&self) -> Vec<BanEntry> {
        let mut entries: Vec<BanEntry> = self.addresses.lock().values().cloned().collect();
        entries.sort_by(|left, right| left.name.cmp(&right.name));
        entries
    }

    /// The number of player bans.
    #[must_use]
    pub fn player_ban_count(&self) -> usize {
        self.players.lock().len()
    }

    /// The number of address bans.
    #[must_use]
    pub fn address_ban_count(&self) -> usize {
        self.addresses.lock().len()
    }

    /// The persisted form: both lists in one snapshot.
    #[must_use]
    pub fn to_saved(&self) -> BanListSaved {
        BanListSaved {
            players: self.player_bans(),
            addresses: self.address_bans(),
        }
    }

    /// Replaces both lists from a saved snapshot.
    pub fn apply_saved(&self, saved: BanListSaved) {
        let mut players = self.players.lock();
        players.clear();
        for entry in saved.players {
            if let Some(uuid) = entry.uuid {
                players.insert(uuid, entry);
            }
        }
        drop(players);
        let mut addresses = self.addresses.lock();
        addresses.clear();
        for entry in saved.addresses {
            if let Some(address) = entry.address {
                addresses.insert(address, entry);
            }
        }
    }
}

/// On-disk form of the ban lists.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BanListSaved {
    /// Player bans, keyed by UUID at load time.
    pub players: Vec<BanEntry>,
    /// Address bans, keyed by address at load time.
    pub addresses: Vec<BanEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uuid(n: u8) -> Uuid {
        Uuid::from_bytes([n; 16])
    }

    fn ip(text: &str) -> IpAddr {
        text.parse().expect("valid test address")
    }

    #[test]
    fn a_banned_player_is_rejected() {
        let bans = BanList::new();
        bans.ban_player(BanEntry::player(
            uuid(1),
            "Alice",
            "griefing",
            "console",
            100,
            None,
        ));
        assert!(bans.is_banned(uuid(1), 100));
        assert!(!bans.is_banned(uuid(2), 100));
    }

    #[test]
    fn a_pardon_lets_the_player_back_in() {
        let bans = BanList::new();
        bans.ban_player(BanEntry::player(uuid(1), "Alice", "", "console", 0, None));
        assert!(bans.pardon(uuid(1)));
        assert!(!bans.pardon(uuid(1)));
        assert!(!bans.is_banned(uuid(1), 0));
    }

    #[test]
    fn a_temporary_ban_expires_on_its_own() {
        let bans = BanList::new();
        bans.ban_player(BanEntry::player(
            uuid(1),
            "Alice",
            "",
            "console",
            100,
            Some(200),
        ));
        assert!(bans.is_banned(uuid(1), 199));
        assert!(!bans.is_banned(uuid(1), 200));
        // The expired entry is discarded, not just hidden.
        assert_eq!(bans.player_ban_count(), 0);
    }

    #[test]
    fn a_second_ban_replaces_the_first() {
        let bans = BanList::new();
        bans.ban_player(BanEntry::player(uuid(1), "Alice", "one", "console", 0, None));
        bans.ban_player(BanEntry::player(uuid(1), "Alice", "two", "console", 0, None));
        assert_eq!(bans.player_ban_count(), 1);
        assert_eq!(bans.player_bans()[0].reason, "two");
    }

    #[test]
    fn address_bans_are_separate_from_player_bans() {
        let bans = BanList::new();
        bans.ban_address(BanEntry::address(
            ip("203.0.113.7"),
            "203.0.113.7",
            "",
            "console",
            0,
            None,
        ));
        assert!(bans.is_address_banned(ip("203.0.113.7"), 0));
        assert!(!bans.is_banned(uuid(1), 0));
        assert!(bans.pardon_address(ip("203.0.113.7")));
        assert!(!bans.is_address_banned(ip("203.0.113.7"), 0));
    }

    #[test]
    fn an_empty_reason_uses_the_vanilla_default() {
        let entry = BanEntry::player(uuid(1), "Alice", "", "console", 0, None);
        assert_eq!(entry.display_reason(), "Banned by an operator.");
        let entry = BanEntry::player(uuid(1), "Alice", "spam", "console", 0, None);
        assert_eq!(entry.display_reason(), "spam");
    }

    #[test]
    fn listings_are_sorted_by_name() {
        let bans = BanList::new();
        bans.ban_player(BanEntry::player(uuid(1), "Zoe", "", "console", 0, None));
        bans.ban_player(BanEntry::player(uuid(2), "Alice", "", "console", 0, None));
        assert_eq!(
            bans.player_bans()
                .iter()
                .map(|entry| entry.name.as_str())
                .collect::<Vec<_>>(),
            vec!["Alice", "Zoe"]
        );
    }
}
