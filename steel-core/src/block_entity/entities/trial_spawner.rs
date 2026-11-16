//! Trial spawner block entity (`TrialSpawnerBlockEntity`).
//!
//! Implements the vanilla MC 26.3 Trial Spawner state machine:
//! `WaitingForPlayers -> Active -> WaitingForRewardEjection -> EjectingReward -> Cooldown (36,000 ticks)`,
//! including Bad Omen / Trial Omen conversion into an Ominous Trial Spawner,
//! wave mob spawning scaled by nearby survival players, live mob tracking,
//! reward loot ejection, and round-trip NBT persistence.

use std::sync::{Arc, Weak};

use glam::DVec3;
use rand::RngExt as _;
use simdnbt::borrow::BaseNbtCompound as BorrowedNbtCompound;
use simdnbt::owned::NbtCompound;
use steel_protocol::packets::game::SoundSource;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::blocks::properties::{BlockStateProperties, TrialSpawnerState as SpawnerPhase};
use steel_registry::{
    REGISTRY, RegistryExt as _, sound_events, vanilla_block_entity_types, vanilla_game_rules,
    vanilla_items, vanilla_mob_effects,
};
use steel_utils::types::{Difficulty, UpdateFlags};
use steel_utils::{
    BlockPos, BlockStateId, DowncastType, DowncastTypeKey, Identifier, locks::SyncMutex,
};
use uuid::Uuid;

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::entity::{ENTITIES, Entity as _, LivingEntity as _, MobEffectInstance, next_entity_id};
use crate::world::World;

const PLAYER_DETECT_RANGE: f64 = 14.0;
const SPAWN_RANGE: i32 = 4;
const DEFAULT_TOTAL_MOBS: i32 = 6;
const DEFAULT_MOBS_PER_PLAYER: i32 = 2;
const DEFAULT_SIMULTANEOUS_MOBS: i32 = 2;
const TICKS_BETWEEN_SPAWNS: i32 = 40;
const REWARD_DELAY_TICKS: i32 = 40;
/// Vanilla `TrialSpawner.targetCooldownLength`: 30 minutes (`36_000` ticks).
const TARGET_COOLDOWN_TICKS: i64 = 36_000;
const TRIAL_OMEN_TICKS_PER_LEVEL: i32 = 18_000;

#[derive(Debug, Default)]
struct TrialSpawnerData {
    entity: String,
    spawns_per_player: i32,
    total_mobs: i32,
    loot_table: String,
    total_mobs_spawned: i32,
    tracked_players: usize,
    current_mobs: Vec<Uuid>,
    next_mob_spawns_at: i64,
    cooldown_ends_at: i64,
}

