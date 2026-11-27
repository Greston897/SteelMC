//! Права — строковые ноды как в Bukkit/LuckPerms (`steel.build`, `myplugin.admin`), c дефолтами.

use rustc_hash::FxHashMap;
use steel_utils::locks::SyncRwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionDefault {
    True,
    False,
    Op,
    NotOp,
}

impl PermissionDefault {
    pub fn is_granted(&self, is_op: bool) -> bool {
        match self { Self::True => true, Self::False => false, Self::Op => is_op, Self::NotOp => !is_op }
    }
}

#[derive(Debug, Clone)]
pub struct Permission {
    pub name: String,
    pub description: String,
    pub default: PermissionDefault,
    pub children: FxHashMap<String, bool>,
}

impl Permission {
    pub fn new(name: impl Into<String>, default: PermissionDefault) -> Self {
        Self { name: name.into(), description: String::new(), default, children: FxHashMap::default() }
    }
    pub fn with_description(mut self, d: impl Into<String>) -> Self {
        self.description = d.into(); self
    }
}

/// Реестр прав — инжектится в `PluginContext`.
pub struct PermissionRegistryHandle {
    perms: SyncRwLock<FxHashMap<String, Permission>>,
}

impl Default for PermissionRegistryHandle {
    fn default() -> Self { Self::new() }
}
impl PermissionRegistryHandle {
    pub fn new() -> Self { Self { perms: SyncRwLock::new(FxHashMap::default()) } }

    pub fn register(&self, perm: Permission) -> bool {
        let mut w = self.perms.write();
        if w.contains_key(&perm.name) { return false; }
        w.insert(perm.name.clone(), perm);
        true
    }
    pub fn unregister(&self, name: &str) -> bool {
        self.perms.write().remove(name).is_some()
    }
    pub fn has(&self, name: &str) -> bool { self.perms.read().contains_key(name) }
    pub fn get(&self, name: &str) -> Option<Permission> { self.perms.read().get(name).cloned() }
    pub fn all(&self) -> Vec<Permission> { self.perms.read().values().cloned().collect() }
    pub fn len(&self) -> usize { self.perms.read().len() }
    pub fn is_empty(&self) -> bool { self.perms.read().is_empty() }
}

/// Хелпер для `Player::has_permission` — матчит wildcard `myplugin.*`.
pub fn permission_matches(pattern: &str, permission: &str) -> bool {
    if pattern == permission { return true; }
    if let Some(prefix) = pattern.strip_suffix(".*") {
        return permission == prefix || permission.starts_with(&format!("{prefix}."));
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wildcard() {
        assert!(permission_matches("myplugin.*", "myplugin.admin"));
        assert!(permission_matches("myplugin.*", "myplugin.admin.ban"));
        assert!(!permission_matches("myplugin.*", "other.admin"));
        assert!(permission_matches("steel.build", "steel.build"));
    }
}
