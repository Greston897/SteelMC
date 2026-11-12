//! Sculk catalyst and sculk shrieker block entities.
//!
//! Both blocks are pure markers on the server: the catalyst reacts to entity
//! deaths through the game-event system, and the shrieker reacts to vibrations.
//! Vanilla still allocates block entities for them because the client reads their
//! animation state from the render data, and the shrieker persists whether a
//! player has been warned.

use std::sync::{Arc, Weak};

use simdnbt::borrow::BaseNbtCompound as BorrowedNbtCompound;
use simdnbt::owned::NbtCompound;
use steel_registry::block_entity_type::BlockEntityTypeRef;
use steel_registry::vanilla_block_entity_types;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey, locks::SyncMutex};

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::world::World;

/// Sculk catalyst block entity: only carries the base state.
pub struct SculkCatalystBlockEntity {
    base: Arc<BlockEntityBase>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `SculkCatalystBlockEntity`.
unsafe impl DowncastType for SculkCatalystBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/sculk_catalyst");
}

impl SculkCatalystBlockEntity {
    /// Creates a new sculk catalyst block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self {
            base: Arc::new(BlockEntityBase::new(
                &vanilla_block_entity_types::SCULK_CATALYST,
                level,
                pos,
                state,
            )),
        }
    }
}

impl BlockEntity for SculkCatalystBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }
}

/// Sculk shrieker block entity.
pub struct SculkShriekerBlockEntity {
    base: Arc<BlockEntityBase>,
    /// Warning level, incremented each time a player triggers the shrieker and
    /// persisted across saves like vanilla's `warningLevel`.
    warning_level: SyncMutex<i32>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `SculkShriekerBlockEntity`.
unsafe impl DowncastType for SculkShriekerBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/sculk_shrieker");
}

impl SculkShriekerBlockEntity {
    /// Creates a new sculk shrieker block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self {
            base: Arc::new(BlockEntityBase::new(
                &vanilla_block_entity_types::SCULK_SHRIEKER,
                level,
                pos,
                state,
            )),
            warning_level: SyncMutex::new(0),
        }
    }

    /// Current warning level.
    #[must_use]
    pub fn warning_level(&self) -> i32 {
        *self.warning_level.lock()
    }

    /// Sets the warning level.
    pub fn set_warning_level(&self, level: i32) {
        *self.warning_level.lock() = level;
    }
}

impl BlockEntity for SculkShriekerBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let view: simdnbt::borrow::NbtCompound<'_, '_> = nbt.into();
        *self.warning_level.lock() = view.int("warning_level").unwrap_or(0);
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        nbt.insert("warning_level", *self.warning_level.lock());
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_blocks};

    use super::*;

    #[test]
    fn shrieker_tracks_warning_level() {
        init_vanilla_registry();
        let shrieker = SculkShriekerBlockEntity::new(
            Weak::new(),
            BlockPos::new(0, 0, 0),
            vanilla_blocks::SCULK_SHRIEKER.default_state(),
        );
        assert_eq!(shrieker.warning_level(), 0);
        shrieker.set_warning_level(2);
        assert_eq!(shrieker.warning_level(), 2);
    }
}
