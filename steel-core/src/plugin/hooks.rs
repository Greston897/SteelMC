//! Хуки поведения — прокси `steel-core` <-> `steel-api`.
//! Плагин регистрирует `BlockBehaviorHook` через `steel_api::BlockHooks`,
//! ядро делегирует сюда из `BlockBehaviorRegistry`.

#![allow(missing_docs, reason = "plugin system has extensive docs in STEEL-API.md")]
#![allow(clippy::missing_panics_doc, reason = "plugin manager panics are documented in API")]
use std::sync::{Arc, LazyLock};
use parking_lot::RwLock;
use steel_api::block::BlockBehaviorHook;
use steel_utils::Identifier;

/// Глобальный реестр плагин-хуков (один на процесс, как `BLOCK_BEHAVIORS`).
pub struct PluginHookRegistry {
    hooks: RwLock<std::collections::HashMap<Identifier, Arc<dyn BlockBehaviorHook>>>,
}

impl PluginHookRegistry {
    pub fn new() -> Self { Self { hooks: RwLock::new(Default::default()) } }

    pub fn register(&self, id: Identifier, hook: Arc<dyn BlockBehaviorHook>) -> Option<Arc<dyn BlockBehaviorHook>> {
        self.hooks.write().insert(id, hook)
    }
    pub fn get(&self, id: &Identifier) -> Option<Arc<dyn BlockBehaviorHook>> {
        self.hooks.read().get(id).cloned()
    }
    pub fn remove(&self, id: &Identifier) -> bool {
        self.hooks.write().remove(id).is_some()
    }
    pub fn len(&self) -> usize { self.hooks.read().len() }
    pub fn clear(&self) { self.hooks.write().clear(); }
    pub fn is_empty(&self) -> bool { self.hooks.read().is_empty() }
}

impl Default for PluginHookRegistry {
    fn default() -> Self { Self::new() }
}

/// Глобальный инстанс — `steel_api` хуки живут тут, доступны из `BlockBehavior`.
pub static PLUGIN_HOOKS: LazyLock<PluginHookRegistry> = LazyLock::new(PluginHookRegistry::new);

/// Хелпер для `BlockBehavior` делегирования: если есть плагин-хук — вызвай его, иначе дефолт.
pub fn with_hook<T>(id: &Identifier, f: impl FnOnce(&Arc<dyn BlockBehaviorHook>) -> T, default: T) -> T {
    if let Some(hook) = PLUGIN_HOOKS.get(id) {
        f(&hook)
    } else {
        default
    }
}
