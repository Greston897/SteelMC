//! Предмет-API + хуки.

use steel_registry::items::ItemRef;
use steel_registry::item_stack::ItemStack as RegistryStack;

/// Удобная обёртка — тонкий newtype над registry `ItemStack` для плагинов.
#[derive(Debug, Clone)]
pub struct ItemStack {
    inner: RegistryStack,
}

impl ItemStack {
    pub fn from_inner(inner: RegistryStack) -> Self { Self { inner } }
    pub fn into_inner(self) -> RegistryStack { self.inner }
    pub fn inner(&self) -> &RegistryStack { &self.inner }
    pub fn inner_mut(&mut self) -> &mut RegistryStack { &mut self.inner }

    pub fn empty() -> Self {
        Self { inner: RegistryStack::empty() }
    }
    pub fn of(item: ItemRef, count: i32) -> Self {
        Self { inner: RegistryStack::with_count(item, count) }
    }
    pub fn is_empty(&self) -> bool { self.inner.is_empty() }
    pub fn count(&self) -> i32 { self.inner.count() }
    pub fn set_count(&mut self, c: i32) { self.inner.set_count(c); }
    pub fn item(&self) -> ItemRef { self.inner.item }
}

pub trait ItemBehaviorHook: Send + Sync + 'static {
    fn on_use(&self, _stack: &mut ItemStack, _player: uuid::Uuid) -> bool { true }
    fn on_use_on_block(&self, _stack: &mut ItemStack, _player: uuid::Uuid, _pos: steel_utils::BlockPos) -> bool { true }
    fn on_attack_entity(&self, _stack: &mut ItemStack, _attacker: uuid::Uuid, _target: i32) -> bool { true }
    fn on_mine_block(&self, _stack: &mut ItemStack, _pos: steel_utils::BlockPos) -> bool { true }
    fn on_tick(&self, _stack: &mut ItemStack, _entity: i32) {}
    fn on_consume(&self, _stack: &mut ItemStack, _entity: i32) {}
}
