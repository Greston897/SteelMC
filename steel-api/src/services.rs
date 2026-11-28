//! ServicesManager — как `org.bukkit.plugin.ServicesManager` + Vault `RegisteredServiceProvider`.
//! Плагин регистрирует `Economy`, `Permission` (LuckPerms), `Chat` провайдер, другие берут через `getRegistration`.

use std::any::{Any, TypeId};
use std::sync::Arc;
use rustc_hash::FxHashMap;
use steel_utils::locks::SyncRwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ServicePriority { Lowest = 0, Low = 1, Normal = 2, High = 3, Highest = 4 }

#[derive(Clone)]
pub struct ServiceRegistration {
    pub plugin: String,
    pub priority: ServicePriority,
    pub provider: Arc<dyn Any + Send + Sync>,
    type_id: TypeId,
}
impl ServiceRegistration {
    pub fn type_id(&self) -> TypeId { self.type_id }
}

pub struct ServicesManager {
    services: SyncRwLock<FxHashMap<TypeId, Vec<ServiceRegistration>>>,
}
impl Default for ServicesManager { fn default() -> Self { Self::new() } }
impl ServicesManager {
    pub fn new() -> Self { Self { services: SyncRwLock::new(FxHashMap::default()) } }

    pub fn register<T: Any + Send + Sync>(&self, plugin: &str, provider: Arc<T>, priority: ServicePriority) {
        let mut w = self.services.write();
        let v = w.entry(TypeId::of::<T>()).or_default();
        v.push(ServiceRegistration { plugin: plugin.to_string(), priority, provider: provider as Arc<dyn Any + Send + Sync>, type_id: TypeId::of::<T>() });
        v.sort_by_key(|r| r.priority);
    }
    pub fn unregister<T: Any>(&self, plugin: &str) {
        let mut w = self.services.write();
        if let Some(v) = w.get_mut(&TypeId::of::<T>()) { v.retain(|r| r.plugin != plugin); }
    }
    pub fn unregister_all(&self, plugin: &str) {
        let mut w = self.services.write();
        for v in w.values_mut() { v.retain(|r| r.plugin != plugin); }
    }
    pub fn get<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        let r = self.services.read();
        r.get(&TypeId::of::<T>()).and_then(|v| v.last()).and_then(|reg| reg.provider.clone().downcast::<T>().ok())
    }
    pub fn is_provided<T: Any>(&self) -> bool {
        let r = self.services.read();
        r.get(&TypeId::of::<T>()).is_some_and(|v| !v.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    trait MyService: Send + Sync + Any { fn hello(&self) -> &str; }
    struct Impl;
    impl MyService for Impl { fn hello(&self) -> &str { "hi" } }
    #[test]
    fn services_flow() {
        let m = ServicesManager::new();
        // register a concrete type, not trait object directly (Any needs concrete)
        let p: Arc<String> = Arc::new("prov".to_string());
        m.register::<String>("test", p.clone(), ServicePriority::Normal);
        assert!(m.is_provided::<String>());
        let got = m.get::<String>().unwrap();
        assert_eq!(got.as_str(), "prov");
        m.unregister::<String>("test");
        assert!(!m.is_provided::<String>());
    }
}
