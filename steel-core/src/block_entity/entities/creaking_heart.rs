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
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey, locks::SyncMutex};

use glam::DVec3;
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
    /// Vanilla `CreakingHeartBlockEntity.tick`: while the heart sits between two
    /// pale oak logs and a player is nearby, it keeps exactly one creaking alive.
    fn spawn_creaking(&self, world: &Arc<World>) {
        if self.creaking().is_some() {
            return;
        }
        let pos = self.get_block_pos();
        if !Self::is_valid_heart(world, pos) || !Self::has_nearby_player(world, pos) {
            return;
        }
        let position = DVec3::new(
            f64::from(pos.x()) + 0.5,
            f64::from(pos.y()) + 0.5,
            f64::from(pos.z()) + 0.5,
        );
        let creaking = ENTITIES.create(
            &vanilla_entities::CREAKING,
            next_entity_id(),
            position,
            Arc::downgrade(world),
        );
        let Some(creaking) = creaking else {
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
        }
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

    /// Vanilla `CreakingHeartBlock.isValid`: the heart needs pale oak logs on both
    /// the X and Z axes.
    fn is_valid_heart(world: &World, pos: BlockPos) -> bool {
        let axis = |direction: Direction| {
            world
                .get_block_state(pos.relative(direction))
                .get_block()
                == &vanilla_blocks::PALE_OAK_LOG
        };
        (axis(Direction::North) && axis(Direction::South))
            || (axis(Direction::East) && axis(Direction::West))
    }
}

impl BlockEntity for CreakingHeartBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn tick(&self, world: &Arc<World>) {
        self.spawn_creaking(world);
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