/// Trial spawner block entity.
pub struct TrialSpawnerBlockEntity {
    base: Arc<BlockEntityBase>,
    state: SyncMutex<TrialSpawnerData>,
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
            state: SyncMutex::new(TrialSpawnerData::default()),
        }
    }

    /// Entity type key this spawner spawns.
    #[must_use]
    pub fn entity(&self) -> String {
        self.state.lock().entity.clone()
    }

    /// Total mobs configured for completion.
    #[must_use]
    pub fn total_mobs(&self) -> i32 {
        self.state.lock().total_mobs
    }

    /// Rolls the configured reward loot table (or drops a trial key when
    /// activated with default Trial Chambers rewards) above the spawner.
    pub fn eject_reward(&self, world: &Arc<World>) -> usize {
        let key = self.state.lock().loot_table.clone();
        if key.is_empty() {
            return 0;
        }
        Self::drop_loot_key(
            world,
            self.get_block_pos().above(),
            self.get_block_state(),
            &key,
        )
    }

    fn drop_loot_key(world: &Arc<World>, pos: BlockPos, state: BlockStateId, key: &str) -> usize {
        let identifier = parse_id(key);
        let Some(table) = REGISTRY.loot_tables.by_key(&identifier) else {
            return 0;
        };
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

    fn detect_survival_players(world: &World, pos: BlockPos) -> (usize, bool) {
        let center = DVec3::new(
            f64::from(pos.x()) + 0.5,
            f64::from(pos.y()) + 0.5,
            f64::from(pos.z()) + 0.5,
        );
        let max_sq = PLAYER_DETECT_RANGE * PLAYER_DETECT_RANGE;
        let mut count = 0usize;
        let mut became_ominous = false;
        world.players.iter_players(|_uuid, player| {
            if player.is_spectator()
                || player.has_infinite_materials()
                || !crate::entity::LivingEntity::is_alive(player.as_ref())
                || player.position().distance_squared(center) > max_sq
            {
                return true;
            }
            count += 1;
            if let Some(bad_omen) = player.mob_effect(vanilla_mob_effects::BAD_OMEN) {
                let level = (bad_omen.amplifier() + 1).max(1);
                let _ = player.remove_mob_effect(vanilla_mob_effects::BAD_OMEN);
                player.add_mob_effect(MobEffectInstance::with_duration(
                    vanilla_mob_effects::TRIAL_OMEN,
                    level.saturating_mul(TRIAL_OMEN_TICKS_PER_LEVEL),
                    0,
                ));
                became_ominous = true;
            } else if player.has_mob_effect(vanilla_mob_effects::TRIAL_OMEN) {
                became_ominous = true;
            }
            true
        });
        (count, became_ominous)
    }

    fn try_spawn_one(world: &Arc<World>, pos: BlockPos, entity_key: &str) -> Option<Uuid> {
        let id = parse_id(entity_key);
        let entity_type = REGISTRY.entity_types.by_key(&id)?;
        let mut rng = rand::rng();
        for _ in 0..8 {
            let dx = rng.random_range(-SPAWN_RANGE..=SPAWN_RANGE);
            let dy = rng.random_range(-1..=1);
            let dz = rng.random_range(-SPAWN_RANGE..=SPAWN_RANGE);
            let candidate = pos.offset(dx, dy, dz);
            if !world.get_block_state(candidate).is_air() {
                continue;
            }
            let spawn_pos = DVec3::new(
                f64::from(candidate.x()) + 0.5,
                f64::from(candidate.y()),
                f64::from(candidate.z()) + 0.5,
            );
            let entity = ENTITIES.create(
                entity_type,
                next_entity_id(),
                spawn_pos,
                Arc::downgrade(world),
            )?;
            let uuid = entity.uuid();
            if world.try_add_entity(entity).is_ok() {
                world.play_sound(
                    &sound_events::BLOCK_TRIAL_SPAWNER_SPAWN_MOB,
                    SoundSource::Blocks,
                    candidate,
                    1.0,
                    1.0,
                    None,
                );
                return Some(uuid);
            }
        }
        None
    }
}

fn parse_id(raw: &str) -> Identifier {
    match raw.split_once(':') {
        Some((ns, path)) => Identifier::new(ns.to_owned(), path.to_owned()),
        None => Identifier::vanilla(raw.to_owned()),
    }
}

fn set_spawner_state(
    world: &Arc<World>,
    pos: BlockPos,
    state: BlockStateId,
    phase: SpawnerPhase,
    ominous: bool,
) {
    let next = state
        .set_value(&BlockStateProperties::TRIAL_SPAWNER_STATE, phase)
        .set_value(&BlockStateProperties::OMINOUS, ominous);
    if next != state {
        world.set_block(pos, next, UpdateFlags::UPDATE_ALL);
    }
}

impl BlockEntity for TrialSpawnerBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn tick(&self, world: &Arc<World>) {
        let pos = self.get_block_pos();
        let state = world.get_block_state(pos);
        let Some(phase) = state.try_get_value(&BlockStateProperties::TRIAL_SPAWNER_STATE) else {
            return;
        };
        let mut ominous = state
            .try_get_value(&BlockStateProperties::OMINOUS)
            .unwrap_or(false);
        let now = world.game_time();

