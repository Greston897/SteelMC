//! Игрок — вход, выход, чат, движение, взаимодействие.
use glam::DVec3;
use steel_utils::BlockPos;
use crate::event::Cancellable;

#[derive(Debug, Clone)]
pub struct PlayerJoinEvent {
    pub player_id: uuid::Uuid,
    pub player_name: String,
    pub join_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PlayerQuitEvent {
    pub player_id: uuid::Uuid,
    pub player_name: String,
    pub quit_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PlayerChatEvent {
    pub player_id: uuid::Uuid,
    pub message: String,
    pub cancelled: bool,
}
impl Cancellable for PlayerChatEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct PlayerCommandPreprocessEvent {
    pub player_id: uuid::Uuid,
    pub command: String,
    pub cancelled: bool,
}
impl Cancellable for PlayerCommandPreprocessEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct PlayerMoveEvent {
    pub player_id: uuid::Uuid,
    pub from: DVec3,
    pub to: DVec3,
    pub cancelled: bool,
}
impl Cancellable for PlayerMoveEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct PlayerInteractEvent {
    pub player_id: uuid::Uuid,
    pub pos: Option<BlockPos>,
    pub hand: u8,
    pub cancelled: bool,
}
impl Cancellable for PlayerInteractEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct PlayerInteractEntityEvent {
    pub player_id: uuid::Uuid,
    pub entity_id: i32,
    pub hand: u8,
    pub cancelled: bool,
}
impl Cancellable for PlayerInteractEntityEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct PlayerDropItemEvent {
    pub player_id: uuid::Uuid,
    pub item: String,
    pub cancelled: bool,
}
impl Cancellable for PlayerDropItemEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct PlayerPickupItemEvent {
    pub player_id: uuid::Uuid,
    pub entity_id: i32,
    pub cancelled: bool,
}
impl Cancellable for PlayerPickupItemEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct PlayerRespawnEvent {
    pub player_id: uuid::Uuid,
    pub respawn_pos: Option<BlockPos>,
}

#[derive(Debug, Clone)]
pub struct PlayerGameModeChangeEvent {
    pub player_id: uuid::Uuid,
    pub old_mode: String,
    pub new_mode: String,
    pub cancelled: bool,
}
impl Cancellable for PlayerGameModeChangeEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct PlayerExpChangeEvent {
    pub player_id: uuid::Uuid,
    pub amount: i32,
}

#[derive(Debug, Clone)]
pub struct PlayerItemConsumeEvent {
    pub player_id: uuid::Uuid,
    pub item: String,
    pub cancelled: bool,
}
impl Cancellable for PlayerItemConsumeEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct PlayerAdvancementDoneEvent {
    pub player_id: uuid::Uuid,
    pub advancement: String,
}
