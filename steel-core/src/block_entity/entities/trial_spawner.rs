//! Trial spawner block entity.
//!
//! Vanilla `TrialSpawnerBlockEntity` stores the spawner configuration mirrored
//! from the block entity: the entity type to spawn, the number of mobs per
//! player, the total mob count, and the reward loot table given on completion.

use std::sync::{Arc, Weak};

use simdnbt::borrow::BaseNbtCompound as BorrowedNbtCompound;
use simdnbt::owned::NbtCompound;
use steel_registry::vanilla_block_entity_types;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey, locks::SyncMutex};

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::world::World;

/// Trial spawner configuration persisted in the block entity.
#[derive(Debug, Default)]
struct TrialSpawnerState {
    /// Entity type spawned by this spawner, as a registry key.
    entity: String,
    /// Mobs spawned per player present.
    spawns_per_player: i32,
    /// Total mobs this spawner must spawn before it can be completed.
    total_mobs: i32,
    /// Loot table granted on completion.
    loot_table: String,
}

/// Trial spawner block entity.
pub struct TrialSpawnerBlockEntity {
    base: Arc<BlockEntityBase>,
    state: SyncMutex<TrialSpawnerState>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `TrialSpawnerBlockEntity`.
unsafe impl DowncastType for TrialSpawnerBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/trial_spawner");
}

impl TrialSpawnerBlockEntity {
    /// Creates a new trial spawner block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self {
            base: Arc::new(BlockEntityBase::new(
                &vanilla_block_entity_types::TRIAL_SPAWNER,
                level,
                pos,
                state,
            )),
            state: SyncMutex::new(TrialSpawnerState::default()),
        }
    }

    /// Entity type key this spawner spawns.
    #[must_use]
    pub fn entity(&self) -> String {
        self.state.lock().entity.clone()
    }

    /// Total mobs required for completion.
    #[must_use]
    pub fn total_mobs(&self) -> i32 {
        self.state.lock().total_mobs
    }
}

impl BlockEntity for TrialSpawnerBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let view: simdnbt::borrow::NbtCompound<'_, '_> = nbt.into();
        let mut state = self.state.lock();
        state.entity = view.string("entity").to_str().to_string();
        state.spawns_per_player = view.int("spawns_per_player").unwrap_or(0);
        state.total_mobs = view.int("total_mobs").unwrap_or(0);
        state.loot_table = view.string("loot_table").to_str().to_string();
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        let state = self.state.lock();
        nbt.insert("entity", state.entity.as_str());
        nbt.insert("spawns_per_player", state.spawns_per_player);
        nbt.insert("total_mobs", state.total_mobs);
        nbt.insert("loot_table", state.loot_table.as_str());
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_blocks};

    use super::*;

    #[test]
    fn trial_spawner_starts_unconfigured() {
        init_vanilla_registry();
        let spawner = TrialSpawnerBlockEntity::new(
            Weak::new(),
            BlockPos::new(0, 0, 0),
            vanilla_blocks::TRIAL_SPAWNER.default_state(),
        );
        assert!(spawner.entity().is_empty());
        assert_eq!(spawner.total_mobs(), 0);
    }
}
