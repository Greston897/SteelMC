//! Мир — чанки, погода, время, генерация, порталы.
use steel_utils::{BlockPos, ChunkPos};
use crate::event::Cancellable;

#[derive(Debug, Clone)]
pub struct WorldLoadEvent {
    pub world_name: String,
}

#[derive(Debug, Clone)]
pub struct WorldUnloadEvent {
    pub world_name: String,
    pub cancelled: bool,
}
impl Cancellable for WorldUnloadEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct WorldSaveEvent {
    pub world_name: String,
}

#[derive(Debug, Clone)]
pub struct ChunkLoadEvent {
    pub world_name: String,
    pub pos: ChunkPos,
}

#[derive(Debug, Clone)]
pub struct ChunkUnloadEvent {
    pub world_name: String,
    pub pos: ChunkPos,
    pub cancelled: bool,
}
impl Cancellable for ChunkUnloadEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct ChunkPopulateEvent {
    pub world_name: String,
    pub pos: ChunkPos,
}

#[derive(Debug, Clone)]
pub struct WeatherChangeEvent {
    pub world_name: String,
    pub to_storm: bool,
    pub cancelled: bool,
}
impl Cancellable for WeatherChangeEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct TimeSkipEvent {
    pub world_name: String,
    pub skip_amount: i64,
    pub reason: String,
    pub cancelled: bool,
}
impl Cancellable for TimeSkipEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct PortalCreateEvent {
    pub world_name: String,
    pub pos: BlockPos,
    pub reason: String,
    pub cancelled: bool,
}
impl Cancellable for PortalCreateEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct StructureGrowEvent {
    pub world_name: String,
    pub pos: BlockPos,
    pub structure: String,
    pub cancelled: bool,
}
impl Cancellable for StructureGrowEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct WorldGameRuleChangeEvent {
    pub world_name: String,
    pub rule: String,
    pub value: String,
    pub cancelled: bool,
}
impl Cancellable for WorldGameRuleChangeEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}
