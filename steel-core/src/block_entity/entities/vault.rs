//! Vault block entity.
//!
//! Vanilla `VaultBlockEntity` tracks the server-side reward state: who has
//! already looted it, the current loot table, the display item, and the
//! remaining reward count. The block state mirrors `vault_state`
//! (`inactive`, `active`, `unlocking`, `ejecting`).

use std::sync::{Arc, Weak};

use simdnbt::borrow::BaseNbtCompound as BorrowedNbtCompound;
use simdnbt::owned::NbtCompound;
use steel_registry::block_entity_type::BlockEntityTypeRef;
use steel_registry::vanilla_block_entity_types;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey, locks::SyncMutex};

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::world::World;

/// Vault server-side state persisted in the block entity.
#[derive(Debug, Default)]
struct VaultState {
    /// Loot table key; empty when the vault was configured without one.
    loot_table: String,
    /// Display item identifier shown inside the vault, if any.
    display_item: Option<String>,
    /// Reward count remaining before the vault is exhausted.
    total_players: i32,
}

/// Vault block entity.
pub struct VaultBlockEntity {
    base: Arc<BlockEntityBase>,
    state: SyncMutex<VaultState>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `VaultBlockEntity`.
unsafe impl DowncastType for VaultBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/vault");
}

impl VaultBlockEntity {
    /// Creates a new vault block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self::new_with_type(&vanilla_block_entity_types::VAULT, level, pos, state)
    }

    /// Creates a vault with the given block entity type (trial spawners reuse
    /// the same shared-data shape in vanilla, so the constructor is generic).
    fn new_with_type(
        block_entity_type: BlockEntityTypeRef,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> Self {
        Self {
            base: Arc::new(BlockEntityBase::new(block_entity_type, level, pos, state)),
            state: SyncMutex::new(VaultState::default()),
        }
    }

    /// Configured loot table key.
    #[must_use]
    pub fn loot_table(&self) -> String {
        self.state.lock().loot_table.clone()
    }

    /// Remaining reward count.
    #[must_use]
    pub fn total_players(&self) -> i32 {
        self.state.lock().total_players
    }
}

impl BlockEntity for VaultBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let view: simdnbt::borrow::NbtCompound<'_, '_> = nbt.into();
        let mut state = self.state.lock();
        state.loot_table = view.string("loot_table").to_str().to_string();
        let display = view.string("display_item").to_str();
        state.display_item = (!display.is_empty()).then(|| display.to_string());
        state.total_players = view.int("total_players").unwrap_or(0);
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        let state = self.state.lock();
        nbt.insert("loot_table", state.loot_table.as_str());
        if let Some(display) = &state.display_item {
            nbt.insert("display_item", display.as_str());
        }
        nbt.insert("total_players", state.total_players);
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_blocks};

    use super::*;

    #[test]
    fn vault_starts_unconfigured() {
        init_vanilla_registry();
        let vault = VaultBlockEntity::new(
            Weak::new(),
            BlockPos::new(0, 0, 0),
            vanilla_blocks::VAULT.default_state(),
        );
        assert!(vault.loot_table().is_empty());
        assert_eq!(vault.total_players(), 0);
    }
}
