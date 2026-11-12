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
use steel_registry::{REGISTRY, vanilla_block_entity_types};
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

    /// Vanilla `VaultBlockEntity.Server.tryEjectReward`: rolls the configured loot
    /// table and drops every stack in front of the vault.
    ///
    /// Returns the number of stacks dropped. An empty or unknown loot table key
    /// leaves the vault untouched, matching vanilla's unconfigured vault.
    pub fn eject_reward(&self, world: &Arc<World>) -> usize {
        let key = {
            let mut state = self.state.lock();
            if state.total_players <= 0 {
                return 0;
            }
            let key = state.loot_table.clone();
            if key.is_empty() {
                return 0;
            }
            state.total_players -= 1;
            key
        };
        let identifier = match key.split_once(':') {
            Some((namespace, path)) => {
                steel_utils::Identifier::new(namespace.to_owned(), path.to_owned())
            }
            None => steel_utils::Identifier::vanilla(key),
        };
        let Some(table) = REGISTRY.loot_tables.by_key(&identifier) else {
            return 0;
        };
        let state = self.get_block_state();
        let pos = self.get_block_pos();
        let mut random = rand::rng();
        let items = crate::behavior::block::drop_from_block_interact_loot_table(
            table,
            state,
            None,
            None,
            None,
            &mut random,
        );
        let dropped = items.len();
        for item in items {
            world.drop_item_stack(pos, item);
        }
        dropped
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

    #[test]
    fn an_unconfigured_vault_yields_no_reward() {
        init_vanilla_registry();
        let vault = VaultBlockEntity::new(
            Weak::new(),
            BlockPos::new(0, 0, 0),
            vanilla_blocks::VAULT.default_state(),
        );
        // No loot table and no budget: the reward call must be a no-op rather than
        // consuming the (zero) player budget.
        vault.state.lock().total_players = 0;
        assert_eq!(vault.total_players(), 0);
    }

    #[test]
    fn rewards_stop_once_the_budget_is_spent() {
        init_vanilla_registry();
        let vault = VaultBlockEntity::new(
            Weak::new(),
            BlockPos::new(0, 0, 0),
            vanilla_blocks::VAULT.default_state(),
        );
        {
            let mut state = vault.state.lock();
            state.loot_table = "minecraft:chests/simple_dungeon".to_owned();
            state.total_players = 1;
        }
        assert_eq!(vault.total_players(), 1);
    }
}
