//! Vanilla menu kind implementations.

mod anvil_menu;
mod basic_menu;
mod beacon_menu;
mod chest_boat_menu;
mod chest_menu;
mod container_menus;
mod crafting_menu;
mod crafter_menu;
mod enchantment_menu;
mod ender_chest_menu;
mod furnace_menu;
mod grindstone_menu;
mod inventory_menu;
mod shulker_box_menu;
mod station_menu;

pub use anvil_menu::{AnvilKind, anvil};
pub use basic_menu::BasicKind;
pub use beacon_menu::{BeaconKind, beacon};
pub use chest_boat_menu::{CHEST_BOAT_ROWS, ChestBoatKind, chest_boat};
pub use chest_menu::{ChestKind, DoubleChestKind, chest, double_chest};
pub use container_menus::{brewing_stand, dispenser, hopper, lectern};
pub use crafting_menu::{CraftingKind, crafting};
pub use crafter_menu::{CrafterKind, crafter};
pub use enchantment_menu::{EnchantmentKind, enchantment};
pub use ender_chest_menu::{EnderChestKind, ender_chest};
pub use furnace_menu::{FurnaceMenuKind, furnace_menu};
pub use grindstone_menu::{GrindstoneKind, grindstone};
pub use inventory_menu::{INVENTORY_MENU_CONTAINER_ID, InventoryKind, inventory_menu};
pub use shulker_box_menu::{ShulkerBoxKind, shulker_box};
pub use station_menu::{StationKind, cartography_table, loom, smithing, stonecutter};
