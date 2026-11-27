//! Загрузчик плагинов — пока compile-time (статическая линковка), в будущем `dlopen`.
//! Паттерн как у `bevy`/`paper`: плагины регистрируются через `inventory` или явный `register`.

#![allow(missing_docs, clippy::missing_panics_doc)]
use std::sync::Arc;
use steel_api::plugin::Plugin;
use super::PluginManager;

/// Трейт для compile-time плагина (авторегистрация).
/// Реализуй `fn create() -> Arc<dyn Plugin>` и пометь `#[linkme]` если используешь `linkme`.
pub trait CompiledPlugin {
    fn create() -> Arc<dyn Plugin> where Self: Sized;
}

/// Явная регистрация списка compiled-плагинов (простейший путь без `linkme`).
pub fn register_all(mgr: &PluginManager, plugins: Vec<Arc<dyn Plugin>>) {
    for p in plugins {
        let name = p.name().to_string();
        if let Err(e) = mgr.register(p) {
            log::warn!("failed to register plugin {name}: {e}");
        }
    }
}

/// Пример: `register_all(mgr, vec![Arc::new(MyPlugin::default())])` в `Server::new`.
