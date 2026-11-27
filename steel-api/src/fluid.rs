//! Флюид-API — вода, лава, кастомные флюиды.
//! Плагин может регистрировать свой `FluidBehaviorHook` как для блоков.

use rustc_hash::FxHashMap;
use steel_utils::{BlockPos, Identifier};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FluidType {
    Empty, Water, Lava, Custom(u32),
}

#[derive(Debug, Clone)]
pub struct FluidState {
    pub ty: FluidType,
    pub level: u8,
    pub falling: bool,
    pub source: bool,
}

impl FluidState {
    pub fn empty() -> Self { Self { ty: FluidType::Empty, level: 0, falling: false, source: false } }
    pub fn water(level: u8) -> Self { Self { ty: FluidType::Water, level, falling: false, source: level==8 } }
    pub fn lava(level: u8) -> Self { Self { ty: FluidType::Lava, level, falling: false, source: level==8 } }
    pub fn is_source(&self) -> bool { self.source }
    pub fn is_empty(&self) -> bool { self.ty==FluidType::Empty }
}

pub trait FluidBehaviorHook: Send + Sync + 'static {
    fn can_flow(&self, _from: BlockPos, _to: BlockPos, _world: &dyn crate::world::World) -> bool { true }
    fn flow_speed(&self) -> i32 { 5 }
    fn level_decrease(&self) -> i32 { 1 }
    fn can_convert_to_source(&self) -> bool { false }
    fn tick(&self, _pos: BlockPos, _world: &dyn crate::world::World) {}
    fn on_placed(&self, _pos: BlockPos, _world: &dyn crate::world::World) {}
}

/// Реестр флюидов — в `PluginContext`.
pub struct FluidRegistry {
    hooks: steel_utils::locks::SyncRwLock<rustc_hash::FxHashMap<Identifier, Box<dyn FluidBehaviorHook>>>,
}

impl Default for FluidRegistry { fn default() -> Self { Self::new() } }

impl FluidRegistry {
    pub fn new() -> Self { Self { hooks: steel_utils::locks::SyncRwLock::new(Default::default()) } }
    pub fn register(&self, id: Identifier, hook: Box<dyn FluidBehaviorHook>) -> bool {
        let mut w=self.hooks.write();
        if w.contains_key(&id) { return false; }
        w.insert(id, hook); true
    }
    pub fn get(&self, id: &Identifier) -> Option<*const dyn FluidBehaviorHook> { None }
    pub fn contains(&self, id: &Identifier) -> bool { self.hooks.read().contains_key(id) }
}
