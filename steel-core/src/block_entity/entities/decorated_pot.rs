//! Decorated pot block entity.
//!
//! Vanilla `DecoratedPotBlockEntity` stores the four sherd items shown on each
//! face (`back`, `left`, `right`, `front`) plus an optional loot table used when
//! the pot is generated as structure loot.

use std::sync::{Arc, Weak};

use simdnbt::ToNbtTag;
use simdnbt::borrow::{BaseNbtCompound as BorrowedNbtCompound, NbtCompound as NbtCompoundView};
use simdnbt::owned::{NbtCompound, NbtTag};
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_block_entity_types;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey, locks::SyncMutex};

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::world::World;

/// Number of faces on a decorated pot.
pub const DECORATED_POT_SIDES: usize = 4;

/// Decorated pot block entity.
pub struct DecoratedPotBlockEntity {
    base: Arc<BlockEntityBase>,
    /// One item per side, in vanilla order: back, left, right, front.
    items: SyncMutex<[ItemStack; DECORATED_POT_SIDES]>,
    /// Optional loot table applied on first interaction after world generation.
    loot_table: SyncMutex<Option<String>>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `DecoratedPotBlockEntity`.
unsafe impl DowncastType for DecoratedPotBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/decorated_pot");
}

impl DecoratedPotBlockEntity {
    /// Creates a new decorated pot block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self {
            base: Arc::new(BlockEntityBase::new(
                &vanilla_block_entity_types::DECORATED_POT,
                level,
                pos,
                state,
            )),
            items: SyncMutex::new(std::array::from_fn(|_| ItemStack::empty())),
            loot_table: SyncMutex::new(None),
        }
    }

    /// Returns the item on the given side.
    #[must_use]
    pub fn item_on_side(&self, side: usize) -> ItemStack {
        self.items.lock()[side.min(DECORATED_POT_SIDES - 1)].clone()
    }

    /// Sets the item on the given side.
    pub fn set_item_on_side(&self, side: usize, item: ItemStack) {
        if side < DECORATED_POT_SIDES {
            self.items.lock()[side] = item;
        }
    }

    /// Loot table key assigned at generation, if any.
    #[must_use]
    pub fn loot_table(&self) -> Option<String> {
        self.loot_table.lock().clone()
    }
}

impl BlockEntity for DecoratedPotBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn pre_remove_side_effects(&self, pos: BlockPos, _state: BlockStateId) {
        let items = {
            let mut guard = self.items.lock();
            std::mem::replace(&mut *guard, std::array::from_fn(|_| ItemStack::empty()))
        };
        let Some(world) = self.get_level() else {
            return;
        };
        for item in items {
            if !item.is_empty() {
                world.drop_item_stack(pos, item);
            }
        }
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let nbt_view: NbtCompoundView<'_, '_> = nbt.into();
        let mut items: [ItemStack; DECORATED_POT_SIDES] =
            std::array::from_fn(|_| ItemStack::empty());
        // Vanilla persists the sherds as one compound keyed by side name.
        if let Some(sherds) = nbt_view.compound("sherds") {
            for (index, key) in ["back", "left", "right", "front"].into_iter().enumerate() {
                if let Some(item_nbt) = sherds.compound(key)
                    && let Some(stack) = ItemStack::from_borrowed_compound(&item_nbt)
                {
                    items[index] = stack;
                }
            }
        }
        *self.items.lock() = items;
        *self.loot_table.lock() = nbt_view.string("LootTable").to_str().to_string().into();
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        let items = self.items.lock();
        let mut sherds = NbtCompound::new();
        for (index, key) in ["back", "left", "right", "front"].into_iter().enumerate() {
            let item = &items[index];
            if !item.is_empty()
                && let NbtTag::Compound(item_nbt) = item.clone().to_nbt_tag()
            {
                sherds.insert(key, item_nbt);
            }
        }
        drop(items);
        nbt.insert("sherds", sherds);
        if let Some(loot_table) = self.loot_table.lock().as_deref() {
            nbt.insert("LootTable", loot_table);
        }
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_blocks, vanilla_items};

    use super::*;

    #[test]
    fn decorated_pot_starts_empty() {
        init_vanilla_registry();
        let pot = DecoratedPotBlockEntity::new(
            Weak::new(),
            BlockPos::new(0, 0, 0),
            vanilla_blocks::DECORATED_POT.default_state(),
        );
        assert!(pot.item_on_side(0).is_empty());
    }

    #[test]
    fn set_sherd_roundtrips_through_side_accessor() {
        init_vanilla_registry();
        let pot = DecoratedPotBlockEntity::new(
            Weak::new(),
            BlockPos::new(0, 0, 0),
            vanilla_blocks::DECORATED_POT.default_state(),
        );
        pot.set_item_on_side(2, ItemStack::new(&*vanilla_items::BRICK));
        assert_eq!(pot.item_on_side(2).count(), 1);
    }
}