        let can_spawn = world.difficulty() != Difficulty::Peaceful
            && world.get_game_rule(&vanilla_game_rules::SPAWN_MOBS);

        match phase {
            SpawnerPhase::Inactive => {
                let configured = !self.state.lock().entity.is_empty();
                if configured && can_spawn {
                    set_spawner_state(world, pos, state, SpawnerPhase::WaitingForPlayers, ominous);
                }
            }
            SpawnerPhase::WaitingForPlayers => {
                if now % 20 != 0 {
                    return;
                }
                if !can_spawn || self.state.lock().entity.is_empty() {
                    set_spawner_state(world, pos, state, SpawnerPhase::Inactive, ominous);
                    return;
                }
                let (players, make_ominous) = Self::detect_survival_players(world, pos);
                if players > 0 {
                    if make_ominous && !ominous {
                        ominous = true;
                        world.play_sound(
                            &sound_events::BLOCK_TRIAL_SPAWNER_OMINOUS_ACTIVATE,
                            SoundSource::Blocks,
                            pos,
                            1.0,
                            1.0,
                            None,
                        );
                    } else {
                        world.play_sound(
                            &sound_events::BLOCK_TRIAL_SPAWNER_DETECT_PLAYER,
                            SoundSource::Blocks,
                            pos,
                            1.0,
                            1.0,
                            None,
                        );
                    }
                    {
                        let mut data = self.state.lock();
                        data.tracked_players = players;
                        data.total_mobs_spawned = 0;
                        data.current_mobs.clear();
                        data.next_mob_spawns_at = now + 20;
                    }
                    self.set_changed();
                    set_spawner_state(world, pos, state, SpawnerPhase::Active, ominous);
                }
            }
            SpawnerPhase::Active => {
                if !can_spawn {
                    set_spawner_state(world, pos, state, SpawnerPhase::Inactive, false);
                    return;
                }
                if now % 20 == 0 {
                    let (players, make_ominous) = Self::detect_survival_players(world, pos);
                    let mut data = self.state.lock();
                    if players > data.tracked_players {
                        data.tracked_players = players;
                    }
                    if make_ominous && !ominous {
                        data.total_mobs_spawned = 0;
                        drop(data);
                        set_spawner_state(world, pos, state, SpawnerPhase::Active, true);
                        world.play_sound(
                            &sound_events::BLOCK_TRIAL_SPAWNER_OMINOUS_ACTIVATE,
                            SoundSource::Blocks,
                            pos,
                            1.0,
                            1.0,
                            None,
                        );
                        return;
                    }
                }

                let spawn_request = {
                    let mut data = self.state.lock();
                    data.current_mobs.retain(|uuid| {
                        world
                            .get_entity_by_uuid(uuid)
                            .is_some_and(|entity| entity.is_alive())
                    });
                    let base_total = if data.total_mobs > 0 {
                        data.total_mobs
                    } else {
                        DEFAULT_TOTAL_MOBS
                    };
                    let per_player = if data.spawns_per_player > 0 {
                        data.spawns_per_player
                    } else {
                        DEFAULT_MOBS_PER_PLAYER
                    };
                    let extra = data.tracked_players.saturating_sub(1) as i32;
                    let target_total = base_total.saturating_add(extra.saturating_mul(per_player));
                    let max_simultaneous = (DEFAULT_SIMULTANEOUS_MOBS + extra).max(1) as usize;

                    if data.total_mobs_spawned >= target_total && data.current_mobs.is_empty() {
                        data.cooldown_ends_at = now + i64::from(REWARD_DELAY_TICKS);
                        drop(data);
                        self.set_changed();
                        set_spawner_state(
                            world,
                            pos,
                            state,
                            SpawnerPhase::WaitingForRewardEjection,
                            ominous,
                        );
                        return;
                    }

                    if data.total_mobs_spawned < target_total
                        && data.current_mobs.len() < max_simultaneous
                        && now >= data.next_mob_spawns_at
                        && !data.entity.is_empty()
                    {
                        data.next_mob_spawns_at = now + i64::from(TICKS_BETWEEN_SPAWNS);
                        Some(data.entity.clone())
                    } else {
                        None
                    }
                };

                if let Some(entity_key) = spawn_request
                    && let Some(uuid) = Self::try_spawn_one(world, pos, &entity_key)
                {
                    let mut data = self.state.lock();
                    data.current_mobs.push(uuid);
                    data.total_mobs_spawned += 1;
                    drop(data);
                    self.set_changed();
                }
            }
            SpawnerPhase::WaitingForRewardEjection => {
                if now >= self.state.lock().cooldown_ends_at {
                    set_spawner_state(world, pos, state, SpawnerPhase::EjectingReward, ominous);
                }
            }
            SpawnerPhase::EjectingReward => {
                let (loot_table, players) = {
                    let mut data = self.state.lock();
                    let players = data.tracked_players.max(1);
                    data.cooldown_ends_at = now + TARGET_COOLDOWN_TICKS;
                    data.total_mobs_spawned = 0;
                    data.tracked_players = 0;
                    data.current_mobs.clear();
                    (data.loot_table.clone(), players)
                };
                self.set_changed();

                for _ in 0..players {
                    if loot_table.is_empty() {
                        // Vanilla trial spawners eject trial keys on completion (30% ominous key
                        // when ominous, 50% trial key when normal; guarantee a trial key fallback).
                        let key_item = if ominous {
                            &vanilla_items::OMINOUS_TRIAL_KEY
                        } else {
                            &vanilla_items::TRIAL_KEY
                        };
                        world.drop_item_stack(
                            pos.above(),
                            steel_registry::item_stack::ItemStack::new(key_item),
                        );
                    } else {
                        let _ = Self::drop_loot_key(world, pos.above(), state, &loot_table);
                    }
                }
                world.play_sound(
                    &sound_events::BLOCK_TRIAL_SPAWNER_EJECT_ITEM,
                    SoundSource::Blocks,
                    pos,
                    1.0,
                    1.0,
                    None,
                );
                set_spawner_state(world, pos, state, SpawnerPhase::Cooldown, false);
            }
            SpawnerPhase::Cooldown => {
                if now >= self.state.lock().cooldown_ends_at {
                    set_spawner_state(world, pos, state, SpawnerPhase::WaitingForPlayers, false);
                }
            }
        }
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let view: simdnbt::borrow::NbtCompound<'_, '_> = nbt.into();
        let mut state = self.state.lock();
        let nested_entity = view
            .compound("spawn_data")
            .and_then(|sd| sd.compound("entity"))
            .and_then(|e| e.string("id"));
        state.entity = view
            .string("entity")
            .or(nested_entity)
            .map_or_default(|value| value.to_str().to_string());
        state.spawns_per_player = view.int("spawns_per_player").unwrap_or(0);
        state.total_mobs = view.int("total_mobs").unwrap_or(0);
        state.loot_table = view
            .string("loot_table")
            .map_or_default(|value| value.to_str().to_string());
        state.total_mobs_spawned = view.int("total_mobs_spawned").unwrap_or(0);
        state.next_mob_spawns_at = view.long("next_mob_spawns_at").unwrap_or(0);
        state.cooldown_ends_at = view.long("cooldown_ends_at").unwrap_or(0);
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        let state = self.state.lock();
        nbt.insert("entity", state.entity.as_str());
        nbt.insert("spawns_per_player", state.spawns_per_player);
        nbt.insert("total_mobs", state.total_mobs);
        nbt.insert("loot_table", state.loot_table.as_str());
        if state.total_mobs_spawned > 0 {
            nbt.insert("total_mobs_spawned", state.total_mobs_spawned);
        }
        if state.next_mob_spawns_at > 0 {
            nbt.insert("next_mob_spawns_at", state.next_mob_spawns_at);
        }
        if state.cooldown_ends_at > 0 {
            nbt.insert("cooldown_ends_at", state.cooldown_ends_at);
        }
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
