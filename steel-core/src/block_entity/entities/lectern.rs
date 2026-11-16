//! Lectern block entity.
//!
//! Vanilla `LecternBlockEntity` owns a single book slot, the current page, and a
//! comparator output that reports the reading progress through the book. The
//! block state mirrors whether a book is present through `has_book`.

use std::{
    mem,
    sync::{Arc, Weak},
};

use simdnbt::ToNbtTag;
use simdnbt::borrow::{BaseNbtCompound as BorrowedNbtCompound, NbtCompound as NbtCompoundView};
use simdnbt::owned::{NbtCompound, NbtList, NbtTag};
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_block_entity_types;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey, locks::SyncMutex};

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::inventory::container::{Container, load_all_items};
use crate::inventory::lock::{ContainerRef, SharedContainer};
use crate::world::World;

/// Number of slots in a lectern.
pub const LECTERN_SLOTS: usize = 1;

/// Lectern block entity.
pub struct LecternBlockEntity {
    base: Arc<BlockEntityBase>,
    container: Arc<SyncMutex<LecternContainer>>,
    container_ref: ContainerRef,
    page: SyncMutex<i32>,
}

pub(crate) struct LecternContainer {
    items: Vec<ItemStack>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `LecternBlockEntity`.
unsafe impl DowncastType for LecternBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/lectern");
}

// SAFETY: This key is owned by Steel and uniquely identifies the independently
// lockable inventory data used by a lectern.
unsafe impl DowncastType for LecternContainer {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:container/lectern");
}

impl LecternBlockEntity {
    /// Creates a new lectern block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        let base = Arc::new(BlockEntityBase::new(
            &vanilla_block_entity_types::LECTERN,
            level,
            pos,
            state,
        ));
        let container = Arc::new(SyncMutex::new(LecternContainer {
            items: vec![ItemStack::empty(); LECTERN_SLOTS],
        }));
        let shared_container: SharedContainer =
            Arc::<SyncMutex<LecternContainer>>::clone(&container);
        Self {
            container_ref: ContainerRef::owned_by_block_entity(shared_container, Arc::clone(&base)),
            base,
            container,
            page: SyncMutex::new(0),
        }
    }

    /// Current page, matching vanilla's `LecternBlockEntity.page`.
    #[must_use]
    pub fn page(&self) -> i32 {
        *self.page.lock()
    }

    /// Sets the current page.
    pub fn set_page(&self, page: i32) {
        *self.page.lock() = page;
    }

    /// Returns whether a book currently occupies the lectern.
    #[must_use]
    pub fn has_book(&self) -> bool {
        !self.container.lock().items[0].is_empty()
    }
}

impl BlockEntity for LecternBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn pre_remove_side_effects(&self, pos: BlockPos, _state: BlockStateId) {
        let item = {
            let mut container = self.container.lock();
            mem::replace(&mut container.items[0], ItemStack::empty())
        };
        let Some(world) = self.get_level() else {
            return;
        };
        if !item.is_empty() {
            world.drop_item_stack(pos, item);
        }
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let nbt_view: NbtCompoundView<'_, '_> = nbt.into();
        {
            let mut container = self.container.lock();
            container.items.fill(ItemStack::empty());
            load_all_items(nbt_view, &mut container.items);
        }
        *self.page.lock() = nbt_view.int("Page").unwrap_or(0);
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        {
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
        nbt.insert("Page", *self.page.lock());
    }

    fn container_ref(&self) -> Option<ContainerRef> {
        Some(self.container_ref.clone())
    }
}

impl Container for LecternContainer {
    fn items(&self) -> &[ItemStack] {
        &self.items
    }

    fn items_mut(&mut self) -> &mut [ItemStack] {
        &mut self.items
    }

    fn get_container_size(&self) -> usize {
        LECTERN_SLOTS
    }

    fn set_item(&mut self, slot: usize, stack: ItemStack) {
        if slot < LECTERN_SLOTS {
            self.items[slot] = stack;
        }
    }

    fn get_max_stack_size(&self) -> i32 {
        // A lectern holds exactly one book.
        1
    }

    fn set_changed(&mut self) {}
}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_blocks, vanilla_items};

    use super::*;

    #[test]
    fn lectern_holds_books_and_tracks_page() {
        init_vanilla_registry();
        let lectern = LecternBlockEntity::new(
            Weak::new(),
            BlockPos::new(0, 0, 0),
            vanilla_blocks::LECTERN.default_state(),
        );
        assert!(!lectern.has_book());
        lectern
            .container
            .lock()
            .set_item(0, ItemStack::new(&vanilla_items::WRITABLE_BOOK));
        assert!(lectern.has_book());
        lectern.set_page(3);
        assert_eq!(lectern.page(), 3);
    }
}
