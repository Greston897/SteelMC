//! Shared equipment storage for rideable animals.
//!
//! Vanilla `AbstractHorse` owns one `SimpleContainer` whose first two slots are
//! the saddle and the body armor and whose remaining slots are the mount's
//! storage. The mount menu reads that container directly, so the entity has to
//! keep the same shape: growing the size when a chest is added must keep the
//! existing items, which is what `AbstractHorse.createInventory` does.

use std::sync::Weak;

use simdnbt::ToNbtTag as _;
use simdnbt::borrow::NbtCompound as NbtCompoundView;
use simdnbt::owned::{NbtCompound, NbtList, NbtTag};
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_items;
use steel_utils::locks::{IntoShared as _, SyncMutex};

use crate::entity::Entity;
use crate::inventory::container::{Container as _, SimpleContainer};
use crate::inventory::menu::kinds::mount_menu;
use crate::inventory::prelude::*;
use crate::player::Player;

/// The saddle and armor container of one mount.
pub struct MountInventory {
    container: Shared<SimpleContainer>,
    columns: usize,
}

impl MountInventory {
    /// Builds a mount inventory with `columns` storage columns.
    #[must_use]
    pub fn new(columns: usize) -> Self {
        let size = mount_menu::inventory_size(columns);
        Self {
            container: SimpleContainer::new(size).into_shared(),
            columns,
        }
    }

    /// Returns the connection-facing container, which the menu reads.
    #[must_use]
    pub fn container(&self) -> &Shared<SimpleContainer> {
        &self.container
    }

    /// Container handle for the menu builder, which only accepts `ContainerRef`.
    #[must_use]
    pub fn container_ref(&self) -> ContainerRef {
        Shared::clone(&self.container).into()
    }

    /// Returns the number of storage columns.
    #[must_use]
    pub const fn columns(&self) -> usize {
        self.columns
    }

    /// Vanilla `AbstractHorse.createInventory`: keep the items that still fit.
    pub fn resize(&mut self, columns: usize) {
        let size = mount_menu::inventory_size(columns);
        let old = self.container.lock();
        let mut items = Vec::with_capacity(size);
        for slot in 0..size {
            items.push(if slot < old.get_container_size() {
                old.get_item(slot).clone()
            } else {
                ItemStack::empty()
            });
        }
        drop(old);
        self.container = SimpleContainer::from_items(items).into_shared();
        self.columns = columns;
    }

    /// The saddle currently in the saddle slot, if any.
    #[must_use]
    pub fn saddle(&self) -> Option<ItemStack> {
        let stack = self.container.lock().get_item(mount_menu::SLOT_SADDLE).clone();
        (!stack.is_empty()).then_some(stack)
    }

    /// Vanilla `AbstractChestedHorse.addAdditionalSaveData`: every occupied slot
    /// is written with its index so the layout survives a save.
    pub fn save_items(&self, nbt: &mut NbtCompound) {
        let container = self.container.lock();
        let mut items: Vec<NbtCompound> = Vec::new();
        for slot in 0..container.get_container_size() {
            let item = container.get_item(slot);
            if item.is_empty() {
                continue;
            }
            if let NbtTag::Compound(mut item_nbt) = item.clone().to_nbt_tag() {
                item_nbt.insert("Slot", slot as i8);
                items.push(item_nbt);
            }
        }
        drop(container);
        nbt.insert("Items", NbtList::Compound(items));
    }

    /// Reads the list written by [`Self::save_items`].
    ///
    /// Vanilla `AbstractHorse.readAdditionalSaveData` always uses the `Items`
    /// key, so the field name is not a parameter. The borrowed NBT view is taken
    /// by value because that is what the callers already hold.
    pub fn load_items(&self, nbt: NbtCompoundView<'_, '_>) {
        let mut container = self.container.lock();
        let slots = container.items_mut();
        slots.fill(ItemStack::empty());
        crate::inventory::container::load_all_items(nbt, slots);
    }

    /// Whether the mount carries a saddle.
    #[must_use]
    pub fn has_saddle(&self) -> bool {
        self.saddle().is_some()
    }
}

/// Vanilla `AbstractChestedHorse.getInventoryColumns`: a chested donkey or mule
/// shows five storage columns, an unchested one opens no storage at all.
///
/// The saddle slot still exists without a chest, so the container keeps one
/// column of room and [`open_mount_menu`] is what refuses to open the screen.
#[must_use]
pub const fn chested_columns(has_chest: bool) -> usize {
    if has_chest { 5 } else { 1 }
}

/// Vanilla `LivingEntity.isEquippableInSlot(stack, EquipmentSlot.SADDLE)`.
#[must_use]
pub fn is_saddle(stack: &ItemStack) -> bool {
    stack.is(&*vanilla_items::SADDLE)
}

/// Vanilla `AbstractHorse.openCustomInventoryScreen`.
///
/// Only a tamed mount opens its inventory; callers check that with
/// `is_tamed()` before calling in, matching vanilla's own guard.
///
/// Only a tamed mount opens its inventory, and only for players who are not
/// already steering it unless they are the one riding. The screen is sent with
/// `ClientboundMountScreenOpenPacket` — the client cannot build it from
/// `ClientboundOpenScreenPacket`, which has no entity or column count.
pub fn open_mount_menu(
    mount: &dyn Entity,
    player: &Player,
    inventory: &Shared<SimpleContainer>,
    columns: usize,
) {
    // Vanilla `getInventoryColumns` returns 0 for a donkey or mule without a
    // chest; the mount screen would then have no storage rows at all, so there
    // is nothing to open.
    if columns == 0 {
        return;
    }
    let Some(world) = mount.level() else {
        return;
    };
    let Some(shared) = world.get_entity_by_id(mount.id()) else {
        return;
    };
    let owner: Weak<dyn Entity> = std::sync::Arc::downgrade(&shared);
    drop(shared);

    player.open_entity_menu(
        mount.display_name(),
        mount.id(),
        columns,
        std::sync::Arc::clone(inventory),
        owner,
    );
}

/// Vanilla `AbstractHorse` moves the body armor out of the mount when the menu
/// closes, so returning the equipment has to know about it.
#[must_use]
pub fn body_armor(inventory: &Shared<SimpleContainer>) -> Option<ItemStack> {
    let stack = inventory
        .lock()
        .get_item(mount_menu::SLOT_BODY_ARMOR)
        .clone();
    (!stack.is_empty()).then_some(stack)
}

/// Synchronized mount storage, since `AbstractHorse` is ticked and interacted
/// with from several threads.
pub type SharedMountInventory = std::sync::Arc<SyncMutex<MountInventory>>;

/// Creates the shared mount storage for an entity.
#[must_use]
pub fn shared_inventory(columns: usize) -> SharedMountInventory {
    std::sync::Arc::new(SyncMutex::new(MountInventory::new(columns)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resizing_keeps_the_saddle() {
        let mut inventory = MountInventory::new(1);
        inventory
            .container
            .lock()
            .set_item(
                mount_menu::SLOT_SADDLE,
                ItemStack::new(&vanilla_items::SADDLE),
            );

        inventory.resize(5);

        assert!(inventory.has_saddle());
        assert_eq!(inventory.columns(), 5);
    }

    #[test]
    fn a_fresh_mount_inventory_has_no_saddle() {
        assert!(!MountInventory::new(5).has_saddle());
    }
}
