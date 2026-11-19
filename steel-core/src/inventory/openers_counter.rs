//! Container viewer counting, mirroring vanilla `ContainerOpenersCounter`.
//!
//! Vanilla tracks how many players currently have a container open so it can
//! switch the `open` block state, emit open/close sounds, and notify neighbours
//! (which drives comparators and the piglin anger check). The counter is
//! deliberately per block entity: a double chest runs one counter per half, so
//! both halves open together while each keeps its own viewer count.

use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use steel_registry::blocks::properties::BlockStateProperties;
use steel_registry::blocks::{BlockRef, block_state_ext::BlockStateExt};
use steel_registry::sound_event::SoundEventRef;
use steel_utils::{BlockPos, BlockStateId};

use crate::world::World;
use crate::entity::Entity as _;
use crate::inventory::lock::ContainerRef;

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
    pub const fn without_open_state(open_sound: SoundEventRef, close_sound: SoundEventRef) -> Self {
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
        world: &Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
        block: BlockRef,
    ) {
        let openers = self.openers.fetch_add(1, Ordering::AcqRel) + 1;
        if openers != 1 {
            return;
        }
        self.on_open(world, pos, state, block);
        world.schedule_block_tick_default(pos, block, 5);
    }

    /// Vanilla `ContainerOpenersCounter.decrementOpeners`.
    ///
    /// Called when a player closes the menu. Returns the remaining viewer count.
    pub fn decrement_openers(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
        block: BlockRef,
    ) -> i32 {
        let old = self.openers.fetch_update(Ordering::AcqRel, Ordering::Acquire, |value| Some((value - 1).max(0))).unwrap_or(0);
        let remaining = (old - 1).max(0);
        if old == 1 {
            self.on_close(world, pos, state, block);
        }
        remaining
    }

    /// Recomputes the count from scratch, matching `recheckOpeners`.
    ///
    /// Counts nearby non-spectator players whose active menu references this
    /// exact backing inventory. Temporarily dispatched menus defer the scan.
    pub fn recheck_openers(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
        block: BlockRef,
    ) {
        let Some(container) = world.get_block_entity(pos).and_then(ContainerRef::from_block_entity) else {
            return;
        };
        let mut actual = 0;
        let mut unavailable = false;
        world.players.iter_players(|_, player| {
            let p = player.position();
            if player.is_spectator() || p.x < f64::from(pos.x() - 5) || p.x >= f64::from(pos.x() + 6)
                || p.y < f64::from(pos.y() - 5) || p.y >= f64::from(pos.y() + 6)
                || p.z < f64::from(pos.z() - 5) || p.z >= f64::from(pos.z() + 6) {
                return true;
            }
            match player.views_container(container.container_id()) {
                Some(true) => actual += 1,
                Some(false) => {},
                None => unavailable = true,
            }
            true
        });
        if unavailable {
            world.schedule_block_tick_default(pos, block, 5);
            return;
        }
        let old = self.openers.swap(actual, Ordering::AcqRel);
        if old == 0 && actual > 0 {
            self.on_open(world, pos, state, block);
        } else if old > 0 && actual == 0 {
            self.on_close(world, pos, state, block);
        }
        if self.hooks.sets_open_state {
            self.sync_open_state(world, pos, state, block, actual > 0);
        }
        if actual > 0 {
            world.schedule_block_tick_default(pos, block, 5);
        }
    }

    fn on_open(&self, world: &Arc<World>, pos: BlockPos, state: BlockStateId, block: BlockRef) {
        if self.hooks.sets_open_state {
            self.sync_open_state(world, pos, state, block, true);
        }
        world.play_block_sound(self.hooks.open_sound, pos, 0.5, 0.5, None);
    }

    fn on_close(&self, world: &Arc<World>, pos: BlockPos, state: BlockStateId, block: BlockRef) {
        if self.hooks.sets_open_state {
            self.sync_open_state(world, pos, state, block, false);
        }
        world.play_block_sound(self.hooks.close_sound, pos, 0.5, 0.5, None);
    }

    fn sync_open_state(
        &self,
        world: &Arc<World>,
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

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::{bootstrap::init_globals, block_entity::entities::{BarrelBlockEntity, ChestBlockEntity},
        inventory::menu::kinds::{chest, double_chest},
        test_support::{fresh_test_world, insert_ready_full_chunk, tick_test_world, TestPlayerBuilder}};
    use steel_utils::{ChunkPos, Downcast as _, types::{GameType, UpdateFlags}};
    use steel_registry::vanilla_blocks;
    use glam::DVec3;
    use text_components::TextComponent;

    const POS: BlockPos = BlockPos::new(8, 64, 8);

    #[test]
    fn barrel_menu_lifetime_updates_viewers_and_closed_state() {
        init_globals();
        let fixture = fresh_test_world("barrel_menu_viewers");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        world.set_block(POS, vanilla_blocks::BARREL.default_state(), UpdateFlags::UPDATE_ALL);
        let entity = world.get_block_entity(POS).unwrap();
        let barrel = entity.downcast_ref::<BarrelBlockEntity>().unwrap();
        let first = TestPlayerBuilder::new(Arc::clone(world), "ViewerA", 1).build();
        let second = TestPlayerBuilder::new(Arc::clone(world), "ViewerB", 2).build();
        for player in [&first, &second] {
            player.base().set_position_local(DVec3::new(8.5, 64.0, 7.5));
            world.players.insert(Arc::clone(player));
            let container = entity.container_ref().unwrap();
            let inventory = Arc::clone(&player.inventory);
            player.open_menu(TextComponent::plain("Barrel"), move |ctx| chest(inventory, ctx.container_id, container, 3));
        }
        assert_eq!(barrel.openers_counter().openers(), 2);
        assert!(world.get_block_state(POS).get_value(&BlockStateProperties::OPEN));
        barrel.openers_counter().recheck_openers(world, POS, world.get_block_state(POS), &vanilla_blocks::BARREL);
        assert_eq!(barrel.openers_counter().openers(), 2);
        first.close_container();
        assert_eq!(barrel.openers_counter().openers(), 1);
        assert!(world.get_block_state(POS).get_value(&BlockStateProperties::OPEN));
        second.close_container();
        assert_eq!(barrel.openers_counter().openers(), 0);
        assert!(!world.get_block_state(POS).get_value(&BlockStateProperties::OPEN));
        // A second cleanup after a scan must not underflow the count.
        barrel.openers_counter().decrement_openers(world, POS, world.get_block_state(POS), &vanilla_blocks::BARREL);
        assert_eq!(barrel.openers_counter().openers(), 0);
    }

    #[test]
    fn scheduled_recheck_clears_stale_barrel_opener_without_menu() {
        init_globals();
        let fixture = fresh_test_world("barrel_stale_recheck");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        world.set_block(POS, vanilla_blocks::BARREL.default_state(), UpdateFlags::UPDATE_ALL);
        let entity = world.get_block_entity(POS).unwrap();
        let barrel = entity.downcast_ref::<BarrelBlockEntity>().unwrap();
        barrel.openers_counter().increment_openers(world, POS, world.get_block_state(POS), &vanilla_blocks::BARREL);
        assert_eq!(barrel.openers_counter().openers(), 1);
        for tick in 0..6 { tick_test_world(world, tick, true); }
        assert_eq!(barrel.openers_counter().openers(), 0);
        assert!(!world.get_block_state(POS).get_value(&BlockStateProperties::OPEN));
    }

    #[test]
    fn double_chest_counts_each_half_and_scan_filters_spectators_and_distance() {
        init_globals();
        let fixture = fresh_test_world("double_chest_viewers");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let other_pos = POS.offset(1, 0, 0);
        for pos in [POS, other_pos] {
            world.set_block(pos, vanilla_blocks::CHEST.default_state(), UpdateFlags::UPDATE_ALL);
        }
        let a = world.get_block_entity(POS).unwrap();
        let b = world.get_block_entity(other_pos).unwrap();
        let ca = a.downcast_ref::<ChestBlockEntity>().unwrap().openers_counter();
        let cb = b.downcast_ref::<ChestBlockEntity>().unwrap().openers_counter();
        let player = TestPlayerBuilder::new(Arc::clone(world), "DoubleViewer", 1).build();
        player.base().set_position_local(DVec3::new(8.5, 64.0, 7.5));
        world.players.insert(Arc::clone(&player));
        let first = a.container_ref().unwrap();
        let second = b.container_ref().unwrap();
        let inventory = Arc::clone(&player.inventory);
        player.open_menu(TextComponent::plain("Double"), move |ctx| double_chest(inventory, ctx.container_id, first, second));
        assert_eq!((ca.openers(), cb.openers()), (1, 1));
        for (pos, counter) in [(POS, ca), (other_pos, cb)] {
            counter.recheck_openers(world, pos, world.get_block_state(pos), &vanilla_blocks::CHEST);
            assert_eq!(counter.openers(), 1);
        }
        // Radius scan only counts nearby non-spectators with this inventory.
        player.base().set_position_local(DVec3::new(40.0, 64.0, 7.5));
        ca.recheck_openers(world, POS, world.get_block_state(POS), &vanilla_blocks::CHEST);
        assert_eq!(ca.openers(), 0);
        player.base().set_position_local(DVec3::new(8.5, 64.0, 7.5));
        ca.recheck_openers(world, POS, world.get_block_state(POS), &vanilla_blocks::CHEST);
        assert_eq!(ca.openers(), 1);
        player.set_game_mode(GameType::Spectator);
        ca.recheck_openers(world, POS, world.get_block_state(POS), &vanilla_blocks::CHEST);
        assert_eq!(ca.openers(), 0);
        player.close_container();
        cb.recheck_openers(world, other_pos, world.get_block_state(other_pos), &vanilla_blocks::CHEST);
        assert_eq!(cb.openers(), 0);
    }
}
