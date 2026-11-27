//! Инвентарь — клики, открытие, крафт, печка, энчант.

use crate::event::Cancellable;

#[derive(Debug, Clone)]
pub struct InventoryClickEvent {
    pub player_id: uuid::Uuid,
    pub slot: i32,
    pub click_type: String,
    pub cancelled: bool,
}
impl Cancellable for InventoryClickEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct InventoryOpenEvent {
    pub player_id: uuid::Uuid,
    pub title: String,
    pub cancelled: bool,
}
impl Cancellable for InventoryOpenEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct InventoryCloseEvent {
    pub player_id: uuid::Uuid,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct InventoryDragEvent {
    pub player_id: uuid::Uuid,
    pub slots: Vec<i32>,
    pub cancelled: bool,
}
impl Cancellable for InventoryDragEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct CraftItemEvent {
    pub player_id: uuid::Uuid,
    pub recipe: String,
    pub cancelled: bool,
}
impl Cancellable for CraftItemEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct FurnaceSmeltEvent {
    pub pos: steel_utils::BlockPos,
    pub source: String,
    pub result: String,
    pub cancelled: bool,
}
impl Cancellable for FurnaceSmeltEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct EnchantItemEvent {
    pub player_id: uuid::Uuid,
    pub item: String,
    pub level: i32,
    pub enchants: Vec<String>,
    pub cancelled: bool,
}
impl Cancellable for EnchantItemEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct PrepareAnvilEvent {
    pub player_id: uuid::Uuid,
    pub left: String,
    pub right: String,
    pub result: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PrepareGrindstoneEvent {
    pub player_id: uuid::Uuid,
    pub top: String,
    pub bottom: String,
    pub result: Option<String>,
}
