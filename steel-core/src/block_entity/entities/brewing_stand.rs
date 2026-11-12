//! Brewing stand block entity.
//!
//! Vanilla `BrewingStandBlockEntity` owns four slots: three potion slots
//! (`0..=2`) and a fuel slot (`3`). Brewing costs 20 ticks per potion and one
//! blaze powder per operation; each powder fuels 20 operations.

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

/// Number of slots in a brewing stand (three potions plus fuel).
pub const BREWING_STAND_SLOTS: usize = 4;

/// Index of the fuel slot, matching vanilla's `BrewingStandBlockEntity.FUEL_SLOT`.
pub const FUEL_SLOT: usize = 3;

/// Ticks one brew takes in vanilla (`BREWING_TIME = 400` ticks is the whole
/// fuel burn; one operation is 20 ticks).
pub const BREW_TIME: i32 = 20;

/// Brewing stand state persisted alongside the inventory.
struct BrewingState {
    /// Remaining ticks of the current brew, counting down to zero.
    brew_time: i32,
    /// Remaining fuel units; one unit is one brew operation.
    fuel: i32,
}

/// Brewing stand block entity.
pub struct BrewingStandBlockEntity {
    base: Arc<BlockEntityBase>,
    container: Arc<SyncMutex<BrewingStandContainer>>,
    container_ref: ContainerRef,
    state: SyncMutex<BrewingState>,
}

pub(crate) struct BrewingStandContainer {
    items: Vec<ItemStack>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `BrewingStandBlockEntity`.
unsafe impl DowncastType for BrewingStandBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/brewing_stand");
}

// SAFETY: This key is owned by Steel and uniquely identifies the independently
// lockable inventory data used by a brewing stand.
unsafe impl DowncastType for BrewingStandContainer {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:container/brewing_stand");
}

impl BrewingStandBlockEntity {
    /// Creates a new brewing stand block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        let base = Arc::new(BlockEntityBase::new(
            &vanilla_block_entity_types::BREWING_STAND,
            level,
            pos,
            state,
        ));
        let container = Arc::new(SyncMutex::new(BrewingStandContainer {
            items: vec![ItemStack::empty(); BREWING_STAND_SLOTS],
        }));
        let shared_container: SharedContainer =
            Arc::<SyncMutex<BrewingStandContainer>>::clone(&container);
        Self {
            container_ref: ContainerRef::owned_by_block_entity(shared_container, Arc::clone(&base)),
            base,
            container,
            state: SyncMutex::new(BrewingState {
                brew_time: 0,
                fuel: 0,
            }),
        }
    }

    /// Remaining fuel units, matching vanilla's `BrewingStandBlockEntity.fuel`.
    #[must_use]
    pub fn fuel(&self) -> i32 {
        self.state.lock().fuel
    }

    /// Remaining ticks of the current brew.
    #[must_use]
    pub fn brew_time(&self) -> i32 {
        self.state.lock().brew_time
    }
}

impl BlockEntity for BrewingStandBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn pre_remove_side_effects(&self, pos: BlockPos, _state: BlockStateId) {
        let items = {
            let mut container = self.container.lock();
            mem::replace(
                &mut container.items,
                vec![ItemStack::empty(); BREWING_STAND_SLOTS],
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
        let mut state = self.state.lock();
        state.brew_time = nbt_view.int("BrewTime").unwrap_or(0);
        state.fuel = nbt_view.int("Fuel").unwrap_or(0);
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
        let state = self.state.lock();
        nbt.insert("BrewTime", state.brew_time);
        nbt.insert("Fuel", state.fuel);
    }

    fn container_ref(&self) -> Option<ContainerRef> {
        Some(self.container_ref.clone())
    }
}

impl Container for BrewingStandContainer {
    fn items(&self) -> &[ItemStack] {
        &self.items
    }

    fn items_mut(&mut self) -> &mut [ItemStack] {
        &mut self.items
    }

    fn get_container_size(&self) -> usize {
        BREWING_STAND_SLOTS
    }

    fn set_item(&mut self, slot: usize, mut stack: ItemStack) {
        if slot < BREWING_STAND_SLOTS {
            let max_stack_size = self.get_max_stack_size_for_item(&stack);
            if !stack.is_empty() && stack.count() > max_stack_size {
                stack.set_count(max_stack_size);
            }
            self.items[slot] = stack;
        }
    }

    fn get_max_stack_size(&self) -> i32 {
        // Vanilla restricts the fuel slot to one item and potion slots to one
        // as well, so every slot in a brewing stand behaves as a single slot.
        1
    }

    fn set_changed(&mut self) {}
}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_blocks, vanilla_items};

    use super::*;

    fn test_brewing_stand() -> BrewingStandBlockEntity {
        init_vanilla_registry();
        BrewingStandBlockEntity::new(
            Weak::new(),
            BlockPos::new(1, 2, 3),
            vanilla_blocks::BREWING_STAND.default_state(),
        )
    }

    #[test]
    fn brewing_stand_has_four_slots() {
        let brewing_stand = test_brewing_stand();
        assert_eq!(brewing_stand.container.lock().items.len(), BREWING_STAND_SLOTS);
    }

    #[test]
    fn fuel_slot_holds_one_item() {
        let brewing_stand = test_brewing_stand();
        brewing_stand
            .container
            .lock()
            .set_item(FUEL_SLOT, ItemStack::with_count(&vanilla_items::BLAZE_POWDER, 5));
        assert_eq!(brewing_stand.container.lock().get_item(FUEL_SLOT).count(), 1);
    }

    #[test]
    fn state_defaults_to_idle() {
        let brewing_stand = test_brewing_stand();
        assert_eq!(brewing_stand.brew_time(), 0);
        assert_eq!(brewing_stand.fuel(), 0);
    }
}
