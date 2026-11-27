//! Сервер — тик, команды, логин, пакеты, плагины.

use crate::event::Cancellable;

#[derive(Debug, Clone)]
pub struct ServerTickEvent {
    pub tick: u64,
}

#[derive(Debug, Clone)]
pub struct ServerCommandEvent {
    pub sender: String,
    pub command: String,
    pub cancelled: bool,
}
impl Cancellable for ServerCommandEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct ServerListPingEvent {
    pub address: String,
    pub protocol: i32,
    pub motd: String,
    pub max_players: i32,
    pub cancelled: bool,
}
impl Cancellable for ServerListPingEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct PlayerLoginEvent {
    pub player_id: uuid::Uuid,
    pub player_name: String,
    pub address: String,
    pub result: String,
    pub kick_message: Option<String>,
    pub cancelled: bool,
}
impl Cancellable for PlayerLoginEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct PlayerPreLoginEvent {
    pub player_name: String,
    pub address: String,
    pub cancelled: bool,
}
impl Cancellable for PlayerPreLoginEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct PluginEnableEvent {
    pub plugin_name: String,
}

#[derive(Debug, Clone)]
pub struct PluginDisableEvent {
    pub plugin_name: String,
}

#[derive(Debug, Clone)]
pub struct PacketReceiveEvent {
    pub player_id: Option<uuid::Uuid>,
    pub protocol: String,
    pub packet_id: i32,
    pub cancelled: bool,
}
impl Cancellable for PacketReceiveEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct PacketSendEvent {
    pub player_id: uuid::Uuid,
    pub packet_id: i32,
    pub cancelled: bool,
}
impl Cancellable for PacketSendEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct ScoreboardEvent {
    pub objective: String,
    pub player: Option<String>,
    pub score: i32,
}

#[derive(Debug, Clone)]
pub struct BroadcastEvent {
    pub message: String,
    pub recipients: Vec<uuid::Uuid>,
    pub cancelled: bool,
}
impl Cancellable for BroadcastEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}
