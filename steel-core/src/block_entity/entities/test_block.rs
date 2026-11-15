//! Test block entity.
//!
//! Vanilla `TestBlockEntity` keeps the mode, the log message and the powered
//! flag in NBT, and the mode is mirrored into the block state. A marker block
//! entity would drop all of it, so the test block editor and a `/setblock` with
//! block-entity data would lose their state.

use std::sync::{Arc, Weak};

use simdnbt::borrow::{BaseNbtCompound as BorrowedNbtCompound, NbtCompound as NbtCompoundView};
use simdnbt::owned::NbtCompound;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{BlockStateProperties, TestBlockMode};
use steel_registry::vanilla_block_entity_types;
use steel_utils::locks::SyncMutex;
use steel_utils::nbt::NbtValueInput as _;

use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey};

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::world::World;

/// Vanilla `TestBlockEntity.DEFAULT_MESSAGE`.
const DEFAULT_MESSAGE: &str = "";

/// Mutable test-block state.
#[derive(Debug, Clone, PartialEq, Eq)]
struct TestBlockState {
    mode: TestBlockMode,
    message: String,
    powered: bool,
    /// Vanilla `TestBlockEntity.triggered`: set by a non-`START` trigger, cleared
    /// by `reset`.
    triggered: bool,
}

impl TestBlockState {
    fn new(state: BlockStateId) -> Self {
        Self {
            // Vanilla takes the initial mode from the block state.
            mode: state
                .try_get_value(&BlockStateProperties::TEST_BLOCK_MODE)
                .unwrap_or(TestBlockMode::Fail),
            message: DEFAULT_MESSAGE.to_owned(),
            powered: false,
            triggered: false,
        }
    }
}

/// Vanilla `TestBlockEntity`.
pub struct TestBlockEntity {
    base: Arc<BlockEntityBase>,
    state: SyncMutex<TestBlockState>,
}

// SAFETY: This key is owned by Steel and uniquely identifies this type.
unsafe impl DowncastType for TestBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/test_block");
}

impl TestBlockEntity {
    /// Creates a new test block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self {
            base: Arc::new(BlockEntityBase::new(
                &vanilla_block_entity_types::TEST_BLOCK,
                level,
                pos,
                state,
            )),
            state: SyncMutex::new(TestBlockState::new(state)),
        }
    }

    /// Vanilla `TestBlockEntity.getMode`.
    #[must_use]
    pub fn mode(&self) -> TestBlockMode {
        self.state.lock().mode
    }

    /// Vanilla `TestBlockEntity.setMode`: stores the mode and mirrors it into the
    /// block state so the client renders the right variant.
    pub fn set_mode(&self, mode: TestBlockMode) {
        self.state.lock().mode = mode;
        self.update_block_state();
    }

    /// Vanilla `TestBlockEntity.getMessage`.
    #[must_use]
    pub fn message(&self) -> String {
        self.state.lock().message.clone()
    }

    /// Vanilla `TestBlockEntity.setMessage`.
    pub fn set_message(&self, message: String) {
        self.state.lock().message = message;
    }

    /// Vanilla `TestBlockEntity.isPowered`.
    #[must_use]
    pub fn is_powered(&self) -> bool {
        self.state.lock().powered
    }

    /// Vanilla `TestBlockEntity.setPowered`.
    pub fn set_powered(&self, powered: bool) {
        self.state.lock().powered = powered;
    }

    /// Vanilla `TestBlockEntity.updateBlockState`.
    fn update_block_state(&self) {
        let Some(world) = self.base.level() else {
            return;
        };
        let pos = self.base.pos();
        let state = world.get_block_state(pos);
        // Vanilla `blockState.is(Blocks.TEST_BLOCK)`.
        if state.get_block() != &steel_registry::vanilla_blocks::TEST_BLOCK {
            return;
        }
        let mode = self.state.lock().mode;
        let updated = state.set_value(&BlockStateProperties::TEST_BLOCK_MODE, mode);
        if updated != state {
            world.set_block(
                pos,
                updated,
                steel_utils::types::UpdateFlags::UPDATE_CLIENTS,
            );
        }
    }

    /// Vanilla `TestBlockEntity.hasTriggered`.
    #[must_use]
    pub fn has_triggered(&self) -> bool {
        self.state.lock().triggered
    }

    /// Vanilla `TestBlockEntity.reset`: clears the trigger flag and, for a
    /// `START` block, un-powers itself and updates its neighbors so the next
    /// test run starts clean.
    pub fn reset(&self, world: &Arc<World>) {
        self.state.lock().triggered = false;
        if self.mode() != TestBlockMode::Start {
            return;
        }
        self.set_powered(false);
        let pos = self.base.pos();
        let block = world.get_block_state(pos).get_block();
        world.update_neighbors_at(pos, block);
    }

    /// Vanilla `TestBlockEntity.trigger`: a `START` block powers itself, updates
    /// its neighbors and logs; the other modes only log and latch the flag.
    pub fn trigger(&self, world: &Arc<World>) {
        if self.mode() == TestBlockMode::Start {
            self.set_powered(true);
            let pos = self.base.pos();
            let block = world.get_block_state(pos).get_block();
            world.update_neighbors_at(pos, block);
            self.log();
            return;
        }
        if self.mode() == TestBlockMode::Log {
            self.log();
        }
        self.state.lock().triggered = true;
    }

    /// Vanilla `TestBlockEntity.log`: the message is reported when it is not
    /// blank.
    pub fn log(&self) {
        let message = self.message();
        if message.trim().is_empty() {
            return;
        }
        log::info!(
            "Test {} (at {}): {}",
            test_mode_name(self.mode()),
            self.base.pos(),
            message
        );
    }
}

/// Vanilla `TestBlockMode.getSerializedName`.
const fn test_mode_name(mode: TestBlockMode) -> &'static str {
    match mode {
        TestBlockMode::Start => "start",
        TestBlockMode::Log => "log",
        TestBlockMode::Fail => "fail",
        TestBlockMode::Accept => "accept",
    }
}

/// Parses a stored mode name, defaulting to `fail` like vanilla's codec.
fn test_mode_from_name(name: &str) -> TestBlockMode {
    match name {
        "start" => TestBlockMode::Start,
        "log" => TestBlockMode::Log,
        "accept" => TestBlockMode::Accept,
        _ => TestBlockMode::Fail,
    }
}

impl BlockEntity for TestBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let nbt_view: NbtCompoundView<'_, '_> = nbt.into();
        let nbt: NbtCompoundView<'_, '_> = nbt_view.into();
        let mut state = self.state.lock();
        state.mode = nbt
            .string("mode")
            .map_or(TestBlockMode::Fail, |value| test_mode_from_name(value.to_str()));
        state.message = nbt
            .string("message")
            .map_or_else(|| DEFAULT_MESSAGE.to_owned(), |value| value.to_str().to_owned());
        state.powered = nbt_view.byte_value("powered").is_some_and(|value| value != 0);
    }

    /// Vanilla `TestBlockEntity.getUpdateTag`: `saveCustomOnly`.
    fn get_update_tag(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        self.save_additional(&mut nbt);
        Some(nbt)
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        let state = self.state.lock();
        nbt.insert("mode", test_mode_name(state.mode).to_owned());
        nbt.insert("message", state.message.clone());
        nbt.insert("powered", u8::from(state.powered));
    }
}
