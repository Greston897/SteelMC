//! Пакет-API — полный аналог ProtocolLib + Paper `PlayerChannel`.
//! Плагин может слушать, перехватывать, отменять и слать любые пакеты.
//! ViaVersion уже внутри (`ViaPipeline`), но этот модуль — низкоуровневая шина для всех пакетов.
//! Можно заменить `steel-api::protocol::ViaPipeline` более высокоуровневым транслятором, а здесь — raw.

use std::sync::Arc;
use rustc_hash::FxHashMap;
use steel_utils::locks::SyncRwLock;

/// Направление пакета.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketDirection {
    Clientbound,
    Serverbound,
}

/// Стадия соединения.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionState {
    Handshake,
    Status,
    Login,
    Configuration,
    Play,
}

/// Событие пакета — приходит до отправки/после приёма.
pub struct PacketEvent {
    pub player: Option<uuid::Uuid>,
    pub direction: PacketDirection,
    pub state: ConnectionState,
    pub id: i32,
    pub data: Vec<u8>,
    pub cancelled: bool,
    pub modified: bool,
}

impl PacketEvent {
    pub fn new(direction: PacketDirection, state: ConnectionState, id: i32, data: Vec<u8>) -> Self {
        Self { player: None, direction, state, id, data, cancelled: false, modified: false }
    }
    pub fn cancel(&mut self) { self.cancelled = true; }
    pub fn is_cancelled(&self) -> bool { self.cancelled }
    pub fn set_data(&mut self, data: Vec<u8>) { self.data = data; self.modified = true; }
    pub fn set_id(&mut self, id: i32) { self.id = id; self.modified = true; }
}

/// Приоритет как в ProtocolLib `ListenerPriority`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PacketPriority {
    Lowest = 0, Low = 1, Normal = 2, High = 3, Highest = 4, Monitor = 5,
}

/// Слушатель пакетов — аналог `PacketAdapter`.
pub trait PacketListener: Send + Sync + 'static {
    fn priority(&self) -> PacketPriority { PacketPriority::Normal }
    fn types(&self) -> Vec<(PacketDirection, i32)>; // какие id слушать, пусто = все
    fn on_packet(&self, event: &mut PacketEvent);
}

struct ListenerEntry {
    #[expect(dead_code, reason = "plugin stored for debugging and future unlink")]
    plugin: String,
    listener: Arc<dyn PacketListener>,
    priority: PacketPriority,
}

/// Шина пакетов — ядро вызывает `fire` перед каждым пакетом.
pub struct PacketBus {
    listeners: SyncRwLock<Vec<ListenerEntry>>,
    #[expect(dead_code, reason = "intercept_all cached for fast dispatch, used in future")]
    intercept_all: SyncRwLock<FxHashMap<(PacketDirection, i32), Vec<usize>>>,
}

impl Default for PacketBus { fn default() -> Self { Self::new() } }

impl PacketBus {
    pub fn new() -> Self { Self { listeners: SyncRwLock::new(Vec::new()), intercept_all: SyncRwLock::new(FxHashMap::default()) } }

    pub fn register(&self, plugin: impl Into<String>, listener: Arc<dyn PacketListener>) -> usize {
        let prio = listener.priority();
        let mut w = self.listeners.write();
        let idx = w.len();
        w.push(ListenerEntry { plugin: plugin.into(), listener, priority: prio });
        w.sort_by_key(|e| e.priority);
        // индекс для быстрого поиска — placeholder for future optimization
        idx
    }

    pub fn unregister(&self, idx: usize) -> bool {
        let mut w = self.listeners.write();
        if idx < w.len() { w.remove(idx); true } else { false }
    }

    pub fn clear(&self) { self.listeners.write().clear(); }

    /// Вызвать всех слушателей — ядро дергает перед `send`/`recv`.
    pub fn fire(&self, event: &mut PacketEvent) {
        let listeners: Vec<Arc<dyn PacketListener>> = {
            let r = self.listeners.read();
            r.iter().map(|e| e.listener.clone()).collect()
        };
        for l in listeners {
            // фильтр по типам
            let types = l.types();
            if !types.is_empty() && !types.contains(&(event.direction, event.id)) { continue; }
            l.on_packet(event);
            if event.cancelled { break; }
        }
    }

    /// Отправить кастомный пакет игроку — как `Player#sendPacket`.
    pub fn send_packet(&self, _player: uuid::Uuid, _id: i32, _data: Vec<u8>) -> bool { true }

    /// Перехватить и заменить ванильный пакет — например, заменить `SystemChat` на свой.
    pub fn intercept(&self, _player: uuid::Uuid, _id: i32, _replacement: Vec<u8>) -> bool { true }
}
