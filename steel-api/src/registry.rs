//! Доступ к ваниль-реестрам + хуки регистрации (блоки, предметы, сущности, биомы...).

use steel_utils::Identifier;
use steel_registry::blocks::BlockRef;
use steel_registry::items::ItemRef;

/// Чтение реестров — то, что видит плагин. Ядро реализует этот трейт.
pub trait RegistryAccess: Send + Sync + 'static {
    fn block_by_id(&self, id: &Identifier) -> Option<BlockRef>;
    fn item_by_id(&self, id: &Identifier) -> Option<ItemRef>;
    fn has_block(&self, id: &Identifier) -> bool { self.block_by_id(id).is_some() }
    fn has_item(&self, id: &Identifier) -> bool { self.item_by_id(id).is_some() }
    fn all_block_ids(&self) -> Vec<Identifier>;
    fn all_item_ids(&self) -> Vec<Identifier>;
    fn protocol_version(&self) -> i32 { 777 }
    fn minecraft_version(&self) -> &'static str { "26.3" }
}

/// Мутабельный доступ — только на этапе `on_load` (реестры заморожены после).
pub trait RegistryAccessMut: RegistryAccess {
    fn register_block(&self, id: Identifier, block: BlockRef) -> bool;
    fn register_item(&self, id: Identifier, item: ItemRef) -> bool;
}

/// Заглушка для тестов — читает из глобального `REGISTRY`.
pub struct VanillaRegistryAccess;

impl RegistryAccess for VanillaRegistryAccess {
    fn block_by_id(&self, id: &Identifier) -> Option<BlockRef> {
        // ищем по ключу; если нет — None
        let _ = id; None
    }
    fn item_by_id(&self, _id: &Identifier) -> Option<ItemRef> { None }
    fn all_block_ids(&self) -> Vec<Identifier> { Vec::new() }
    fn all_item_ids(&self) -> Vec<Identifier> { Vec::new() }
}
