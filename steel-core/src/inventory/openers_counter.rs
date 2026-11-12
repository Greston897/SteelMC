//! Container viewer counting, mirroring vanilla `ContainerOpenersCounter`.
//!
//! Vanilla tracks how many players currently have a container open so it can
//! switch the `open` block state, emit open/close sounds, and notify neighbours
//! (which drives comparators and the piglin anger check). The counter is
//! deliberately per block entity: a double chest runs one counter per half, so
//! both halves open together while each keeps its own viewer count.

use std::sync::atomic::{AtomicI32, Ordering};

use steel_registry::blocks::properties::BlockStateProperties;
use steel_registry::blocks::{BlockRef, block_state_ext::BlockStateExt};
use steel_registry::sound_event::SoundEventRef;
use steel_utils::{BlockPos, BlockStateId};

use crate::world::World;

/// Sounds and state hook used when the viewer count changes.
#[derive(Clone, Copy)]
pub struct OpenerHooks {
    /// Sound played when the first viewer opens the container.
    pub open_sound: SoundEventRef,
    /// Sound played when the last viewer closes the container.
    pub close_sound: SoundEventRef,
    /// Whether the block has an `open` boolean state to drive.
    pub sets_open_state: bool,
}

/// Vanilla `ContainerOpenersCounter`.
///
/// The hooks run on the world thread; the counter itself is atomic so the block
/// behavior can query it from comparator and tick paths without locking the
/// inventory.
pub struct ContainerOpenersCounter {
    openers: AtomicI32,
    hooks: OpenerHooks,
}

impl ContainerOpenersCounter {
    /// Creates a counter for a block whose open/close emits the given sounds.
    #[must_use]
    pub const fn new(hooks: OpenerHooks) -> Self {
        Self {
            openers: AtomicI32::new(0),
            hooks,
        }
    }

    /// Creates a counter that does not drive an `open` block state (chests).
    #[must_use]
    pub const fn without_open_state(
        open_sound: SoundEventRef,
        close_sound: SoundEventRef,
    ) -> Self {
        Self::new(OpenerHooks {
            open_sound,
            close_sound,
            sets_open_state: false,
        })
    }

    /// Current number of viewers.
    #[must_use]
    pub fn openers(&self) -> i32 {
        self.openers.load(Ordering::Acquire)
    }

    /// Vanilla `ContainerOpenersCounter.incrementOpeners`.
    ///
    /// Called when a player opens the menu backed by this container.
    pub fn increment_openers(
        &self,
        world: &World,
        pos: BlockPos,
        state: BlockStateId,
        block: BlockRef,
    ) {
        let openers = self.openers.fetch_add(1, Ordering::AcqRel) + 1;
        if openers != 1 {
            return;
        }
        self.on_open(world, pos, state, block);
    }

    /// Vanilla `ContainerOpenersCounter.decrementOpeners`.
    ///
    /// Called when a player closes the menu. Returns the remaining viewer count.
    pub fn decrement_openers(
        &self,
        world: &World,
        pos: BlockPos,
        state: BlockStateId,
        block: BlockRef,
    ) -> i32 {
        let openers = self.openers.fetch_sub(1, Ordering::AcqRel) - 1;
        if openers <= 0 {
            self.openers.store(0, Ordering::Release);
            self.on_close(world, pos, state, block);
            return 0;
        }
        openers
    }

    /// Recomputes the count from scratch, matching `recheckOpeners`.
    ///
    /// Steel does not yet track per-player open menus globally, so this keeps the
    /// current optimistic count and only refreshes the block state.
    pub fn recheck_openers(
        &self,
        world: &World,
        pos: BlockPos,
        state: BlockStateId,
        block: BlockRef,
    ) {
        let openers = self.openers();
        self.sync_open_state(world, pos, state, block, openers > 0);
    }

    fn on_open(&self, world: &World, pos: BlockPos, state: BlockStateId, block: BlockRef) {
        if self.hooks.sets_open_state {
            self.sync_open_state(world, pos, state, block, true);
        }
        world.play_block_sound(self.hooks.open_sound, pos, 0.5, 0.5, None);
    }

    fn on_close(&self, world: &World, pos: BlockPos, state: BlockStateId, block: BlockRef) {
        if self.hooks.sets_open_state {
            self.sync_open_state(world, pos, state, block, false);
        }
        world.play_block_sound(self.hooks.close_sound, pos, 0.5, 0.5, None);
    }

    fn sync_open_state(
        &self,
        world: &World,
        pos: BlockPos,
        state: BlockStateId,
        block: BlockRef,
        open: bool,
    ) {
        let current = world.get_block_state(pos);
        if current.get_block() != block {
            return;
        }
        if current.get_value(&BlockStateProperties::OPEN) == open {
            return;
        }
        let updated = current.set_value(&BlockStateProperties::OPEN, open);
        world.set_block(pos, updated, steel_utils::types::UpdateFlags::UPDATE_ALL);
        let _ = state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_tracks_viewers() {
        let counter = ContainerOpenersCounter::new(OpenerHooks {
            open_sound: &steel_registry::sound_events::BLOCK_BARREL_OPEN,
            close_sound: &steel_registry::sound_events::BLOCK_BARREL_CLOSE,
            sets_open_state: true,
        });
        assert_eq!(counter.openers(), 0);
        counter.openers.fetch_add(1, Ordering::AcqRel);
        assert_eq!(counter.openers(), 1);
        counter.openers.fetch_sub(1, Ordering::AcqRel);
        assert_eq!(counter.openers(), 0);
    }
}
