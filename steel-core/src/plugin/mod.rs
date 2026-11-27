//! Плагин-система ядра — мост между `steel-api` и реальным миром.
//! Каждая механика ядра доступна плагину: блоки, предметы, жидкости, сущности,
//! чанки, редстоун, физика, порталы, лут, мирген, команды, пакеты, скорборд.
//!
//! Архитектура:
//! - `PluginManager` — загрузка/включение/выключение, зависимости, `EventBus`/`Scheduler`/`ViaPipeline`
//! - `hooks` — прокси для `steel-core::behavior::BlockBehavior` → `steel_api::BlockBehaviorHook`
//! - `via` — интеграция ViaVersion (трансляция входящих/исходящих пакетов)
//! - `loader` — compile-time реестр (`inventory`/`linkme`) + будущий `dlopen` для `.so` плагинов
#![allow(missing_docs, reason = "plugin system has extensive docs in STEEL-API.md")]
#![allow(clippy::missing_panics_doc, reason = "plugin manager panics are documented in API")]
use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use steel_api::plugin::{Plugin, PluginContext, PluginDescriptor, PluginResult, PluginState};
use steel_api::{EventBus, Scheduler, ViaPipeline, protocol::ProtocolVersion};
use steel_api::command::CommandRegistryHandle;
use steel_api::permission::PermissionRegistryHandle;
use steel_api::registry::RegistryAccess;

pub mod hooks;
pub mod loader;
pub mod via;

/// Менеджер плагинов — владеет шиной, планировщиком, Via.
pub struct PluginManager {
    buses: Arc<EventBus>,
    scheduler: Arc<Scheduler>,
    via: Arc<ViaPipeline>,
    commands: Arc<CommandRegistryHandle>,
    permissions: Arc<PermissionRegistryHandle>,
    registry: Arc<dyn RegistryAccess>,
    plugins: RwLock<HashMap<String, PluginEntry>>,
}

struct PluginEntry {
    plugin: Arc<dyn Plugin>,
    descriptor: PluginDescriptor,
    state: PluginState,
    context: Arc<PluginContext>,
}

impl PluginManager {
    pub fn new(registry: Arc<dyn RegistryAccess>) -> Self {
        let buses = Arc::new(EventBus::new());
        let scheduler = Arc::new(Scheduler::new());
        let via = Arc::new(ViaPipeline::new());
        let commands = Arc::new(CommandRegistryHandle::new());
        let permissions = Arc::new(PermissionRegistryHandle::new());
        Self {
            buses,
            scheduler,
            via,
            commands,
            permissions,
            registry,
            plugins: RwLock::new(HashMap::new()),
        }
    }

    pub fn event_bus(&self) -> &Arc<EventBus> { &self.buses }
    pub fn scheduler(&self) -> &Arc<Scheduler> { &self.scheduler }
    pub fn via(&self) -> &Arc<ViaPipeline> { &self.via }
    pub fn commands(&self) -> &Arc<CommandRegistryHandle> { &self.commands }
    pub fn permissions(&self) -> &Arc<PermissionRegistryHandle> { &self.permissions }

    /// Зарегистрировать плагин (compile-time). Вызывает `on_load`.
    pub fn register(&self, plugin: Arc<dyn Plugin>) -> PluginResult<()> {
        let desc = plugin.descriptor();
        desc.validate()?;
        let name = desc.name.clone();
        if self.plugins.read().contains_key(&name) {
            return Err(steel_api::plugin::PluginError::AlreadyLoaded(name));
        }
        let ctx = Arc::new(PluginContext::new(
            desc.clone(),
            format!("plugins/{name}").into(),
            self.buses.clone(),
            self.scheduler.clone(),
            self.commands.clone(),
            self.permissions.clone(),
            self.registry.clone(),
        ));
        plugin.on_load(&ctx)?;
        let entry = PluginEntry { plugin: plugin.clone(), descriptor: desc, state: PluginState::Loaded, context: ctx };
        self.plugins.write().insert(name, entry);
        Ok(())
    }

    /// Включить все `Loaded` плагины (после загрузки миров).
    pub fn enable_all(&self) -> Vec<(String, PluginResult<()>)> {
        let names: Vec<String> = self.plugins.read().keys().cloned().collect();
        let mut out = Vec::new();
        for name in names {
            let res = self.enable(&name);
            out.push((name, res));
        }
        out
    }

    pub fn enable(&self, name: &str) -> PluginResult<()> {
        let mut w = self.plugins.write();
        let e = w.get_mut(name).ok_or_else(|| steel_api::plugin::PluginError::Io(format!("plugin {name} not found")))?;
        if e.state == PluginState::Enabled { return Ok(()); }
        let r = e.plugin.on_enable(&e.context);
        match r {
            Ok(()) => { e.state = PluginState::Enabled; Ok(()) },
            Err(err) => { e.state = PluginState::Failed; Err(err) },
        }
    }

    pub fn disable(&self, name: &str) -> bool {
        let mut w = self.plugins.write();
        if let Some(e) = w.get_mut(name) {
            e.plugin.on_disable(&e.context);
            e.state = PluginState::Disabled;
            self.scheduler.cancel_all_for(name);
            true
        } else { false }
    }

    pub fn disable_all(&self) {
        let names: Vec<String> = self.plugins.read().keys().cloned().collect();
        for n in names { self.disable(&n); }
    }

    pub fn list(&self) -> Vec<(String, PluginState)> {
        self.plugins.read().iter().map(|(k,v)| (k.clone(), v.state)).collect()
    }

    pub fn is_via_supported(&self, client_proto: i32, server_proto: i32) -> bool {
        self.via.is_supported(ProtocolVersion::new(client_proto), ProtocolVersion::new(server_proto))
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        struct DummyReg;
        impl RegistryAccess for DummyReg {
            fn block_by_id(&self, _: &steel_utils::Identifier) -> Option<steel_registry::blocks::BlockRef> { None }
            fn item_by_id(&self, _: &steel_utils::Identifier) -> Option<steel_registry::items::ItemRef> { None }
            fn all_block_ids(&self) -> Vec<steel_utils::Identifier> { Vec::new() }
            fn all_item_ids(&self) -> Vec<steel_utils::Identifier> { Vec::new() }
        }
        Self::new(Arc::new(DummyReg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use steel_api::plugin::{Plugin, PluginContext, PluginResult};

    struct DummyPlugin;
    impl Plugin for DummyPlugin {
        fn name(&self) -> &str { "dummy" }
        fn version(&self) -> &str { "1.0.0" }
        fn on_enable(&self, ctx: &PluginContext) -> PluginResult<()> {
            ctx.logger().info("dummy enabled");
            Ok(())
        }
    }

    #[test]
    fn register_enable_disable() {
        let mgr = PluginManager::default();
        mgr.register(Arc::new(DummyPlugin)).unwrap();
        assert_eq!(mgr.list().len(), 1);
        mgr.enable_all();
        assert!(mgr.list().iter().any(|(_,s)| *s == PluginState::Enabled));
        mgr.disable("dummy");
        assert!(mgr.list().iter().any(|(_,s)| *s == PluginState::Disabled));
    }

    #[test]
    fn via_pipeline_works() {
        let mgr = PluginManager::default();
        // 777 <-> 777 пасс
        assert!(mgr.is_via_supported(777, 777));
        // неизвестный — нет пути
        assert!(!mgr.is_via_supported(999, 777));
    }
}
