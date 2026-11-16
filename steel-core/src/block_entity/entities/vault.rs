//! Vault block entity (`VaultBlockEntity`).
//!
//! Tracks per-player reward redemption (`rewarded_players`), loot table, display
//! item, key item (`trial_key` vs `ominous_trial_key`), and state machine
//! transitions (`Inactive` <-> `Active` -> `Unlocking` -> `Ejecting`). Supports
//! both vanilla 26.3 nested `config`/`server_data` NBT and Steel's compact tags.

use std::sync::{Arc, Weak};

use glam::DVec3;
use rustc_hash::FxHashSet;
use simdnbt::borrow::BaseNbtCompound as BorrowedNbtCompound;
use simdnbt::owned::{NbtCompound, NbtList, NbtTag};
use steel_protocol::packets::game::SoundSource;
use steel_registry::block_entity_type::BlockEntityTypeRef;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::blocks::properties::{BlockStateProperties, VaultState as VaultBlockState};
use steel_registry::item_stack::ItemStack;
use steel_registry::{REGISTRY, sound_events, vanilla_block_entity_types, vanilla_items};
use steel_utils::types::UpdateFlags;
use steel_utils::{
    BlockPos, BlockStateId, DowncastType, DowncastTypeKey, Identifier, locks::SyncMutex,
};
use uuid::Uuid;

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::entity::Entity as _;
use crate::player::Player;
use crate::world::World;

const DEFAULT_VAULT_LOOT_TABLE: &str = "minecraft:chests/trial_chambers/reward";
const DEFAULT_OMINOUS_VAULT_LOOT_TABLE: &str = "minecraft:chests/trial_chambers/reward_ominous";
const VAULT_ACTIVATION_RANGE: f64 = 4.0;
const VAULT_EJECTION_TICKS: i32 = 20;

/// Outcome of a player attempting to unlock a vault with a held stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultUnlockOutcome {
    /// Held stack is not the right trial key for this vault.
    WrongKey,
    /// Player has already claimed rewards from this vault.
    AlreadyRewarded,
    /// Key accepted, reward ejected, and player recorded.
    Unlocked,
}

#[derive(Debug, Default)]
struct VaultData {
    loot_table: String,
    display_item: Option<String>,
    total_players: i32,
    rewarded_players: FxHashSet<u128>,
    ejection_ticks: i32,
}

/// Vault block entity.
pub struct VaultBlockEntity {
    base: Arc<BlockEntityBase>,
    state: SyncMutex<VaultData>,
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

