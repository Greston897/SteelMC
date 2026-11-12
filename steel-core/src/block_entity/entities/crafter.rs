//! Crafter block entity.
//!
//! Vanilla `CrafterBlockEntity` holds a 3x3 grid (`SIZE = 9`), a slot-disable
//! mask, a trigger count used by the `triggered` block state, and a crafting
//! ticker that runs every game tick.

use std::sync::{Arc, Weak};

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

/// Number of slots in the crafter grid.
pub const CRAFTER_SLOTS: usize = 9;

/// Crafter block entity.
pub struct CrafterBlockEntity {
    base: Arc<BlockEntityBase>,
    container: Arc<SyncMutex<CrafterContainer>>,
    container_ref: ContainerRef,
    /// Bitset of disabled slots: bit `n` set means slot `n` is disabled.
    disabled_slots: SyncMutex<u16>,
    /// Triggered count, mirrored into the `triggered` block state.
    triggered: SyncMutex<i32>,
}

pub(crate) struct CrafterContainer {
    items: Vec<ItemStack>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `CrafterBlockEntity`.
unsafe impl DowncastType for CrafterBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/crafter");
}

// SAFETY: This key is owned by Steel and uniquely identifies the independently
// lockable inventory data used by a crafter.
unsafe impl DowncastType for CrafterContainer {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:container/crafter");
}

impl CrafterBlockEntity {
    /// Creates a new crafter block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        let base = Arc::new(BlockEntityBase::new(
            &vanilla_block_entity_types::CRAFTER,
            level,
            pos,
            state,
        ));
        let container = Arc::new(SyncMutex::new(CrafterContainer {
            items: vec![ItemStack::empty(); CRAFTER_SLOTS],
        }));
        let shared_container: SharedContainer =
            Arc::<SyncMutex<CrafterContainer>>::clone(&container);
        Self {
            container_ref: ContainerRef::owned_by_block_entity(shared_container, Arc::clone(&base)),
            base,
            container,
            disabled_slots: SyncMutex::new(0),
            triggered: SyncMutex::new(0),
        }
    }

    /// Returns the disabled-slot bitmask.
    #[must_use]
    pub fn disabled_slots(&self) -> u16 {
        *self.disabled_slots.lock()
    }

    /// Sets the disabled-slot bitmask.
    pub fn set_disabled_slots(&self, mask: u16) {
        *self.disabled_slots.lock() = mask;
    }

    /// Triggered counter mirrored into the block state.
    #[must_use]
    pub fn triggered(&self) -> i32 {
        *self.triggered.lock()
    }
}

impl BlockEntity for CrafterBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn pre_remove_side_effects(&self, pos: BlockPos, _state: BlockStateId) {
        let items = {
            let mut container = self.container.lock();
            std::mem::replace(
                &mut container.items,
                vec![ItemStack::empty(); CRAFTER_SLOTS],
            )
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
        {
            let mut container = self.container.lock();
            container.items.fill(ItemStack::empty());
            load_all_items(nbt_view, &mut container.items);
        }
        *self.disabled_slots.lock() = nbt_view.int("disabled_slots").unwrap_or(0) as u16;
        *self.triggered.lock() = nbt_view.int("triggered").unwrap_or(0);
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
        nbt.insert("disabled_slots", i32::from(*self.disabled_slots.lock()));
        nbt.insert("triggered", *self.triggered.lock());
    }

    fn container_ref(&self) -> Option<ContainerRef> {
        Some(self.container_ref.clone())
    }
}

impl Container for CrafterContainer {
    fn items(&self) -> &[ItemStack] {
        &self.items
    }

    fn items_mut(&mut self) -> &mut [ItemStack] {
        &mut self.items
    }

    fn get_container_size(&self) -> usize {
        CRAFTER_SLOTS
    }

    fn set_item(&mut self, slot: usize, mut stack: ItemStack) {
        if slot < CRAFTER_SLOTS {
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
    use steel_registry::{init_vanilla_registry, vanilla_blocks};

    use super::*;

    fn test_crafter() -> CrafterBlockEntity {
        init_vanilla_registry();
        CrafterBlockEntity::new(
            Weak::new(),
            BlockPos::new(0, 0, 0),
            vanilla_blocks::CRAFTER.default_state(),
        )
    }

    #[test]
    fn crafter_has_nine_slots() {
        assert_eq!(test_crafter().container.lock().items.len(), CRAFTER_SLOTS);
    }

    #[test]
    fn disabled_slot_mask_roundtrips() {
        let crafter = test_crafter();
        crafter.set_disabled_slots(0b101);
        assert_eq!(crafter.disabled_slots(), 0b101);
    }
}
