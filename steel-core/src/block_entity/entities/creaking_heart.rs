//! Creaking heart block entity.
//!
//! Vanilla `CreakingHeartBlockEntity` stores the creaking it spawned and whether
//! it currently links a creaking. Breaking the heart with a creaking alive kills
//! the creaking (`creaking.transforms_to_item` behavior), and the `active` block
//! state mirrors the link.

use std::sync::{Arc, Weak};

use simdnbt::borrow::BaseNbtCompound as BorrowedNbtCompound;
use simdnbt::owned::{NbtCompound, NbtTag};
use steel_registry::vanilla_block_entity_types;
use steel_utils::{
    BlockPos, BlockStateId, Downcast as _, DowncastType, DowncastTypeKey, locks::SyncMutex,
};

use glam::DVec3;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::{vanilla_blocks, vanilla_entities};
use steel_utils::Direction;

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::entity::{ENTITIES, Entity, next_entity_id};
use crate::world::World;

/// Vanilla distance within which a player keeps a heart active.
const CREAKING_SPAWN_PLAYER_RANGE: f64 = 32.0;

/// Creaking heart block entity.
pub struct CreakingHeartBlockEntity {
    base: Arc<BlockEntityBase>,
    /// UUID of the linked creaking, matching vanilla's `creaking` field.
    creaking: SyncMutex<Option<u128>>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `CreakingHeartBlockEntity`.
unsafe impl DowncastType for CreakingHeartBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/creaking_heart");
}

impl CreakingHeartBlockEntity {
    /// Creates a new creaking heart block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self {
            base: Arc::new(BlockEntityBase::new(
                &vanilla_block_entity_types::CREAKING_HEART,
                level,
                pos,
                state,
            )),
            creaking: SyncMutex::new(None),
        }
    }

    /// Returns the linked creaking UUID, if any.
    #[must_use]
    pub fn creaking(&self) -> Option<u128> {
        *self.creaking.lock()
    }

    /// Links a creaking to this heart.
    pub fn set_creaking(&self, uuid: Option<u128>) {
        *self.creaking.lock() = uuid;
    }
}

impl CreakingHeartBlockEntity {
    /// Vanilla `CreakingHeartBlockEntity.serverTick`: checks the aligned pale oak
    /// log sandwich + time of day, syncs `CREAKING_HEART_STATE`
    /// (`Uprooted` / `Dormant` / `Awake`), spawns a linked creaking at night
    /// while a survival player is within 32 blocks, and despawns it at dawn.
    fn server_tick(&self, world: &Arc<World>) {
        if world.game_time() % 20 != 0 {
            return;
        }
        let pos = self.get_block_pos();
        let state = world.get_block_state(pos);
        if state.get_block() != &vanilla_blocks::CREAKING_HEART {
            return;
        }

        let desired_state = Self::compute_heart_state(world, pos, state);
        if state.try_get_value(&steel_registry::blocks::properties::BlockStateProperties::CREAKING_HEART_STATE)
            != Some(desired_state)
        {
            world.set_block(
                pos,
                state.set_value(
                    &steel_registry::blocks::properties::BlockStateProperties::CREAKING_HEART_STATE,
                    desired_state,
                ),
                steel_utils::types::UpdateFlags::UPDATE_ALL,
            );
        }

        // Clear stale link if the creaking died or unloaded; despawn it if the
        // heart is no longer Awake (e.g. sunrise or log removed).
        if let Some(linked_uuid) = self.creaking() {
            let uuid = uuid::Uuid::from_u128(linked_uuid);
            match world.get_entity_by_uuid(&uuid) {
                None => self.set_creaking(None),
                Some(entity) if !entity.is_alive() => self.set_creaking(None),
                Some(entity)
                    if desired_state
                        != steel_registry::blocks::properties::CreakingHeartState::Awake =>
                {
                    self.set_creaking(None);
                    let kill = crate::entity::damage::DamageSource::environment(
                        &steel_registry::vanilla_damage_types::GENERIC_KILL,
                    );
                    entity.hurt(world, &kill, f32::MAX);
                    return;
                }
                Some(_) => {
                    world.update_neighbor_for_output_signal(pos, &vanilla_blocks::CREAKING_HEART);
                    return;
                }
            }
        }

        if desired_state != steel_registry::blocks::properties::CreakingHeartState::Awake
            || world.difficulty() == steel_utils::types::Difficulty::Peaceful
            || !world.get_game_rule(&steel_registry::vanilla_game_rules::SPAWN_MOBS)
            || !Self::has_nearby_player(world, pos)
        {
            return;
        }

        let position = DVec3::new(
            f64::from(pos.x()) + 0.5,
            f64::from(pos.y()) + 0.5,
            f64::from(pos.z()) + 0.5,
        );
        let Some(creaking) = ENTITIES.create(
            &vanilla_entities::CREAKING,
            next_entity_id(),
            position,
            Arc::downgrade(world),
        ) else {
            return;
        };
        if let Some(creaking_entity) = creaking
            .downcast_ref::<crate::entity::entities::mobs::hostile::CreakingEntity>()
        {
            creaking_entity.set_creaking_heart_pos(Some(pos));
        }
        let creaking_uuid = Entity::uuid(creaking.as_ref()).as_u128();
        if world.try_add_entity(creaking).is_ok() {
            self.set_creaking(Some(creaking_uuid));
            self.set_changed();
            world.update_neighbor_for_output_signal(pos, &vanilla_blocks::CREAKING_HEART);
        }
    }

