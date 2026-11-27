//! Инвентарь-API — контейнеры, слоты, вьюхи (как `Inventory` в Bukkit, но типобезопасно).

use steel_registry::item_stack::ItemStack;

/// Тип инвентаря.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryType {
    Chest,
    Hopper,
    Furnace,
    Crafting,
    Anvil,
    Enchanting,
    Beacon,
    ShulkerBox,
    Barrel,
    Player,
    Custom,
}

/// Видимая игроку вьюха инвентаря.
pub trait InventoryView: Send + Sync {
    fn title(&self) -> &str;
    fn set_title(&self, title: &str);
    fn top(&self) -> &dyn Inventory;
    fn bottom(&self) -> &dyn Inventory;
    fn player_id(&self) -> uuid::Uuid;
}

/// Сам инвентарь — массив слотов.
pub trait Inventory: Send + Sync {
    fn size(&self) -> usize;
    fn inventory_type(&self) -> InventoryType { InventoryType::Custom }
    fn get_item(&self, slot: usize) -> Option<ItemStack>;
    fn set_item(&self, slot: usize, item: Option<ItemStack>) -> Option<ItemStack>;
    fn add_item(&self, item: ItemStack) -> Option<ItemStack>;
    fn remove_item(&self, slot: usize) -> Option<ItemStack>;
    fn clear(&self);
    fn is_empty(&self) -> bool;
    fn contains(&self, item: &str) -> bool;
    fn first_empty(&self) -> Option<usize>;
    fn viewers(&self) -> Vec<uuid::Uuid>;
}
