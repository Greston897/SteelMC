//! Vanilla `OminousItemSpawner`.
//!
//! A trial-chamber helper that drops one item stack on a fixed schedule and then
//! removes itself. Vanilla stores only the pending item stack; the countdown
//! lives in NBT (`spawn_item_after_ticks`).

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_entity_data::OminousItemSpawnerEntityData;
use steel_utils::locks::SyncMutex;
use steel_utils::nbt::NbtValueInput as _;
use steel_utils::{DowncastType, DowncastTypeKey};

use crate::entity::{Entity, EntityBase, EntityBaseLoad, EntitySyncedData, RemovalReason};
use crate::world::World;

/// Vanilla `OminousItemSpawner` delay bounds (`SPAWN_ITEM_AFTER_TICKS_MIN/MAX`).
const SPAWN_ITEM_AFTER_TICKS_MIN: i32 = 40;
const SPAWN_ITEM_AFTER_TICKS_MAX: i32 = 120;

#[entity_behavior(class = "OminousItemSpawner")]
/// Vanilla ominous item spawner.
pub struct OminousItemSpawnerEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    entity_data: SyncMutex<OminousItemSpawnerEntityData>,
    state: SyncMutex<OminousItemSpawnerState>,
}

#[derive(Debug, Clone, Copy)]
struct OminousItemSpawnerState {
    /// Ticks left before the stored item is dropped.
    spawn_item_after_ticks: i32,
}

impl Default for OminousItemSpawnerState {
    fn default() -> Self {
        Self {
            spawn_item_after_ticks: SPAWN_ITEM_AFTER_TICKS_MIN,
        }
    }
}

// SAFETY: This key is owned by Steel and uniquely identifies `OminousItemSpawnerEntity`.
unsafe impl DowncastType for OminousItemSpawnerEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/ominous_item_spawner");
}

impl OminousItemSpawnerEntity {
    /// Creates a new ominous item spawner.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            entity_data: SyncMutex::new(OminousItemSpawnerEntityData::new()),
            state: SyncMutex::new(OminousItemSpawnerState::default()),
        }
    }

    /// Creates an ominous item spawner from saved data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            entity_data: SyncMutex::new(OminousItemSpawnerEntityData::new()),
            state: SyncMutex::new(OminousItemSpawnerState::default()),
        }
    }

    /// The item that will be dropped (vanilla `DATA_ITEM_STACK` / `item`).
    #[must_use]
    pub fn item(&self) -> ItemStack {
        self.entity_data.lock().item.get().clone()
    }

    /// Sets the pending item stack.
    pub fn set_item(&self, item: ItemStack) {
        self.entity_data.lock().item.set(item);
    }

    /// Ticks left until the item is dropped.
    #[must_use]
    pub fn spawn_item_after_ticks(&self) -> i32 {
        self.state.lock().spawn_item_after_ticks
    }

    /// Sets the remaining countdown, clamped to the vanilla window.
    pub fn set_spawn_item_after_ticks(&self, ticks: i32) {
        self.state.lock().spawn_item_after_ticks =
            ticks.clamp(SPAWN_ITEM_AFTER_TICKS_MIN, SPAWN_ITEM_AFTER_TICKS_MAX);
    }

    /// Vanilla `OminousItemSpawner.spawnItem`: drop the stored stack and vanish.
    fn spawn_item(self: &Arc<Self>) {
        let item = self.item();
        if !item.is_empty() {
            let _ = self.spawn_at_location(item, 0.0);
        }
        self.set_removed(RemovalReason::Discarded);
    }
}

impl Entity for OminousItemSpawnerEntity {
    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn synced_data(&self) -> Option<&dyn EntitySyncedData> {
        Some(&self.entity_data)
    }

    fn tick(self: Arc<Self>) {
        let remaining = {
            let mut state = self.state.lock();
            state.spawn_item_after_ticks -= 1;
            state.spawn_item_after_ticks
        };
        if remaining <= 0 {
            self.spawn_item();
        }
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        nbt.insert("spawn_item_after_ticks", self.spawn_item_after_ticks());
        if !self.item().is_empty() {
            nbt.insert("item", self.item().to_nbt_tag_ref());
        }
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        if let Some(ticks) = nbt.int_value("spawn_item_after_ticks") {
            self.state.lock().spawn_item_after_ticks = ticks;
        }
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_entities};

    use super::*;

    #[test]
    fn new_ominous_item_spawners_start_empty() {
        init_vanilla_registry();
        let spawner = OminousItemSpawnerEntity::new(
            &vanilla_entities::OMINOUS_ITEM_SPAWNER,
            1,
            DVec3::ZERO,
            Weak::new(),
        );
        assert!(spawner.item().is_empty());
        assert_eq!(spawner.spawn_item_after_ticks(), SPAWN_ITEM_AFTER_TICKS_MIN);
    }

    #[test]
    fn countdown_is_clamped_to_the_vanilla_window() {
        init_vanilla_registry();
        let spawner = OminousItemSpawnerEntity::new(
            &vanilla_entities::OMINOUS_ITEM_SPAWNER,
            1,
            DVec3::ZERO,
            Weak::new(),
        );
        spawner.set_spawn_item_after_ticks(0);
        assert_eq!(spawner.spawn_item_after_ticks(), SPAWN_ITEM_AFTER_TICKS_MIN);
        spawner.set_spawn_item_after_ticks(1000);
        assert_eq!(spawner.spawn_item_after_ticks(), SPAWN_ITEM_AFTER_TICKS_MAX);
    }
}
