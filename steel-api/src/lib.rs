//! # `Steel API` — плагин-API ядра SteelMC
//!
//! Своё, понятное, на уровне Bukkit/Paper, но лучше: Rust-идиоматично, типобезопасно,
//! покрывает **каждую механику** ядра. Любой плагин может переопределить любую механику.
//!
//! ## Философия (почему своё, а не Paper-форк)
//! - **Bukkit-совместимость по идеям, не по багам:** знакомые `Event`, `Listener`, `Scheduler`,
//!   `Command`, `Block`/`World`/`Player`/`Entity`, но без `Bukkit.getServer()` монстра.
//! - **Rust:** `Result` вместо исключений, `trait` вместо наследования, `Send + Sync`.
//! - **Каждая механика — хук:** block/item/fluid behavior, entity AI, worldgen, loot, redstone,
//!   физика, порталы, скорборд, инвентари, пакеты. Если ваниль это делает — ты можешь это поменять.
//! - **ViaVersion внутри:** трансляция протоколов — первоклассный API, не костыль-снаружи.
//!
//! ## Быстрый старт
//! ```rust,ignore
//! use steel_api::prelude::*;
//!
//! pub struct MyPlugin;
//! impl Plugin for MyPlugin {
//!     fn name(&self) -> &str { "my_plugin" }
//!     fn version(&self) -> &str { "1.0.0" }
//!     fn on_enable(&self, ctx: &PluginContext) -> PluginResult<()> {
//!         ctx.events().subscribe_simple(PlayerJoinEvent { player_id: uuid::Uuid::nil(), player_name: String::new(), join_message: None }, EventPriority::Normal, |_| {});
//!         Ok(())
//!     }
//! }
//! ```
#![allow(missing_docs, reason = "API has extensive docs in README, not every item needs rustdoc")]
#![allow(clippy::missing_panics_doc, reason = "many API examples panic for brevity, documented in tests")]

pub mod block;
pub mod command;
pub mod entity;
pub mod event;
pub mod events;
pub mod inventory;
pub mod item;
pub mod permission;
pub mod player;
pub mod plugin;
pub mod protocol;
pub mod registry;
pub mod scheduler;
pub mod world;

pub mod prelude;

pub use plugin::{Plugin, PluginContext, PluginDescriptor, PluginError, PluginResult};
pub use event::{Cancellable, Event, EventBus, EventPriority, ListenerHandle};
pub use protocol::{ProtocolVersion, ViaPipeline};
pub use scheduler::{Scheduler, TaskHandle, TaskId};

/// Версия API (синхронизирована с `workspace.package.version`).
pub const API_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Короткое имя ядра для логов.
pub const CORE_NAME: &str = "SteelMC";

/// Макрос для статической регистрации плагина (compile-time, без `dlopen`).
/// Используй `inventory` / `linkme` если нужен динамический лоадер.
#[macro_export]
macro_rules! export_plugin {
    ($ty:ty) => {
        // no-op для compile-time плагинов — просто проверка типа
        const _: fn() = || {
            fn assert_plugin<T: $crate::plugin::Plugin>() {}
            assert_plugin::<$ty>();
        };
    };
}
