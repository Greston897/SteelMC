//! Dispenser and dropper block entity.
//!
//! Vanilla uses one class, `DispenserBlockEntity`, for both blocks: nine slots
//! in a 3x3 grid. The block behavior decides whether a fired item is dispensed
//! (with its projectile behavior) or dropped as an item entity.

use std::{
    mem,
    sync::{Arc, Weak},
};

use simdnbt::ToNbtTag;
use simdnbt::borrow::{BaseNbtCompound as BorrowedNbtCompound, NbtCompound as NbtCompoundView};
use simdnbt::owned::{NbtCompound, NbtList, NbtTag};
use steel_registry::block_entity_type::BlockEntityTypeRef;
use steel_registry::item_stack::ItemStack;
use steel_registry::data_components::{
    DataComponentMap, SeededContainerLoot, vanilla_components::CONTAINER_LOOT,
};
use steel_registry::vanilla_block_entity_types;
use steel_utils::locks::SyncMutex;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey};

use crate::block_entity::randomizable_container::RandomizableContainer;
use crate::block_entity::{BlockEntity, BlockEntityBase, ImplicitComponentGetter};
use crate::inventory::container::{Container, load_all_items};
use crate::inventory::lock::{ContainerRef, SharedContainer};
use crate::world::World;

/// Number of slots in a dispenser or dropper.
pub const DISPENSER_SLOTS: usize = 9;

/// Dispenser and dropper block entity.
pub struct DispenserBlockEntity {
    base: Arc<BlockEntityBase>,
    container: Arc<SyncMutex<DispenserContainer>>,
    container_ref: ContainerRef,
    /// Selected slot index, used by the client and by comparator output.
    ///
    /// Vanilla syncs this value for droppers and dispensers because the client
    /// animates the item staying in the slot for a few ticks after firing.
    selected_slot: SyncMutex<i32>,
    /// Loot state for dispensers and droppers placed by a structure template.
    loot: SyncMutex<RandomizableContainer>,
}

pub(crate) struct DispenserContainer {
    items: Vec<ItemStack>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `DispenserBlockEntity`.
unsafe impl DowncastType for DispenserBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/dispenser");
}

// SAFETY: This key is owned by Steel and uniquely identifies the independently
// lockable inventory data used by a dispenser or dropper block entity.
unsafe impl DowncastType for DispenserContainer {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:container/dispenser");
}

impl DispenserBlockEntity {
    /// Creates a new dispenser block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self::new_with_type(&vanilla_block_entity_types::DISPENSER, level, pos, state)
    }

    /// Creates a new dropper block entity.
    #[must_use]
    pub fn new_dropper(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self::new_with_type(&vanilla_block_entity_types::DROPPER, level, pos, state)
    }

    fn new_with_type(
        block_entity_type: BlockEntityTypeRef,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> Self {
        let base = Arc::new(BlockEntityBase::new(block_entity_type, level, pos, state));
        let container = Arc::new(SyncMutex::new(DispenserContainer {
            items: vec![ItemStack::empty(); DISPENSER_SLOTS],
        }));
        let shared_container: SharedContainer =
            Arc::<SyncMutex<DispenserContainer>>::clone(&container);
        Self {
            container_ref: ContainerRef::owned_by_block_entity(shared_container, Arc::clone(&base)),
            base,
            container,
            selected_slot: SyncMutex::new(-1),
            loot: SyncMutex::new(RandomizableContainer::none()),
        }
    }

    /// Returns the currently animating slot, or `-1` when idle.
    #[must_use]
    pub fn selected_slot(&self) -> i32 {
        *self.selected_slot.lock()
    }

    /// Vanilla `RandomizableContainerBlockEntity.unpackLootTable`.
    ///
    /// Returns whether a pending table was rolled.
    pub fn unpack_loot(&self) -> bool {
        if !self.loot.lock().has_loot_table() {
            return false;
        }
        let world_seed = self.get_level().map_or(0, |world| world.seed());
        // Lock order matches the save path: container first, loot state second.
        let mut container = self.container.lock();
        let mut loot = self.loot.lock();
        loot.unpack_loot_table(&mut container.items, world_seed)
    }
}

impl BlockEntity for DispenserBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn pre_remove_side_effects(&self, pos: BlockPos, _state: BlockStateId) {
        // Vanilla `RandomizableContainerBlockEntity.playerWillDestroy` unpacks a
        // pending table into the container, so breaking a generated container
        // drops its loot instead of nothing.
        self.unpack_loot();
        let items = {
            let mut container = self.container.lock();
            mem::replace(
                &mut container.items,
                vec![ItemStack::empty(); DISPENSER_SLOTS],
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
        if let Some(slot) = nbt_view.int("SelectedSlot") {
            *self.selected_slot.lock() = slot;
        }

        // Vanilla `RandomizableContainerBlockEntity.loadAdditional`.
        RandomizableContainer::load_from_nbt(nbt_view, &self.loot);
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
        nbt.insert("SelectedSlot", *self.selected_slot.lock());
        drop(container);

        // Vanilla `RandomizableContainer.trySaveLootTable`: an unopened
        // structure container keeps its table, an unpacked one writes nothing.
        self.loot.lock().save_to_nbt(nbt);
    }

    fn container_ref(&self) -> Option<ContainerRef> {
        Some(self.container_ref.clone())
    }

    fn unpack_loot_table(&self) {
        self.unpack_loot();
    }

    /// Vanilla `RandomizableContainerBlockEntity.applyImplicitComponents`.
    fn apply_implicit_components(&self, components: &mut ImplicitComponentGetter<'_>) {
        if let Some(loot) = components.get(CONTAINER_LOOT) {
            self.loot
                .lock()
                .set_loot_table(Some(loot.loot_table().clone()), loot.seed());
        }
    }

    /// Vanilla `RandomizableContainerBlockEntity.collectImplicitComponents`.
    fn collect_implicit_components(&self, components: &mut DataComponentMap) {
        let loot = self.loot.lock();
        if let Some(table) = loot.loot_table() {
            components.set(
                CONTAINER_LOOT,
                Some(SeededContainerLoot::new(
                    table.clone(),
                    loot.loot_table_seed(),
                )),
            );
        }
    }
}

impl Container for DispenserContainer {
    fn items(&self) -> &[ItemStack] {
        &self.items
    }

    fn items_mut(&mut self) -> &mut [ItemStack] {
        &mut self.items
    }

    fn get_container_size(&self) -> usize {
        DISPENSER_SLOTS
    }

    fn set_item(&mut self, slot: usize, mut stack: ItemStack) {
        if slot < DISPENSER_SLOTS {
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

    fn test_dispenser() -> DispenserBlockEntity {
        init_vanilla_registry();
        DispenserBlockEntity::new(
            Weak::new(),
            BlockPos::new(1, 2, 3),
            vanilla_blocks::DISPENSER.default_state(),
        )
    }

    #[test]
    fn dispenser_has_nine_slots() {
        let dispenser = test_dispenser();
        assert_eq!(dispenser.container.lock().items.len(), 9);
    }

    #[test]
    fn selected_slot_defaults_to_negative_one() {
        let dispenser = test_dispenser();
        assert_eq!(dispenser.selected_slot(), -1);
    }
}
