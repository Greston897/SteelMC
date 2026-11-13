//! Hopper block entity.
//!
//! Vanilla `HopperBlockEntity` holds five slots: an input row of four slots and
//! one output slot. It moves one item every eight game ticks (a "cooldown" of 8)
//! from its own inventory into the container below, and pulls one item from the
//! container above while it has room.

use std::{
    mem,
    sync::{Arc, Weak},
};

use simdnbt::ToNbtTag;
use simdnbt::borrow::{BaseNbtCompound as BorrowedNbtCompound, NbtCompound as NbtCompoundView};
use simdnbt::owned::{NbtCompound, NbtList, NbtTag};
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_block_entity_types;
use steel_utils::locks::SyncMutex;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey};

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::inventory::container::{Container, load_all_items};
use crate::inventory::lock::{ContainerLockGuard, ContainerRef, SharedContainer};
use crate::world::World;

/// Number of slots in a hopper.
pub const HOPPER_SLOTS: usize = 5;

/// Hopper block entity.
pub struct HopperBlockEntity {
    base: Arc<BlockEntityBase>,
    container: Arc<SyncMutex<HopperContainer>>,
    container_ref: ContainerRef,
    /// Remaining ticks before the next transfer, matching vanilla's `cooldownTime`.
    cooldown: SyncMutex<u8>,
}

pub(crate) struct HopperContainer {
    items: Vec<ItemStack>,
}

/// Vanilla `HopperBlockEntity.MOVE_ITEM_SPEED`: one transfer every eight ticks.
pub const MOVE_COOLDOWN_TICKS: u8 = 8;

// SAFETY: This key is owned by Steel and uniquely identifies `HopperBlockEntity`.
unsafe impl DowncastType for HopperBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/hopper");
}

// SAFETY: This key is owned by Steel and uniquely identifies the independently
// lockable inventory data used by a hopper block entity.
unsafe impl DowncastType for HopperContainer {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:container/hopper");
}

impl HopperBlockEntity {
    /// Creates a new hopper block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        let base = Arc::new(BlockEntityBase::new(
            &vanilla_block_entity_types::HOPPER,
            level,
            pos,
            state,
        ));
        let container = Arc::new(SyncMutex::new(HopperContainer {
            items: vec![ItemStack::empty(); HOPPER_SLOTS],
        }));
        let shared_container: SharedContainer =
            Arc::<SyncMutex<HopperContainer>>::clone(&container);
        Self {
            container_ref: ContainerRef::owned_by_block_entity(shared_container, Arc::clone(&base)),
            base,
            container,
            cooldown: SyncMutex::new(0),
        }
    }

    /// Vanilla `HopperBlockEntity.suckInItems`: pulls one item from the container
    /// above into this hopper.
    ///
    /// Returns whether an item moved. The source slot is only decremented once the
    /// hopper accepts the item, so a full hopper never deletes items.
    pub fn suck_in_items(&self, world: &World, pos: BlockPos) -> bool {
        let Some(source) = world
            .get_block_entity(pos.above())
            .and_then(ContainerRef::from_block_entity)
        else {
            return false;
        };
        let Some(slot) = ({
            let guard = ContainerLockGuard::lock_all(&[&source]);
            guard
                .get(source.container_id())
                .and_then(|container| {
                    (0..container.get_container_size())
                        .find(|slot| !container.get_item(*slot).is_empty())
                })
        }) else {
            return false;
        };
        let mut pulled = {
            let guard = ContainerLockGuard::lock_all(&[&source]);
            let Some(container) = guard.get(source.container_id()) else {
                return false;
            };
            container.get_item(slot).copy_with_count(1)
        };
        {
            let mut hopper = self.container.lock();
            if !hopper.add(&mut pulled) {
                return false;
            }
            hopper.set_changed();
        }
        // The hopper took the item, so remove exactly it from the source.
        let mut guard = ContainerLockGuard::lock_all(&[&source]);
        let Some(mut container) = guard.get_mut(source.container_id()) else {
            return false;
        };
        container.get_item_mut(slot).shrink(1);
        if container.get_item(slot).count() <= 0 {
            *container.get_item_mut(slot) = ItemStack::empty();
        }
        container.set_changed();
        true
    }

    /// Vanilla `HopperBlockEntity.tryMoveItems`: pushes one item from the hopper
    /// into the container below.
    ///
    /// Returns whether an item moved.
    pub fn try_move_items(&self, world: &World, pos: BlockPos) -> bool {
        let Some(destination) = world
            .get_block_entity(pos.below())
            .and_then(ContainerRef::from_block_entity)
        else {
            return false;
        };
        // Vanilla drains the output slot (0) first, then the four input slots.
        let Some(slot) = ({
            let hopper = self.container.lock();
            (0..HOPPER_SLOTS).find(|slot| !hopper.items[*slot].is_empty())
        }) else {
            return false;
        };
        let mut moving = {
            let mut hopper = self.container.lock();
            hopper.items[slot].copy_with_count(1)
        };
        let accepted = {
            let mut guard = ContainerLockGuard::lock_all(&[&destination]);
            let Some(mut container) = guard.get_mut(destination.container_id()) else {
                return false;
            };
            container.add(&mut moving)
        };
        if !moving.is_empty() {
            return false;
        }
        let mut hopper = self.container.lock();
        hopper.items[slot].shrink(1);
        if hopper.items[slot].count() <= 0 {
            hopper.items[slot] = ItemStack::empty();
        }
        hopper.set_changed();
        let _ = accepted;
        true
    }

    /// Remaining cooldown ticks before the next transfer.
    #[must_use]
    pub fn cooldown(&self) -> u8 {
        *self.cooldown.lock()
    }

    /// Sets the remaining cooldown, used by the block ticker.
    pub fn set_cooldown(&self, value: u8) {
        *self.cooldown.lock() = value;
    }

    /// Advances the transfer state by one game tick, matching vanilla's
    /// `HopperBlockEntity.pushItems` cooldown gate.
    pub fn tick_transfer(&self, world: &World, pos: BlockPos) {
        let mut cooldown = self.cooldown.lock();
        if *cooldown > 0 {
            *cooldown -= 1;
            return;
        }
        drop(cooldown);
        if !self.try_move_items(world, pos) {
            self.suck_in_items(world, pos);
        }
        self.set_cooldown(MOVE_COOLDOWN_TICKS);
    }
}