    fn new_with_type(
        block_entity_type: BlockEntityTypeRef,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> Self {
        Self {
            base: Arc::new(BlockEntityBase::new(block_entity_type, level, pos, state)),
            state: SyncMutex::new(VaultData::default()),
        }
    }

    /// Configured loot table key.
    #[must_use]
    pub fn loot_table(&self) -> String {
        self.state.lock().loot_table.clone()
    }

    /// Remaining explicit reward count (for scripted/counted vaults).
    #[must_use]
    pub fn total_players(&self) -> i32 {
        self.state.lock().total_players
    }

    /// Whether `uuid` has already claimed this vault's reward.
    #[must_use]
    pub fn has_rewarded_player(&self, uuid: Uuid) -> bool {
        self.state.lock().rewarded_players.contains(&uuid.as_u128())
    }

    /// Returns whether `stack` matches this vault's required trial key.
    #[must_use]
    pub fn is_valid_key(block_state: BlockStateId, stack: &ItemStack) -> bool {
        let ominous = block_state
            .try_get_value(&BlockStateProperties::OMINOUS)
            .unwrap_or(false);
        if ominous {
            stack.is(&vanilla_items::OMINOUS_TRIAL_KEY)
        } else {
            stack.is(&vanilla_items::TRIAL_KEY)
        }
    }

    /// Attempts to unlock the vault for `player` using `key_stack` (vanilla
    /// `VaultBlockEntity.Server.tryInsertKey`). Does not shrink `key_stack`; the
    /// caller consumes one item on [`VaultUnlockOutcome::Unlocked`].
    pub fn try_insert_key(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
        block_state: BlockStateId,
        player: &Player,
        key_stack: &ItemStack,
    ) -> VaultUnlockOutcome {
        if !Self::is_valid_key(block_state, key_stack) {
            world.play_sound(
                &sound_events::BLOCK_VAULT_INSERT_ITEM_FAIL,
                SoundSource::Blocks,
                pos,
                1.0,
                1.0,
                None,
            );
            return VaultUnlockOutcome::WrongKey;
        }

        let player_id = player.uuid().as_u128();
        let ominous = block_state
            .try_get_value(&BlockStateProperties::OMINOUS)
            .unwrap_or(false);
        let loot_key = {
            let mut data = self.state.lock();
            if data.rewarded_players.contains(&player_id) {
                drop(data);
                world.play_sound(
                    &sound_events::BLOCK_VAULT_REJECT_REWARDED_PLAYER,
                    SoundSource::Blocks,
                    pos,
                    1.0,
                    1.0,
                    None,
                );
                return VaultUnlockOutcome::AlreadyRewarded;
            }
            data.rewarded_players.insert(player_id);
            data.ejection_ticks = VAULT_EJECTION_TICKS;
            if data.total_players > 0 {
                data.total_players -= 1;
            }
            if data.loot_table.is_empty() {
                if ominous {
                    DEFAULT_OMINOUS_VAULT_LOOT_TABLE.to_owned()
                } else {
                    DEFAULT_VAULT_LOOT_TABLE.to_owned()
                }
            } else {
                data.loot_table.clone()
            }
        };
        self.set_changed();

        if block_state
            .try_get_value(&BlockStateProperties::VAULT_STATE)
            .is_some()
        {
            world.set_block(
                pos,
                block_state.set_value(
                    &BlockStateProperties::VAULT_STATE,
                    VaultBlockState::Ejecting,
                ),
                UpdateFlags::UPDATE_ALL,
            );
        }
        world.play_sound(
            &sound_events::BLOCK_VAULT_INSERT_ITEM,
            SoundSource::Blocks,
            pos,
            1.0,
            1.0,
            None,
        );
        world.play_sound(
            &sound_events::BLOCK_VAULT_OPEN_SHUTTER,
            SoundSource::Blocks,
            pos,
            1.0,
            1.0,
            None,
        );

        let _ = Self::drop_loot_table(world, pos.above(), block_state, &loot_key);
        world.play_sound(
            &sound_events::BLOCK_VAULT_EJECT_ITEM,
            SoundSource::Blocks,
            pos,
            1.0,
            1.0,
            None,
        );
        VaultUnlockOutcome::Unlocked
    }

    /// Vanilla `VaultBlockEntity.Server.tryEjectReward` for direct callers.
    pub fn eject_reward(&self, world: &Arc<World>) -> usize {
        let key = {
            let mut state = self.state.lock();
            if state.total_players <= 0 || state.loot_table.is_empty() {
                return 0;
            }
            state.total_players -= 1;
            state.loot_table.clone()
        };
        Self::drop_loot_table(world, self.get_block_pos(), self.get_block_state(), &key)
    }

    fn drop_loot_table(
        world: &Arc<World>,
        drop_pos: BlockPos,
        block_state: BlockStateId,
        key: &str,
    ) -> usize {
        let identifier = match key.split_once(':') {
            Some((namespace, path)) => Identifier::new(namespace.to_owned(), path.to_owned()),
            None => Identifier::vanilla(key.to_owned()),
        };
        let Some(table) = REGISTRY.loot_tables.by_key(&identifier) else {
            return 0;
        };
        let mut random = rand::rng();
        let items = crate::behavior::block::drop_from_block_interact_loot_table(
            table,
            block_state,
            None,
            None,
            None,
            &mut random,
        );
        let dropped = items.len();
        for item in items {
            world.drop_item_stack(drop_pos, item);
        }
        dropped
    }

    fn has_unrewarded_nearby_player(&self, world: &World, pos: BlockPos) -> bool {
        let center = DVec3::new(
            f64::from(pos.x()) + 0.5,
            f64::from(pos.y()) + 0.5,
            f64::from(pos.z()) + 0.5,
        );
        let max_dist_sq = VAULT_ACTIVATION_RANGE * VAULT_ACTIVATION_RANGE;
        let rewarded = self.state.lock().rewarded_players.clone();
        let mut found = false;
        world.players.iter_players(|uuid, player| {
            if !player.is_spectator()
                && !rewarded.contains(&uuid.as_u128())
                && player.position().distance_squared(center) <= max_dist_sq
            {
                found = true;
                false
            } else {
                true
            }
        });
        found
    }
}

fn uuid_to_int_array(uuid: u128) -> Vec<i32> {
    uuid.to_be_bytes()
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| i32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

fn int_array_to_uuid(ints: &[i32]) -> Option<u128> {
    if ints.len() != 4 {
        return None;
    }
    let mut bytes = [0u8; 16];
    for (i, v) in ints.iter().enumerate() {
        bytes[i * 4..i * 4 + 4].copy_from_slice(&v.to_be_bytes());
    }
    Some(u128::from_be_bytes(bytes))
}

impl BlockEntity for VaultBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn tick(&self, world: &Arc<World>) {
        let pos = self.get_block_pos();
        let state = world.get_block_state(pos);
        let Some(current_phase) = state.try_get_value(&BlockStateProperties::VAULT_STATE) else {
            return;
        };

        {
            let mut data = self.state.lock();
            if data.ejection_ticks > 0 {
                data.ejection_ticks -= 1;
                if data.ejection_ticks > 0 {
                    return;
                }
            } else if world.game_time() % 20 != 0 {
                return;
            }
        }

        let desired = if self.has_unrewarded_nearby_player(world, pos) {
            VaultBlockState::Active
        } else {
            VaultBlockState::Inactive
        };
        if desired != current_phase {
            world.set_block(
                pos,
                state.set_value(&BlockStateProperties::VAULT_STATE, desired),
                UpdateFlags::UPDATE_ALL,
            );
            let sound = match (current_phase, desired) {
                (VaultBlockState::Ejecting, _) => &sound_events::BLOCK_VAULT_CLOSE_SHUTTER,
                (_, VaultBlockState::Active) => &sound_events::BLOCK_VAULT_ACTIVATE,
                _ => &sound_events::BLOCK_VAULT_DEACTIVATE,
            };
            world.play_sound(sound, SoundSource::Blocks, pos, 1.0, 1.0, None);
        }
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let view: simdnbt::borrow::NbtCompound<'_, '_> = nbt.into();
        let mut state = self.state.lock();
        state.loot_table = view
            .string("loot_table")
            .or_else(|| view.compound("config").and_then(|c| c.string("loot_table")))
            .map_or_default(|value| value.to_str().to_string());
        state.display_item = view
            .string("display_item")
            .map(|value| value.to_str().to_string())
            .filter(|display| !display.is_empty());
        state.total_players = view.int("total_players").unwrap_or(0);
        state.rewarded_players.clear();
        if let Some(server_data) = view.compound("server_data")
            && let Some(list) = server_data.list("rewarded_players")
            && let Some(arrays) = list.int_arrays()
        {
            for arr in arrays {
                let ints: Vec<i32> = arr.to_vec();
                if let Some(id) = int_array_to_uuid(&ints) {
                    state.rewarded_players.insert(id);
                }
            }
        }
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        let state = self.state.lock();
        nbt.insert("loot_table", state.loot_table.as_str());
        if let Some(display) = &state.display_item {
            nbt.insert("display_item", display.as_str());
        }
        nbt.insert("total_players", state.total_players);
        if !state.loot_table.is_empty() {
            let mut config = NbtCompound::new();
            config.insert("loot_table", state.loot_table.as_str());
            nbt.insert("config", config);
        }
        if !state.rewarded_players.is_empty() {
            let mut server_data = NbtCompound::new();
            let uuids: Vec<Vec<i32>> = state
                .rewarded_players
                .iter()
                .copied()
                .map(uuid_to_int_array)
                .collect();
            server_data.insert("rewarded_players", NbtTag::List(NbtList::IntArray(uuids)));
            nbt.insert("server_data", server_data);
        }
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
    fn vault_matches_normal_and_ominous_keys() {
        init_vanilla_registry();
        let normal = vanilla_blocks::VAULT.default_state();
        let ominous = normal.set_value(&BlockStateProperties::OMINOUS, true);
        let trial_key = ItemStack::new(&vanilla_items::TRIAL_KEY);
        let ominous_key = ItemStack::new(&vanilla_items::OMINOUS_TRIAL_KEY);

        assert!(VaultBlockEntity::is_valid_key(normal, &trial_key));
        assert!(!VaultBlockEntity::is_valid_key(normal, &ominous_key));
        assert!(VaultBlockEntity::is_valid_key(ominous, &ominous_key));
        assert!(!VaultBlockEntity::is_valid_key(ominous, &trial_key));
    }
}
