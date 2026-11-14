//! Shelf block entity.
//!
//! Vanilla `ShelfBlockEntity` holds three display items (`SLOT_COUNT = 3`), one
//! per shelf compartment, plus a `powered` flag mirrored from the block state.

use std::sync::{Arc, Weak};

use simdnbt::ToNbtTag;
use simdnbt::borrow::BaseNbtCompound as BorrowedNbtCompound;
use simdnbt::owned::{NbtCompound, NbtList, NbtTag};
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_block_entity_types;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey, locks::SyncMutex};

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::inventory::container::{Container, load_all_items};
use crate::inventory::lock::{ContainerRef, SharedContainer};
use crate::world::World;

/// Number of slots on a shelf.
pub const SHELF_SLOTS: usize = 3;

/// Shelf block entity.
pub struct ShelfBlockEntity {
    base: Arc<BlockEntityBase>,
    container: Arc<SyncMutex<ShelfContainer>>,
    container_ref: ContainerRef,
}

pub(crate) struct ShelfContainer {
    items: Vec<ItemStack>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `ShelfBlockEntity`.
unsafe impl DowncastType for ShelfBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/shelf");
}

// SAFETY: This key is owned by Steel and uniquely identifies the independently
// lockable inventory data used by a shelf.
unsafe impl DowncastType for ShelfContainer {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:container/shelf");
}

impl ShelfBlockEntity {
    /// Creates a new shelf block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        let base = Arc::new(BlockEntityBase::new(
            &vanilla_block_entity_types::SHELF,
            level,
            pos,
            state,
        ));
        let container = Arc::new(SyncMutex::new(ShelfContainer {
            items: vec![ItemStack::empty(); SHELF_SLOTS],
        }));
        let shared_container: SharedContainer =
            Arc::<SyncMutex<ShelfContainer>>::clone(&container);
        Self {
            container_ref: ContainerRef::owned_by_block_entity(shared_container, Arc::clone(&base)),
            base,
            container,
        }
    }
}

impl BlockEntity for ShelfBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn pre_remove_side_effects(&self, pos: BlockPos, _state: BlockStateId) {
        let items = {
            let mut container = self.container.lock();
            std::mem::replace(&mut container.items, vec![ItemStack::empty(); SHELF_SLOTS])
        };
        let Some(world) = self.get_level() else {
            return;
        };
        for item in items {
            world.drop_item_stack(pos, item);
        }
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let view: simdnbt::borrow::NbtCompound<'_, '_> = nbt.into();
        let mut container = self.container.lock();
        container.items.fill(ItemStack::empty());
        load_all_items(view, &mut container.items);
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
    }

    fn container_ref(&self) -> Option<ContainerRef> {
        Some(self.container_ref.clone())
    }
}

impl Container for ShelfContainer {
    fn items(&self) -> &[ItemStack] {
        &self.items
    }

    fn items_mut(&mut self) -> &mut [ItemStack] {
        &mut self.items
    }

    fn get_container_size(&self) -> usize {
        SHELF_SLOTS
    }

    fn set_item(&mut self, slot: usize, mut stack: ItemStack) {
        if slot < SHELF_SLOTS {
            let max_stack_size = self.get_max_stack_size_for_item(&stack);
            if !stack.is_empty() && stack.count() > max_stack_size {
                stack.set_count(max_stack_size);
            }
            self.items[slot] = stack;
        }
    }

    fn get_max_stack_size(&self) -> i32 {
        1
    }

    fn set_changed(&mut self) {}
}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_blocks};

    use super::*;

    #[test]
    fn shelf_has_three_slots() {
        init_vanilla_registry();
        let shelf = ShelfBlockEntity::new(
            Weak::new(),
            BlockPos::new(0, 0, 0),
            vanilla_blocks::OAK_SHELF.default_state(),
        );
        assert_eq!(shelf.container.lock().items.len(), SHELF_SLOTS);
    }
}