/// Returns a single item from the first non-empty slot of a container.
fn first_present_item(container: &dyn Container) -> Option<ItemStack> {
    for slot in 0..container.get_container_size() {
        let item = container.get_item(slot);
        if !item.is_empty() {
            return Some(item.copy_with_count(1));
        }
    }
    None
}

impl BlockEntity for HopperBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn pre_remove_side_effects(&self, pos: BlockPos, _state: BlockStateId) {
        let items = {
            let mut container = self.container.lock();
            mem::replace(&mut container.items, vec![ItemStack::empty(); HOPPER_SLOTS])
        };
        let Some(world) = self.get_level() else {
            return;
        };
        for item in items {
            world.drop_item_stack(pos, item);
        }
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let nbt_view: NbtCompoundView<'_, '_> = nbt.into();
        let mut container = self.container.lock();
        container.items.fill(ItemStack::empty());
        load_all_items(nbt_view, &mut container.items);
        drop(container);
        *self.cooldown.lock() = nbt_view.int("TransferCooldown").unwrap_or(0) as u8;
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        let container = self.container.lock();
        let mut items: Vec<NbtCompound> = Vec::new();
        for (slot, item) in container.items.iter().enumerate() {
            if !item.is_empty()
                && let NbtTag::Compound(mut item_nbt) = item.clone().to_nbt_tag()
            {
                item_nbt.insert("Slot", slot as i8);
                items.push(item_nbt);
            }
        }
        nbt.insert("Items", NbtList::Compound(items));
        drop(container);
        nbt.insert("TransferCooldown", i32::from(*self.cooldown.lock()));
    }

    fn container_ref(&self) -> Option<ContainerRef> {
        Some(self.container_ref.clone())
    }

    fn tick(&self, world: &Arc<World>) {
        // Vanilla ticks a hopper only while its `enabled` state permits it; the
        // block behavior already selects this ticker only for enabled hoppers.
        let pos = self.get_block_pos();
        self.tick_transfer(world, pos);
    }
}

impl Container for HopperContainer {
    fn items(&self) -> &[ItemStack] {
        &self.items
    }

    fn items_mut(&mut self) -> &mut [ItemStack] {
        &mut self.items
    }

    fn get_container_size(&self) -> usize {
        HOPPER_SLOTS
    }

    fn set_item(&mut self, slot: usize, mut stack: ItemStack) {
        if slot < HOPPER_SLOTS {
            let max_stack_size = self.get_max_stack_size_for_item(&stack);
            if !stack.is_empty() && stack.count() > max_stack_size {
                stack.set_count(max_stack_size);
            }
            self.items[slot] = stack;
        }
    }

    fn get_max_stack_size(&self) -> i32 {
        64
    }

    fn set_changed(&mut self) {}
}

#[cfg(test)]
mod tests {
    use simdnbt::borrow::read_compound;
    use steel_registry::{init_vanilla_registry, vanilla_blocks, vanilla_items};

    use super::*;

    fn test_hopper() -> HopperBlockEntity {
        init_vanilla_registry();
        HopperBlockEntity::new(
            Weak::new(),
            BlockPos::new(1, 2, 3),
            vanilla_blocks::HOPPER.default_state(),
        )
    }

    #[test]
    fn hopper_has_five_slots() {
        let hopper = test_hopper();
        assert_eq!(hopper.container.lock().items.len(), 5);
        assert_eq!(hopper.container.lock().get_container_size(), 5);
    }

    #[test]
    fn set_item_limits_stack_to_vanilla_container_maximum() {
        let hopper = test_hopper();
        hopper
            .container
            .lock()
            .set_item(0, ItemStack::with_count(&*vanilla_items::STONE, 100));
        assert_eq!(hopper.container.lock().get_item(0).count(), 64);
    }

    #[test]
    fn cooldown_starts_at_zero_and_round_trips() {
        let hopper = test_hopper();
        assert_eq!(hopper.cooldown(), 0);
        hopper.set_cooldown(MOVE_COOLDOWN_TICKS);
        assert_eq!(hopper.cooldown(), MOVE_COOLDOWN_TICKS);
    }

    #[test]
    fn cooldown_is_persisted_in_nbt() {
        let hopper = test_hopper();
        hopper.set_cooldown(5);
        let nbt = hopper.save_custom_only();
        let mut bytes = Vec::new();
        nbt.write(&mut bytes);
        let mut cursor = std::io::Cursor::new(bytes.as_slice());
        let borrowed = read_compound(&mut cursor).expect("compound");
        let view: NbtCompoundView<'_, '_> = (&borrowed).into();
        assert_eq!(view.int("TransferCooldown"), Some(5));
    }
}
