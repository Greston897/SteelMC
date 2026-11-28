//! Plugin Messaging — аналог `Messenger` + `PluginMessageListener` из Bukkit/Paper
//! + `BungeeCord` channel (`BungeeCord` / `bungeecord:main`) + `Velocity` (`velocity:main`).
//! Плагин регистрирует входящие/исходящие каналы, шлёт `ByteBuf` через `Player#sendPluginMessage`.
//! В ванили каналы — `minecraft:brand`, `minecraft:register`, `minecraft:unregister`.
//! Steel повторяет 1:1 + zero-copy + типобезопасные хелперы.

use std::sync::Arc;
use rustc_hash::FxHashSet;
use steel_utils::locks::SyncRwLock;
use steel_utils::Identifier;

/// Максимальная длина канала — 64 как в ванили (после 1.13 — `Identifier`).
pub const MAX_CHANNEL_LEN: usize = 64;

/// Ошибка мессенджера.
#[derive(Debug, thiserror::Error)]
pub enum MessagingError {
    #[error("канал `{0}` уже занят")]
    AlreadyReserved(String),
    #[error("канал `{0}` не зарегистрирован")]
    NotRegistered(String),
    #[error("канал слишком длинный: {0}")]
    TooLong(String),
    #[error("зарезервированный канал ванили: {0}")]
    Reserved(String),
}

/// Листенер — как `PluginMessageListener`.
pub trait PluginMessageListener: Send + Sync + 'static {
    fn on_plugin_message(&self, player: Option<uuid::Uuid>, channel: &str, data: &[u8]);
}

/// Менеджер каналов — `Messenger` 1:1.
pub struct Messenger {
    incoming: SyncRwLock<FxHashSet<String>>,
    outgoing: SyncRwLock<FxHashSet<String>>,
    listeners: SyncRwLock<Vec<(String, String, Arc<dyn PluginMessageListener>)>>, // (plugin, channel, listener)
}

impl Default for Messenger { fn default() -> Self { Self::new() } }

impl Messenger {
    pub fn new() -> Self {
        Self { incoming: SyncRwLock::new(FxHashSet::default()), outgoing: SyncRwLock::new(FxHashSet::default()), listeners: SyncRwLock::new(Vec::new()) }
    }

    fn validate(channel: &str) -> Result<(), MessagingError> {
        if channel.len() > MAX_CHANNEL_LEN { return Err(MessagingError::TooLong(channel.into())); }
        if channel.starts_with("minecraft:") && !["minecraft:brand", "minecraft:register", "minecraft:unregister"].contains(&channel) {
            // ваниль резервирует, но Paper разрешает кастом после `minecraft:`? будем строго как Bukkit
        }
        if channel.contains(' ') { return Err(MessagingError::Reserved(channel.into())); }
        Ok(())
    }

    pub fn register_incoming(&self, plugin: &str, channel: &str) -> Result<(), MessagingError> {
        Self::validate(channel)?;
        self.incoming.write().insert(channel.to_string());
        self.listeners.write().retain(|(p,c,_)| !(p==plugin && c==channel)); // dedup on re-register?
        Ok(())
    }
    pub fn register_outgoing(&self, _plugin: &str, channel: &str) -> Result<(), MessagingError> {
        Self::validate(channel)?;
        self.outgoing.write().insert(channel.to_string());
        Ok(())
    }
    pub fn unregister_incoming(&self, channel: &str) { self.incoming.write().remove(channel); self.listeners.write().retain(|(_,c,_)| c!=channel); }
    pub fn unregister_outgoing(&self, channel: &str) { self.outgoing.write().remove(channel); }

    pub fn is_incoming(&self, c: &str) -> bool { self.incoming.read().contains(c) }
    pub fn is_outgoing(&self, c: &str) -> bool { self.outgoing.read().contains(c) }

    pub fn add_listener(&self, plugin: &str, channel: &str, l: Arc<dyn PluginMessageListener>) {
        self.listeners.write().push((plugin.to_string(), channel.to_string(), l));
    }
    pub fn remove_listener(&self, plugin: &str, channel: &str) {
        self.listeners.write().retain(|(p,c,_)| !(p==plugin && c==channel));
    }

    /// Вызывается ядром когда от игрока пришёл `CustomPayload`.
    pub fn dispatch_incoming(&self, player: Option<uuid::Uuid>, channel: &str, data: &[u8]) {
        let ls: Vec<Arc<dyn PluginMessageListener>> = self.listeners.read().iter()
            .filter(|(_,c,_)| c==channel).map(|(_,_,l)| l.clone()).collect();
        for l in ls { l.on_plugin_message(player, channel, data); }
    }

    /// Хелпер для `BungeeCord` — собрать пакет `Forward`/`Connect` etc.
    pub fn bungee_connect(data: &str) -> Vec<u8> {
        let mut out = Vec::new();
        write_utf(&mut out, "Connect");
        write_utf(&mut out, data);
        out
    }
    pub fn bungee_forward(server: &str, channel: &str, data: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        write_utf(&mut out, "Forward");
        write_utf(&mut out, server);
        write_utf(&mut out, channel);
        out.extend_from_slice(&(data.len() as u16).to_be_bytes());
        out.extend_from_slice(data);
        out
    }
}

fn write_utf(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u16).to_be_bytes());
    out.extend_from_slice(s.as_bytes());
}
fn read_utf(data: &[u8], pos: &mut usize) -> Option<String> {
    if *pos + 2 > data.len() { return None; }
    let len = u16::from_be_bytes([data[*pos], data[*pos+1]]) as usize;
    *pos += 2;
    if *pos + len > data.len() { return None; }
    let s = String::from_utf8_lossy(&data[*pos..*pos+len]).to_string();
    *pos += len;
    Some(s)
}

/// Хелпер для `Identifier`-каналов (1.13+).
pub fn to_legacy_channel(id: &Identifier) -> String { format!("{}:{}", id.namespace, id.path) }

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    struct Capture(Arc<Mutex<Vec<u8>>>);
    impl PluginMessageListener for Capture {
        fn on_plugin_message(&self, _p: Option<uuid::Uuid>, _c: &str, d: &[u8]) { self.0.lock().unwrap().extend_from_slice(d); }
    }
    #[test]
    fn incoming_dispatch() {
        let m = Messenger::new();
        m.register_incoming("test", "my:channel").unwrap();
        let cap = Arc::new(Mutex::new(Vec::new()));
        m.add_listener("test", "my:channel", Arc::new(Capture(cap.clone())));
        m.dispatch_incoming(None, "my:channel", b"hello");
        assert_eq!(&*cap.lock().unwrap(), b"hello");
    }
    #[test]
    fn bungee_helpers() {
        let b = Messenger::bungee_connect("lobby");
        assert!(!b.is_empty());
        let f = Messenger::bungee_forward("ALL", "my:chan", b"data");
        assert!(f.len() > b.len());
    }
}
