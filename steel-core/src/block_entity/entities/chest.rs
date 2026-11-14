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
use steel_registry::data_components::{
    DataComponentMap, SeededContainerLoot, vanilla_components::CONTAINER_LOOT,
};
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_block_entity_types;
use steel_utils::{
    BlockPos, BlockStateId, DowncastType, DowncastTypeKey, locks::SyncMutex,
};

use crate::block_entity::randomizable_container::RandomizableContainer;
use crate::block_entity::{BlockEntity, BlockEntityBase, ImplicitComponentGetter};
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
    /// Loot state for chests placed by a structure template, which carry a
    /// `LootTable` instead of items.
    loot: SyncMutex<RandomizableContainer>,
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
            loot: SyncMutex::new(RandomizableContainer::none()),
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

    /// Vanilla `RandomizableContainerBlockEntity.unpackLootTable`: fills a
    /// structure chest the first time it is opened.
    ///
    /// Returns whether anything had to be rolled, so the caller can sync the
    /// new contents to whoever opened it.
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

impl BlockEntity for ChestBlockEntity {
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
        drop(container);

        // Vanilla `RandomizableContainerBlockEntity.loadAdditional`: a template
        // chest stores a `LootTable` (and a `LootTableSeed`) instead of items,
        // and is filled the first time a player opens it.
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
        drop(container);

        // Vanilla `RandomizableContainer.trySaveLootTable`: an unopened
        // structure container keeps its table, an unpacked one writes nothing.
        self.loot.lock().save_to_nbt(nbt);
    }

    fn get_update_tag(&self) -> Option<NbtCompound> {
        // Chests do not sync their contents on chunk load.
        None
    }

    fn container_ref(&self) -> Option<ContainerRef> {
        Some(self.container_ref.clone())
    }

    fn unpack_loot_table(&self) {
        self.unpack_loot();
    }

    /// Vanilla `RandomizableContainerBlockEntity.applyImplicitComponents`: the
    /// `minecraft:container_loot` component of the placed stack becomes the
    /// loot table, so a chest item with a table opens with loot.
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
    use steel_registry::data_components::DataComponentPatch;

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

    /// Builds the NBT a structure template stores for a loot chest.
    fn template_chest_nbt(table: &str, seed: i64) -> NbtCompound {
        let mut nbt = NbtCompound::new();
        nbt.insert("id", "minecraft:chest");
        nbt.insert("LootTable", table);
        nbt.insert("LootTableSeed", seed);
        nbt
    }

    #[test]
    fn a_structure_chest_keeps_its_table_until_it_is_opened() {
        init_vanilla_registry();
        let chest = ChestBlockEntity::new_chest(
            Weak::new(),
            BlockPos::new(1, 2, 3),
            vanilla_blocks::CHEST.default_state(),
        );
        // No world is attached, so the stored seed has to survive on its own.
        let nbt = template_chest_nbt("minecraft:chests/simple_dungeon", 424242);
        chest.load_with_owned_components(&nbt).expect("template nbt loads");

        // Vanilla stores the table back while nobody opened the container, so a
        // restart fills the chest instead of leaving it empty.
        let saved = chest.save_custom_only();
        assert_eq!(
            saved
                .string("LootTable")
                .map(|value| value.to_str().into_owned()),
            Some("minecraft:chests/simple_dungeon".to_owned())
        );
        assert_eq!(saved.long("LootTableSeed"), Some(424242));

        // Opening rolls the table into the slots and clears it.
        assert!(chest.unpack_loot(), "the table must roll into the chest");
        assert!(
            chest.container.lock().items.iter().any(|item| !item.is_empty()),
            "the chest must hold the rolled loot"
        );
        let saved = chest.save_custom_only();
        assert_eq!(
            saved.string("LootTable"),
            None,
            "the rolled table is not saved again"
        );
    }

    #[test]
    fn a_chest_item_with_container_loot_carries_the_table() {
        init_vanilla_registry();
        let chest = ChestBlockEntity::new_chest(
            Weak::new(),
            BlockPos::new(4, 5, 6),
            vanilla_blocks::CHEST.default_state(),
        );
        // Vanilla `RandomizableContainerBlockEntity.applyImplicitComponents`: the
        // component of the placed stack becomes the container's loot table.
        let mut patch = DataComponentPatch::new();
        patch.set(
            CONTAINER_LOOT,
            SeededContainerLoot::new(Identifier::vanilla_static("chests/simple_dungeon"), 2024),
        );
        let stack = ItemStack::with_count_and_patch(&*vanilla_items::CHEST, 1, patch);
        chest.apply_components_from_item_stack(&stack);

        assert!(chest.loot.lock().has_loot_table());
        assert_eq!(chest.loot.lock().loot_table_seed(), 2024);
        // The component is implicit, so it stays out of the stored map and the
        // collected map yields exactly one table.
        assert!(!chest.save_custom_only().contains("container_loot"));
        assert!(chest.collect_components().has(CONTAINER_LOOT));
        assert!(chest.unpack_loot());
        assert!(!chest.loot.lock().has_loot_table());
    }

    #[test]
    fn a_structure_chest_rolls_once_only() {
        init_vanilla_registry();
        let chest = ChestBlockEntity::new_chest(
            Weak::new(),
            BlockPos::new(1, 2, 3),
            vanilla_blocks::CHEST.default_state(),
        );
        chest.load_with_owned_components(&template_chest_nbt("minecraft:chests/simple_dungeon", 7))
            .expect("template nbt loads");
        assert!(chest.unpack_loot());
        let first: Vec<ItemStack> = chest.container.lock().items.clone();
        // Vanilla `unpackLootTable` returns early once the table is gone, so an
        // emptied chest stays empty instead of refilling on the next open.
        assert!(!chest.unpack_loot());
        assert_eq!(first, chest.container.lock().items);
    }
}
