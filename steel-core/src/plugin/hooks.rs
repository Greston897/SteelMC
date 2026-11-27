//! Хуки поведения — прокси `steel-core` <-> `steel-api`.
//! Плагин регистрирует `BlockBehaviorHook` через `steel_api::BlockHooks`,
//! ядро делегирует сюда из `BlockBehaviorRegistry`.

#![allow(missing_docs, reason = "plugin system has extensive docs in STEEL-API.md")]
#![allow(clippy::missing_panics_doc, reason = "plugin manager panics are documented in API")]
#![allow(clippy::pedantic, reason = "hooks registry is simple")]
#![allow(clippy::doc_markdown, reason = "hook docs contain Minecraft terms")]
#![allow(clippy::must_use_candidate, reason = "plugin helpers not always must_use")]
#![allow(clippy::missing_const_for_fn, reason = "not needed")]
#![allow(clippy::default_trait_access, reason = "HashMap::default is clear")]
#![allow(clippy::absolute_paths, reason = "plugin uses absolute paths")]
#![allow(clippy::disallowed_types, reason = "now uses FxHashMap/SyncRwLock, allow remaining")]

use std::sync::{Arc, LazyLock};

use rustc_hash::FxHashMap;
use steel_utils::locks::SyncRwLock;
use steel_api::block::BlockBehaviorHook;
use steel_utils::Identifier;

/// Глобальный реестр плагин-хуков (один на процесс, как `BLOCK_BEHAVIORS`).
pub struct PluginHookRegistry {
    hooks: SyncRwLock<FxHashMap<Identifier, Arc<dyn BlockBehaviorHook>>>,
}

impl PluginHookRegistry {
    pub fn new() -> Self { Self { hooks: SyncRwLock::new(FxHashMap::default()) } }

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
