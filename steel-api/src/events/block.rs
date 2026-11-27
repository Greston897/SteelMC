//! Блочные события — стройка, редстоун, физика, жидкости, рост.
use steel_utils::{BlockPos, Identifier};
use steel_registry::blocks::BlockRef;
use crate::event::Cancellable;

// — базовые —
#[derive(Debug, Clone)]
pub struct BlockBreakEvent {
    pub player_id: uuid::Uuid,
    pub pos: BlockPos,
    pub block: BlockRef,
    pub cancelled: bool,
    pub drop_items: bool,
    pub exp_to_drop: i32,
}
impl Cancellable for BlockBreakEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct BlockPlaceEvent {
    pub player_id: uuid::Uuid,
    pub pos: BlockPos,
    pub placed: BlockRef,
    pub replaced: BlockRef,
    pub cancelled: bool,
}
impl Cancellable for BlockPlaceEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct BlockInteractEvent {
    pub player_id: uuid::Uuid,
    pub pos: BlockPos,
    pub block: BlockRef,
    pub hand: u8,
    pub cancelled: bool,
}
impl Cancellable for BlockInteractEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct BlockPhysicsEvent {
    pub pos: BlockPos,
    pub block: BlockRef,
    pub neighbour_pos: BlockPos,
    pub cancelled: bool,
}
impl Cancellable for BlockPhysicsEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct BlockTickEvent {
    pub pos: BlockPos,
    pub block: BlockRef,
}

#[derive(Debug, Clone)]
pub struct BlockRandomTickEvent {
    pub pos: BlockPos,
    pub block: BlockRef,
}

#[derive(Debug, Clone)]
pub struct BlockGrowEvent {
    pub pos: BlockPos,
    pub block: BlockRef,
    pub new_block: BlockRef,
    pub cancelled: bool,
}
impl Cancellable for BlockGrowEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct BlockFromToEvent {
    pub from: BlockPos,
    pub to: BlockPos,
    pub block: BlockRef,
    pub cancelled: bool,
}
impl Cancellable for BlockFromToEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct BlockFadeEvent {
    pub pos: BlockPos,
    pub block: BlockRef,
    pub new_block: BlockRef,
    pub cancelled: bool,
}
impl Cancellable for BlockFadeEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct BlockFormEvent {
    pub pos: BlockPos,
    pub block: BlockRef,
    pub cancelled: bool,
}
impl Cancellable for BlockFormEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct BlockSpreadEvent {
    pub source: BlockPos,
    pub target: BlockPos,
    pub block: BlockRef,
    pub cancelled: bool,
}
impl Cancellable for BlockSpreadEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct BlockPistonExtendEvent {
    pub pos: BlockPos,
    pub direction: u8,
    pub blocks: Vec<BlockPos>,
    pub cancelled: bool,
}
impl Cancellable for BlockPistonExtendEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct BlockPistonRetractEvent {
    pub pos: BlockPos,
    pub direction: u8,
    pub cancelled: bool,
}
impl Cancellable for BlockPistonRetractEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct BlockRedstoneEvent {
    pub pos: BlockPos,
    pub old_power: u8,
    pub new_power: u8,
}

#[derive(Debug, Clone)]
pub struct BlockExplodeEvent {
    pub pos: BlockPos,
    pub block: BlockRef,
    pub yield_rate: f32,
    pub cancelled: bool,
}
impl Cancellable for BlockExplodeEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct FluidLevelChangeEvent {
    pub pos: BlockPos,
    pub fluid: Identifier,
    pub new_level: u8,
    pub cancelled: bool,
}
impl Cancellable for FluidLevelChangeEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct CauldronLevelChangeEvent {
    pub pos: BlockPos,
    pub old_level: i32,
    pub new_level: i32,
    pub reason: String,
    pub cancelled: bool,
}
impl Cancellable for CauldronLevelChangeEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct NotePlayEvent {
    pub pos: BlockPos,
    pub instrument: String,
    pub note: u8,
    pub cancelled: bool,
}
impl Cancellable for NotePlayEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}
