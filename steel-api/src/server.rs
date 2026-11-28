//! Сервер-API — аналог `org.bukkit.Server` + `Bukkit` синглтон, но без глобалок.
//! Через `PluginContext::server()` плагин получает полный доступ к серверу.
//! Всё что есть в Paper/Leaf — здесь есть, плюс нативно Rust.

use std::sync::Arc;
use std::time::Duration;

use crate::player::Player;
use crate::world::World;

/// Статистика TPS/MSPT как в Paper `Server#getTPS()` / Spark.
#[derive(Debug, Clone, Copy)]
pub struct TpsInfo {
    pub tps_1m: f64,
    pub tps_5m: f64,
    pub tps_15m: f64,
    pub mspt: f64,
    pub tick_duration_ns: u64,
}

/// Информация о сервере — аналог `Server` + `Bukkit`.
pub trait SteelServer: Send + Sync + 'static {
    // — идентификация —
    fn name(&self) -> &str { "SteelMC" }
    fn version(&self) -> &str { crate::API_VERSION }
    fn minecraft_version(&self) -> &str { "26.3" }
    fn protocol_version(&self) -> i32 { 777 }

    // — игроки —
    fn online_players(&self) -> Vec<Arc<dyn Player>>;
    fn get_player(&self, id: uuid::Uuid) -> Option<Arc<dyn Player>>;
    fn get_player_by_name(&self, name: &str) -> Option<Arc<dyn Player>>;
    fn offline_player(&self, id: uuid::Uuid) -> Option<Arc<dyn Player>>;
    fn max_players(&self) -> usize { 100 }
    fn set_max_players(&self, _n: usize) {}
    fn online_count(&self) -> usize { self.online_players().len() }

    // — миры —
    fn worlds(&self) -> Vec<Arc<dyn World>>;
    fn get_world(&self, name: &str) -> Option<Arc<dyn World>>;
    fn default_world(&self) -> Option<Arc<dyn World>> { self.get_world("world") }
    fn create_world(&self, _name: &str, _seed: Option<i64>) -> Option<Arc<dyn World>> { None }
    fn unload_world(&self, _name: &str, _save: bool) -> bool { false }

    // — широковещание —
    fn broadcast(&self, msg: &str);
    fn broadcast_component(&self, component: &crate::chat::Component) { self.broadcast(&component.to_legacy()); }
    fn shutdown(&self);
    fn reload(&self) {}
    fn is_primary_thread(&self) -> bool { true }

    // — TPS / производительность —
    fn tps(&self) -> TpsInfo { TpsInfo { tps_1m: 20.0, tps_5m: 20.0, tps_15m: 20.0, mspt: 5.0, tick_duration_ns: 5_000_000 } }
    fn mspt(&self) -> f64 { self.tps().mspt }
    fn uptime(&self) -> Duration { Duration::from_secs(0) }

    // — белые/чёрные списки, вайтлист —
    fn is_whitelisted(&self, _id: uuid::Uuid) -> bool { false }
    fn set_whitelisted(&self, _id: uuid::Uuid, _on: bool) {}
    fn is_banned(&self, _id: uuid::Uuid) -> bool { false }
    fn ban(&self, _id: uuid::Uuid, _reason: Option<&str>, _expires: Option<Duration>) {}
    fn pardon(&self, _id: uuid::Uuid) {}

    // — команды / плагины (как `Bukkit.getPluginManager()`) —
    fn dispatch_command(&self, _sender: &crate::command::CommandSource, _cmd: &str) -> bool { false }

    // — мотд / иконка / пинг —
    fn motd(&self) -> String { "Steel powered by SteelAPI".into() }
    fn set_motd(&self, motd: &str) {}
    fn server_icon(&self) -> Option<Vec<u8>> { None }
}

/// Хэндл сервера — ядро инжектит в `PluginContext`.
pub struct ServerHandle {
    inner: Arc<dyn SteelServer>,
}

impl ServerHandle {
    pub fn new(inner: Arc<dyn SteelServer>) -> Self { Self { inner } }
    pub fn inner(&self) -> &Arc<dyn SteelServer> { &self.inner }
    pub fn broadcast(&self, msg: &str) { self.inner.broadcast(msg); }
    pub fn shutdown(&self) { self.inner.shutdown(); }
}

/// Заглушка для тестов — как `VanillaRegistryAccess`.
pub struct DummyServer;

impl SteelServer for DummyServer {
    fn online_players(&self) -> Vec<Arc<dyn Player>> { Vec::new() }
    fn get_player(&self, _id: uuid::Uuid) -> Option<Arc<dyn Player>> { None }
    fn get_player_by_name(&self, _name: &str) -> Option<Arc<dyn Player>> { None }
    fn offline_player(&self, _id: uuid::Uuid) -> Option<Arc<dyn Player>> { None }
    fn worlds(&self) -> Vec<Arc<dyn World>> { Vec::new() }
    fn get_world(&self, _name: &str) -> Option<Arc<dyn World>> { None }
    fn broadcast(&self, msg: &str) { log::info!("[broadcast] {}", msg); }
    fn shutdown(&self) { log::info!("shutdown requested"); }
}
