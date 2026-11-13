//! Chest and trapped-chest block entity.
//!
//! Vanilla `ChestBlockEntity` holds 27 slots. Chunk loader and hopper
//! interaction live in the block behavior and hopper systems respectively; this
//! type owns the storage, persistence and the `preRemoveSideEffects` spill.

use std::{
    mem,
    sync::{Arc, Weak},
};

use simdnbt::ToNbtTag;
use simdnbt::borrow::{BaseNbtCompound as BorrowedNbtCompound, NbtCompound as NbtCompoundView};
use simdnbt::owned::{NbtCompound, NbtList, NbtTag};
use steel_registry::block_entity_type::BlockEntityTypeRef;
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_block_entity_types;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey, locks::SyncMutex};

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::inventory::container::Container;
use crate::inventory::container::load_all_items;
use crate::inventory::lock::{ContainerRef, SharedContainer};
use crate::inventory::openers_counter::ContainerOpenersCounter;
use crate::world::World;

/// Number of slots in a single chest block (3 rows of 9).
pub const CHEST_SLOTS: usize = 27;

/// Chest block entity.
pub struct ChestBlockEntity {
    base: Arc<BlockEntityBase>,
    container: Arc<SyncMutex<ChestContainer>>,
    container_ref: ContainerRef,
    openers_counter: ContainerOpenersCounter,
}

pub(crate) struct ChestContainer {
    items: Vec<ItemStack>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `ChestBlockEntity`.
unsafe impl DowncastType for ChestBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/chest");
}

// SAFETY: This key is owned by Steel and uniquely identifies the independently
// lockable inventory data used by a chest block entity.
unsafe impl DowncastType for ChestContainer {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:container/chest");
}

impl ChestBlockEntity {
    /// Creates a chest block entity for the given registered type.
    ///
    /// Vanilla uses one class for both the plain and trapped chest, keyed by the
    /// block entity type, so the type is a parameter here.
    #[must_use]
    pub fn new(
        block_entity_type: BlockEntityTypeRef,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> Self {
        let base = Arc::new(BlockEntityBase::new(block_entity_type, level, pos, state));
        let container = Arc::new(SyncMutex::new(ChestContainer {
            items: vec![ItemStack::empty(); CHEST_SLOTS],
        }));
        let shared_container: SharedContainer = Arc::<SyncMutex<ChestContainer>>::clone(&container);
        Self {
            container_ref: ContainerRef::owned_by_block_entity(shared_container, Arc::clone(&base)),
            base,
            container,
            openers_counter: ContainerOpenersCounter::without_open_state(
                &steel_registry::sound_events::BLOCK_CHEST_OPEN,
                &steel_registry::sound_events::BLOCK_CHEST_CLOSE,
            ),
        }
    }

    /// Creates a plain chest block entity.
    #[must_use]
    pub fn new_chest(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self::new(&vanilla_block_entity_types::CHEST, level, pos, state)
    }

    /// Creates a trapped chest block entity.
    #[must_use]
    pub fn new_trapped(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self::new(&vanilla_block_entity_types::TRAPPED_CHEST, level, pos, state)
    }

    /// Viewer counter used for the double-chest lid animation and comparator.
    #[must_use]
    pub fn openers_counter(&self) -> &ContainerOpenersCounter {
        &self.openers_counter
    }
}

impl BlockEntity for ChestBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn pre_remove_side_effects(&self, pos: BlockPos, _state: BlockStateId) {
        let items = {
            let mut container = self.container.lock();
            mem::replace(&mut container.items, vec![ItemStack::empty(); CHEST_SLOTS])
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

    fn get_update_tag(&self) -> Option<NbtCompound> {
        // Chests do not sync their contents on chunk load.
        None
    }

    fn container_ref(&self) -> Option<ContainerRef> {
        Some(self.container_ref.clone())
    }
}

impl Container for ChestContainer {
    fn items(&self) -> &[ItemStack] {
        &self.items
    }

    fn items_mut(&mut self) -> &mut [ItemStack] {
        &mut self.items
    }

    fn get_container_size(&self) -> usize {
        CHEST_SLOTS
    }

    fn set_item(&mut self, slot: usize, mut stack: ItemStack) {
        if slot < CHEST_SLOTS {
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
    use super::*;
    use steel_registry::{init_vanilla_registry, vanilla_blocks, vanilla_items};

    #[test]
    fn chest_holds_twenty_seven_slots() {
        init_vanilla_registry();
        let chest = ChestBlockEntity::new_chest(
            Weak::new(),
            BlockPos::new(1, 2, 3),
            vanilla_blocks::CHEST.default_state(),
        );
        assert_eq!(chest.container.lock().get_container_size(), CHEST_SLOTS);
        chest
            .container
            .lock()
            .set_item(26, ItemStack::with_count(&*vanilla_items::STONE, 100));
        assert_eq!(chest.container.lock().get_item(26).count(), 64);
    }
}
