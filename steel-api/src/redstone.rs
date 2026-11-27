//! Редстоун-API — полный контроль над `BlockRedstoneEvent`, `RedstoneWire`, `Comparator`, `Repeater`.
//! Paper `BlockRedstoneEvent` + Leaf оптимизации.

use rustc_hash::FxHashMap;
use steel_utils::BlockPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedstonePower {
    None,
    Weak(u8),
    Strong(u8),
}

impl RedstonePower {
    pub fn level(self) -> u8 { match self { Self::None=>0, Self::Weak(v)|Self::Strong(v)=>v } }
    pub fn is_powered(self) -> bool { self.level() > 0 }
}

pub trait RedstoneHook: Send + Sync + 'static {
    fn get_weak_power(&self, _pos: BlockPos, _world: &dyn crate::world::World, _side: u8) -> u8 { 0 }
    fn get_strong_power(&self, _pos: BlockPos, _world: &dyn crate::world::World, _side: u8) -> u8 { 0 }
    fn is_conductor(&self, _pos: BlockPos, _world: &dyn crate::world::World) -> Option<bool> { None }
    fn on_power_change(&self, _pos: BlockPos, _old: u8, _new: u8, _world: &dyn crate::world::World) {}
}

pub struct RedstoneRegistry {
    hooks: steel_utils::locks::SyncRwLock<rustc_hash::FxHashMap<steel_utils::Identifier, Box<dyn RedstoneHook>>>,
}

impl Default for RedstoneRegistry { fn default() -> Self { Self::new() } }

impl RedstoneRegistry {
    pub fn new() -> Self { Self { hooks: steel_utils::locks::SyncRwLock::new(Default::default()) } }
    pub fn register(&self, id: steel_utils::Identifier, hook: Box<dyn RedstoneHook>) -> bool {
        let mut w=self.hooks.write();
        if w.contains_key(&id) { return false; }
        w.insert(id, hook); true
    }
    pub fn unregister(&self, id: &steel_utils::Identifier) -> bool { self.hooks.write().remove(id).is_some() }
}