    /// Vanilla `CreakingHeartBlock.updateState`: aligned pale-oak logs along the
    /// heart's own `AXIS` give `Awake` at night or `Dormant` during the day;
    /// missing or misaligned logs leave it `Uprooted`.
    #[must_use]
    pub fn compute_heart_state(
        world: &World,
        pos: BlockPos,
        state: BlockStateId,
    ) -> steel_registry::blocks::properties::CreakingHeartState {
        use steel_registry::blocks::properties::{Axis, BlockStateProperties, CreakingHeartState};
        let axis = state
            .try_get_value(&BlockStateProperties::AXIS)
            .unwrap_or(Axis::Y);
        let (neg, pos_dir) = match axis {
            Axis::X => (Direction::West, Direction::East),
            Axis::Y => (Direction::Down, Direction::Up),
            Axis::Z => (Direction::North, Direction::South),
        };
        let is_aligned_pale_log = |dir: Direction| {
            let neighbor = world.get_block_state(pos.relative(dir));
            let block = neighbor.get_block();
            let is_pale_log = block == &vanilla_blocks::PALE_OAK_LOG
                || block == &vanilla_blocks::STRIPPED_PALE_OAK_LOG
                || block == &vanilla_blocks::PALE_OAK_WOOD
                || block == &vanilla_blocks::STRIPPED_PALE_OAK_WOOD;
            is_pale_log && neighbor.try_get_value(&BlockStateProperties::AXIS) == Some(axis)
        };
        if !is_aligned_pale_log(neg) || !is_aligned_pale_log(pos_dir) {
            CreakingHeartState::Uprooted
        } else if world.is_dark_outside() {
            CreakingHeartState::Awake
        } else {
            CreakingHeartState::Dormant
        }
    }

    /// Vanilla `CreakingHeartBlockEntity.getAnalogOutputSignal`: `15 - floor(dist / 32 * 15)`
    /// for the linked creaking, or `0` when none is active.
    #[must_use]
    pub fn analog_output_signal(&self, world: &World) -> i32 {
        let Some(linked) = self.creaking() else {
            return 0;
        };
        let Some(entity) = world.get_entity_by_uuid(&uuid::Uuid::from_u128(linked)) else {
            return 0;
        };
        let pos = self.get_block_pos();
        let center = DVec3::new(
            f64::from(pos.x()) + 0.5,
            f64::from(pos.y()) + 0.5,
            f64::from(pos.z()) + 0.5,
        );
        let dist = entity.position().distance(center);
        let scaled = (dist / CREAKING_SPAWN_PLAYER_RANGE).clamp(0.0, 1.0);
        15 - (scaled * 15.0).floor() as i32
    }

    /// Vanilla only lets the heart act while a player is close enough to see the
    /// pale garden, which keeps distant hearts from spawning mobs forever.
    fn has_nearby_player(world: &World, pos: BlockPos) -> bool {
        let center = DVec3::new(
            f64::from(pos.x()) + 0.5,
            f64::from(pos.y()) + 0.5,
            f64::from(pos.z()) + 0.5,
        );
        let mut found = false;
        world.players.iter_players(|_uuid, player| {
            if player.position().distance_squared(center)
                <= CREAKING_SPAWN_PLAYER_RANGE * CREAKING_SPAWN_PLAYER_RANGE
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

impl BlockEntity for CreakingHeartBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn tick(&self, world: &Arc<World>) {
        self.server_tick(world);
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let view: simdnbt::borrow::NbtCompound<'_, '_> = nbt.into();
        // Vanilla stores the creaking as a four-int UUID array.
        let linked = view.int_array("creaking").and_then(|values| {
            let mut bytes = [0u8; 16];
            for (index, value) in values.iter().take(4).enumerate() {
                bytes[index * 4..index * 4 + 4].copy_from_slice(&value.to_be_bytes());
            }
            (values.len() == 4).then_some(u128::from_be_bytes(bytes))
        });
        *self.creaking.lock() = linked;
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        if let Some(uuid) = self.creaking() {
            let bytes = uuid.to_be_bytes();
            let ints: Vec<i32> = bytes
                .chunks_exact(4)
                .map(|chunk| i32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect();
            nbt.insert("creaking", NbtTag::IntArray(ints));
        }
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_blocks};

    use super::*;

    #[test]
    fn heart_starts_unlinked() {
        init_vanilla_registry();
        let heart = CreakingHeartBlockEntity::new(
            Weak::new(),
            BlockPos::new(0, 0, 0),
            vanilla_blocks::CREAKING_HEART.default_state(),
        );
        assert_eq!(heart.creaking(), None);
        heart.set_creaking(Some(42));
        assert_eq!(heart.creaking(), Some(42));
    }
}
