//! BanList / Whitelist — как `org.bukkit.BanList` + Paper `BanList`.
//! `Bukkit.getBanList(BanList.Type.NAME/IP).addBan(...)` + `isBanned`, `pardon`.
//! Также `Whitelist` (`white-list` в `server.properties`) — `Player#isWhitelisted`.

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use rustc_hash::{FxHashMap, FxHashSet};
use steel_utils::locks::SyncRwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BanType { Name, Ip }

#[derive(Debug, Clone)]
pub struct BanEntry {
    pub target: String, // name или ip
    pub reason: Option<String>,
    pub source: Option<String>, // кто забанил
    pub created: u64, // epoch secs
    pub expiration: Option<u64>, // None = forever
}
impl BanEntry {
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = self.expiration {
            now_secs() >= exp
        } else { false }
    }
    pub fn remaining(&self) -> Option<Duration> {
        self.expiration.map(|exp| {
            let now = now_secs();
            if exp <= now { Duration::from_secs(0) } else { Duration::from_secs(exp - now) }
        })
    }
}
fn now_secs() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() }

pub struct BanList {
    ty: BanType,
    entries: SyncRwLock<FxHashMap<String, BanEntry>>,
}
impl BanList {
    pub fn new(ty: BanType) -> Self { Self { ty, entries: SyncRwLock::new(FxHashMap::default()) } }
    pub fn ty(&self) -> BanType { self.ty }
    pub fn add(&self, target: &str, reason: Option<&str>, source: Option<&str>, expires: Option<Duration>) -> BanEntry {
        let exp = expires.map(|d| now_secs() + d.as_secs());
        let e = BanEntry { target: target.to_string(), reason: reason.map(|s| s.to_string()), source: source.map(|s| s.to_string()), created: now_secs(), expiration: exp };
        self.entries.write().insert(target.to_string(), e.clone());
        e
    }
    pub fn is_banned(&self, target: &str) -> bool {
        let r = self.entries.read();
        if let Some(e) = r.get(target) { !e.is_expired() } else { false }
    }
    pub fn get(&self, target: &str) -> Option<BanEntry> {
        let r = self.entries.read();
        r.get(target).cloned().filter(|e| !e.is_expired())
    }
    pub fn pardon(&self, target: &str) -> bool { self.entries.write().remove(target).is_some() }
    pub fn all(&self) -> Vec<BanEntry> { self.entries.read().values().filter(|e| !e.is_expired()).cloned().collect() }
    pub fn purge_expired(&self) -> usize {
        let mut w = self.entries.write();
        let before = w.len();
        w.retain(|_, e| !e.is_expired());
        before - w.len()
    }
}

pub struct BanManager {
    name: BanList,
    ip: BanList,
    whitelist: SyncRwLock<FxHashSet<String>>, // lowercased names
    whitelist_enabled: std::sync::atomic::AtomicBool,
}
impl Default for BanManager { fn default() -> Self { Self::new() } }
impl BanManager {
    pub fn new() -> Self {
        Self { name: BanList::new(BanType::Name), ip: BanList::new(BanType::Ip), whitelist: SyncRwLock::new(FxHashSet::default()), whitelist_enabled: std::sync::atomic::AtomicBool::new(false) }
    }
    pub fn ban_list(&self, ty: BanType) -> &BanList { match ty { BanType::Name => &self.name, BanType::Ip => &self.ip } }
    pub fn is_whitelisted(&self, name: &str) -> bool {
        if !self.whitelist_enabled.load(std::sync::atomic::Ordering::Relaxed) { return true; }
        self.whitelist.read().contains(&name.to_ascii_lowercase())
    }
    pub fn set_whitelisted(&self, name: &str, on: bool) {
        let mut w = self.whitelist.write();
        if on { w.insert(name.to_ascii_lowercase()); } else { w.remove(&name.to_ascii_lowercase()); }
    }
    pub fn set_enabled(&self, v: bool) { self.whitelist_enabled.store(v, std::sync::atomic::Ordering::Relaxed); }
    pub fn is_enabled(&self) -> bool { self.whitelist_enabled.load(std::sync::atomic::Ordering::Relaxed) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ban_flow() {
        let b = BanList::new(BanType::Name);
        b.add("Notch", Some("grief"), Some("Admin"), None);
        assert!(b.is_banned("Notch"));
        assert!(!b.is_banned("Herobrine"));
        assert!(b.pardon("Notch"));
        assert!(!b.is_banned("Notch"));
    }
    #[test]
    fn whitelist() {
        let m = BanManager::new();
        m.set_enabled(true);
        m.set_whitelisted("Alex", true);
        assert!(m.is_whitelisted("Alex"));
        assert!(!m.is_whitelisted("Steve"));
        m.set_enabled(false);
        assert!(m.is_whitelisted("Steve"));
    }
}
