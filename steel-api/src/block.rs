//! Блок-API: обёртка над `BlockStateId` + хуки поведения (замена любой механики).

use steel_utils::{BlockPos, BlockStateId, Identifier};
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;

/// Лёгкая обёртка над блоком в мире.
#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub pos: BlockPos,
    pub state: BlockStateId,
    pub block: BlockRef,
}

impl Block {
    pub fn new(pos: BlockPos, state: BlockStateId, block: BlockRef) -> Self {
        Self { pos, state, block }
    }
    pub fn is_air(&self) -> bool {
        self.state.is_air()
    }
    pub fn pos(&self) -> BlockPos { self.pos }
    pub fn state(&self) -> BlockStateId { self.state }
    pub fn block(&self) -> BlockRef { self.block }
}

/// Хуки для переопределения любой блочной механики (аналог `BlockBehavior` в `steel-core`,
/// но плагин-ориентировано).
pub trait BlockBehaviorHook: Send + Sync + 'static {
    fn on_place(&self, _block: &Block, _player: Option<uuid::Uuid>) -> bool { true }
    fn on_break(&self, _block: &Block, _player: Option<uuid::Uuid>) -> bool { true }
    fn on_interact(&self, _block: &Block, _player: uuid::Uuid) -> bool { true }
    fn on_tick(&self, _block: &Block) {}
    fn on_random_tick(&self, _block: &Block) {}
    fn on_entity_inside(&self, _block: &Block, _entity: i32) {}
    fn on_step_on(&self, _block: &Block, _entity: i32) {}
    fn on_fall_on(&self, _block: &Block, _entity: i32, _fall_distance: f32) {}
    fn on_redstone_power(&self, _block: &Block, _power: u8) {}
    fn on_explode(&self, _block: &Block, _yield: f32) -> f32 { _yield }
    fn is_redstone_conductor(&self, _block: &Block) -> Option<bool> { None }
}

/// Реестр хуков — хранится в `PluginContext`.
pub struct BlockHooks {
    hooks: parking_lot::RwLock<std::collections::HashMap<Identifier, Box<dyn BlockBehaviorHook>>>,
}

impl Default for BlockHooks {
    fn default() -> Self { Self::new() }
}

impl BlockHooks {
    pub fn new() -> Self {
        Self { hooks: parking_lot::RwLock::new(Default::default()) }
    }
    pub fn register(&self, id: Identifier, hook: Box<dyn BlockBehaviorHook>) {
        self.hooks.write().insert(id, hook);
    }
    pub fn unregister(&self, id: &Identifier) -> bool {
        self.hooks.write().remove(id).is_some()
    }
    pub fn contains(&self, id: &Identifier) -> bool {
        self.hooks.read().contains_key(id)
    }
    pub fn len(&self) -> usize { self.hooks.read().len() }
    pub fn is_empty(&self) -> bool { self.hooks.read().is_empty() }
}
