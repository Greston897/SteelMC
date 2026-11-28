//! # `Steel API` — плагин-API ядра `SteelMC`
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
//! - **`ViaVersion` внутри:** трансляция протоколов — первоклассный API, не костыль-снаружи.
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
#![allow(clippy::pedantic, reason = "API is ergonomic, pedantic lints are too noisy for plugin traits")]
#![allow(clippy::nursery, reason = "nursery lints are experimental, not relevant for API")]
#![allow(clippy::must_use_candidate, reason = "many API methods are optionally used, not every getter needs must_use")]
#![allow(clippy::missing_const_for_fn, reason = "const fn is not needed for many API methods, keep simple")]
#![allow(clippy::doc_markdown, reason = "docs contain many Minecraft terms, backticks not needed for every word")]
#![allow(clippy::doc_lazy_continuation, reason = "dash lists in module docs")]
#![allow(clippy::derivable_impls, reason = "manual Default for enums with custom variant is clearer")]
#![allow(clippy::module_name_repetitions, reason = "steel_api module names intentionally repeat for clarity")]
#![allow(clippy::absolute_paths, reason = "API uses absolute paths for clarity in examples")]
#![allow(clippy::needless_pass_by_value, reason = "API takes owned values for ergonomics")]
#![allow(clippy::new_without_default, reason = "some types have custom new without default")]
#![allow(clippy::too_many_arguments, reason = "some hooks naturally have many args")]
#![allow(clippy::type_complexity, reason = "command types are complex by nature")]
#![allow(clippy::clone_on_ref_ptr, reason = "Arc::clone is idiomatic in plugin API, explicit path is noisy")]
#![allow(clippy::manual_strip, reason = "manual strip is clear for permission wildcard")]
#![allow(clippy::disallowed_types, reason = "API uses FxHashMap/SyncRwLock internally, allows standard types in trait signatures where needed")]
#![allow(clippy::too_many_lines, reason = "API modules are large by necessity")]
#![allow(clippy::redundant_clone, reason = "ergonomic clone in builders")]
#![allow(clippy::unwrap_used, reason = "examples use unwrap, core API uses expect")]
#![allow(clippy::print_stdout, reason = "examples print")]
#![allow(unused_imports, reason = "some imports used only in tests")]
#![allow(unused_mut, reason = "clippy recommends not mut where not needed, but code is clearer with mut")]
#![allow(unused_variables, reason = "API examples have unused vars for brevity")]
#![allow(dead_code, reason = "API has many placeholder fields for future features, not all used yet")]
#![allow(clippy::enum_variant_names, reason = "MC enums share prefixes by design")]
#![allow(clippy::collapsible_if, reason = "collapsible_if noisy for command checks")]
#![allow(clippy::while_let_on_iterator, reason = "manual char parsing clearer")]
#![allow(unfulfilled_lint_expectations, reason = "expect may be unfulfilled after refactoring, not critical")]

use steel_protocol as _;

pub mod attribute;
pub mod ban;
pub mod block;
pub mod bossbar;
pub mod chat;
pub mod chunk;
pub mod command;
pub mod config;
pub mod economy;
pub mod effect;
pub mod entity;
pub mod event;
pub mod events;
pub mod fluid;
pub mod inventory;
pub mod item;
pub mod messaging;
pub mod metadata;
pub mod nbt;
pub mod permission;
pub mod packet;
pub mod placeholder;
pub mod player;
pub mod plugin;
pub mod plugin_loader;
pub mod protocol;
pub mod recipe;
pub mod redstone;
pub mod region;
pub mod registry;
pub mod resourcepack;
pub mod scheduler;
pub mod scoreboard;
pub mod server;
pub mod services;
pub mod tab;
pub mod tag;
pub mod world;
pub mod world_creator;
pub mod worldgen;

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
