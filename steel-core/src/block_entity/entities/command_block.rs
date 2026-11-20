//! Vanilla `CommandBlockEntity` and `BaseCommandBlock`.
//!
//! One block entity serves the three command blocks; the block decides the mode,
//! exactly like vanilla `CommandBlockEntity.getMode` (`command_block` is
//! `Mode.REDSTONE`, `repeating_command_block` is `Mode.AUTO`, `chain_command_block`
//! is `Mode.SEQUENCE`).
//!
//! Execution model, matching vanilla:
//!
//! * `CommandBlock.neighborChanged` samples redstone and, on a rising edge, calls
//!   `markConditionMet` and schedules a one-tick block tick — except for a
//!   powered automatic/sequence block, which runs straight away.
//! * `CommandBlock.tick` runs the command for `Mode.REDSTONE` when the condition
//!   was met, and for `Mode.AUTO` re-schedules itself while powered.
//! * `CommandBlock.executeChain` walks the blocks in the facing direction and
//!   runs every powered chain block whose condition holds, stopping at the first
//!   failure or at `max_command_sequence_length`.
//! * `BaseCommandBlock.performCommand` refuses to run twice in one game tick
//!   (`lastExecution`), counts successes, and stores the output prefixed with the
//!   wall-clock time.
//!
//! Steel defers the actual command to the server's tick-owned command queue, as
//! it does for every other command source. A block therefore finishes its run on
//! a later tick than vanilla's inline execution; everything else (the tick gate,
//! the success count, the output, the chain order) follows the vanilla code.

use std::sync::{Arc, Weak};

use chrono::Local;
use simdnbt::borrow::{BaseNbtCompound as BorrowedNbtCompound, NbtCompound as NbtCompoundView};
use simdnbt::owned::NbtCompound;
use steel_registry::block_entity_type::BlockEntityTypeRef;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::blocks::properties::BlockStateProperties;
use steel_registry::vanilla_game_rules::COMMAND_BLOCKS_WORK;
use steel_registry::{vanilla_block_entity_types, vanilla_blocks};
use steel_utils::locks::SyncMutex;
use steel_utils::nbt::NbtValueInput as _;
use tokio::sync::oneshot::Receiver;
use tokio::sync::oneshot::error::TryRecvError::Closed;
use tokio::sync::oneshot::error::TryRecvError::Empty;

use steel_utils::{
    BlockPos, BlockStateId, Direction, Downcast as _, DowncastType, DowncastTypeKey,
};
use text_components::TextComponent;

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::command::BackgroundCommandResult;
use crate::world::World;

/// Vanilla `BaseCommandBlock` clock prefix format: `HH:mm:ss`.
const TIME_FORMAT: &str = "%H:%M:%S";

/// Vanilla `CommandBlockEntity.Mode`, named after the block that carries it.
///
/// No `Default`: vanilla has no default mode, the mode is always implied by
/// the block (see [`CommandBlockMode::for_block_key`]), and a derived default
/// would silently mean "redstone" for the other two blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandBlockMode {
    /// `chain_command_block`: runs when the block behind it succeeded.
    Sequence,
    /// `repeating_command_block`: runs every tick while powered.
    Auto,
    /// `command_block`: runs once per rising redstone edge.
    Redstone,
}

impl CommandBlockMode {
    /// The mode a block state implies, exactly like `CommandBlockEntity.getMode`.
    #[must_use]
    pub fn for_block_key(key: &steel_utils::Identifier) -> Self {
        if key == &vanilla_blocks::REPEATING_COMMAND_BLOCK.key {
            Self::Auto
        } else if key == &vanilla_blocks::CHAIN_COMMAND_BLOCK.key {
            Self::Sequence
        } else {
            // `command_block`, and vanilla's fallback for anything else.
            Self::Redstone
        }
    }

    /// Parses the ordinals of the `SetCommandBlock` packet (`SEQUENCE`, `AUTO`,
    /// `REDSTONE`).
    #[must_use]
    pub const fn from_packet_ordinal(ordinal: i32) -> Self {
        match ordinal {
            0 => Self::Sequence,
            1 => Self::Auto,
            _ => Self::Redstone,
        }
    }

    /// Returns the packet ordinal.
    #[must_use]
    pub const fn packet_ordinal(self) -> i32 {
        match self {
            Self::Sequence => 0,
            Self::Auto => 1,
            Self::Redstone => 2,
        }
    }
}

/// Vanilla `BaseCommandBlock` state plus the entity-owned flags.
#[derive(Debug)]
struct CommandBlockState {
    command: String,
    /// Vanilla `BaseCommandBlock.successCount`, also the comparator output.
    success_count: i32,
    track_output: bool,
    last_output: Option<TextComponent>,
    /// Vanilla `BaseCommandBlock.lastExecution`, in game time.
    last_execution: i64,
    update_last_execution: bool,
    /// Vanilla `CommandBlockEntity.auto`.
    automatic: bool,
    /// Vanilla `CommandBlockEntity.powered`.
    powered: bool,
    /// Vanilla `CommandBlockEntity.conditionMet`.
    condition_met: bool,
}

impl Default for CommandBlockState {
    fn default() -> Self {
        Self {
            command: String::new(),
            success_count: 0,
            track_output: true,
            last_output: None,
            last_execution: -1,
            update_last_execution: true,
            automatic: false,
            powered: false,
            condition_met: false,
        }
    }
}

/// Reply of one queued command plus the block it belongs to.
struct PendingRun {
    receiver: Receiver<BackgroundCommandResult>,
    /// Game time at which the run was queued, for `lastExecution`.
    queued_at: i64,
}

/// Vanilla `CommandBlockEntity`.
///
/// The mutexes are shared handles rather than plain fields so a clone of the
/// entity talks to the same command state, which is what the block behavior
/// relies on when it takes the entity out of the world to run a command.
#[derive(Clone)]
pub struct CommandBlockBlockEntity {
    base: Arc<BlockEntityBase>,
    state: Arc<SyncMutex<CommandBlockState>>,
    pending: Arc<SyncMutex<Option<PendingRun>>>,
}

// SAFETY: This key is owned by Steel and uniquely identifies the type.
unsafe impl DowncastType for CommandBlockBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/command_block");
}

impl CommandBlockBlockEntity {
    /// Creates an empty command block.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self::new_with_type(
            &vanilla_block_entity_types::COMMAND_BLOCK,
            level,
            pos,
            state,
        )
    }

    /// Creates a command block for the given registered block entity type.
    #[must_use]
    pub fn new_with_type(
        block_entity_type: BlockEntityTypeRef,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> Self {
        // Vanilla `CommandBlock.newBlockEntity` stamps the variant's automatic
        // flag onto the fresh entity.
        let automatic =
            CommandBlockMode::for_block_key(&state.get_block().key) == CommandBlockMode::Auto;
        Self {
            base: Arc::new(BlockEntityBase::new(block_entity_type, level, pos, state)),
            state: Arc::new(SyncMutex::new(CommandBlockState {
                automatic,
                ..CommandBlockState::default()
            })),
            pending: Arc::new(SyncMutex::new(None)),
        }
    }

    /// Returns the stored command.
    #[must_use]
    pub fn command(&self) -> String {
        self.state.lock().command.clone()
    }

    /// Vanilla `BaseCommandBlock.setCommand`: replacing the command also clears
    /// the success count.
    pub fn set_command(&self, command: impl Into<String>) {
        {
            let mut state = self.state.lock();
            state.command = command.into();
            state.success_count = 0;
        }
        // Vanilla `CommandBlockEntity`'s `BaseCommandBlock.setCommand` override
        // calls `setChanged()`, so the new command is written to disk even when
        // output tracking is off.
        self.base.set_changed();
    }

    /// Returns whether the command is empty or blank.
    #[must_use]
    pub fn command_is_empty(&self) -> bool {
        self.state.lock().command.trim().is_empty()
    }

    /// Vanilla `BaseCommandBlock.getLastOutput`.
    #[must_use]
    pub fn last_output(&self) -> TextComponent {
        self.state
            .lock()
            .last_output
            .clone()
            .unwrap_or_else(|| TextComponent::plain(""))
    }

    /// Vanilla `BaseCommandBlock.setLastOutput`.
    pub fn set_last_output(&self, output: Option<TextComponent>) {
        self.state.lock().last_output = output;
    }

    /// Vanilla `BaseCommandBlock.isTrackOutput`.
    #[must_use]
    pub fn tracks_output(&self) -> bool {
        self.state.lock().track_output
    }

    /// Vanilla `BaseCommandBlock.setTrackOutput`.
    pub fn set_track_output(&self, track: bool) {
        let mut state = self.state.lock();
        state.track_output = track;
        if !track {
            state.last_output = None;
        }
    }

    /// Returns whether the block is conditional.
    #[must_use]
    pub fn is_conditional(&self) -> bool {
        self.base
            .block_state()
            .get_value(&BlockStateProperties::CONDITIONAL)
    }

    /// Vanilla `CommandBlockEntity.isAutomatic`.
    #[must_use]
    pub fn is_automatic(&self) -> bool {
        self.state.lock().automatic
    }

    /// Vanilla `CommandBlockEntity.setAutomatic`.
    pub fn set_automatic(&self, automatic: bool) {
        self.state.lock().automatic = automatic;
    }

    /// Returns the mode implied by the current block state.
    #[must_use]
    pub fn mode(&self) -> CommandBlockMode {
        CommandBlockMode::for_block_key(&self.base.block_state().get_block().key)
    }

    /// Vanilla `CommandBlockEntity.setPowered`.
    pub fn set_powered(&self, powered: bool) {
        self.state.lock().powered = powered;
    }

    /// Vanilla `CommandBlockEntity.onModeSwitch`: switching a block into `AUTO`
    /// while it is powered (or automatic) arms it for the next tick.
    pub fn on_mode_switch(&self, previous_mode: CommandBlockMode, world: &Arc<World>) {
        if previous_mode == self.mode() {
            return;
        }
        if self.mode() == CommandBlockMode::Auto && (self.is_powered() || self.is_automatic()) {
            self.schedule_tick(world, 1);
        }
    }

    /// Vanilla `CommandBlockEntity.scheduleTick`: mark the condition and ask the
    /// block for a one-tick delay.
    pub fn schedule_tick(&self, world: &Arc<World>, delay: i32) {
        let _ = self.mark_condition_met();
        world.schedule_block_tick_default(
            self.get_block_pos(),
            self.base.block_state().get_block(),
            delay,
        );
    }

    /// Vanilla `CommandBlockEntity.setAutomatic`: turning a non-sequence block
    /// automatic while unpowered arms it.
    pub fn set_automatic_and_schedule(&self, automatic: bool, world: &Arc<World>) {
        let was = self.is_automatic();
        self.set_automatic(automatic);
        if !was && automatic && !self.is_powered() && self.mode() != CommandBlockMode::Sequence {
            self.schedule_tick(world, 1);
        }
    }

    /// Vanilla `CommandBlockEntity.isPowered`.
    #[must_use]
    pub fn is_powered(&self) -> bool {
        self.state.lock().powered
    }

    /// Vanilla `CommandBlockEntity.wasConditionMet`.
    #[must_use]
    pub fn was_condition_met(&self) -> bool {
        self.state.lock().condition_met
    }

    /// Vanilla `CommandBlockEntity.markConditionMet`.
    ///
    /// An unconditional block always satisfies its condition. A conditional one
    /// looks at the block behind it and requires that block to be a command block
    /// whose last run had a success count above zero.
    #[must_use]
    pub fn mark_condition_met(&self) -> bool {
        let conditional = self.is_conditional();
        if !conditional {
            self.state.lock().condition_met = true;
            return true;
        }
        let Some(world) = self.base.level() else {
            return false;
        };
        // Vanilla `CommandBlockEntity.markConditionMet` only reads the block
        // behind when that block *is* a command block; otherwise the condition
        // is simply false. Missing that check made a conditional block read a
        // stale block entity that happened to remain at the position.
        let met = {
            let pos = self.source_pos(world.as_ref());
            let behind_state = world.get_block_state(pos);
            Self::is_command_block(behind_state.get_block())
                && world
                    .get_block_entity(pos)
                    .and_then(|entity| entity.downcast_ref::<Self>().cloned())
                    .is_some_and(|behind| behind.success_count() > 0)
        };
        self.state.lock().condition_met = met;
        met
    }

    /// Vanilla `BaseCommandBlock.getSuccessCount`.
    #[must_use]
    pub fn success_count(&self) -> i32 {
        self.state.lock().success_count
    }

    /// Vanilla `BaseCommandBlock.setSuccessCount`.
    pub fn set_success_count(&self, count: i32) {
        self.state.lock().success_count = count;
    }

    /// Vanilla `CommandBlockEntity.wasConditionMet` after a failed run: a
    /// conditional block clears its success count instead of running.
    pub fn clear_success_count_if_conditional(&self) {
        if self.is_conditional() {
            self.set_success_count(0);
        }
    }

    /// Returns whether a command execution is still in flight.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.pending.lock().is_some()
    }

    /// The block this one reads its condition from (vanilla
    /// `worldPosition.relative(FACING.getOpposite())`).
    /// Vanilla `instanceof CommandBlock`: the three vanilla command blocks.
    fn is_command_block(block: BlockRef) -> bool {
        block.key == vanilla_blocks::COMMAND_BLOCK.key
            || block.key == vanilla_blocks::REPEATING_COMMAND_BLOCK.key
            || block.key == vanilla_blocks::CHAIN_COMMAND_BLOCK.key
    }

    fn source_pos(&self, world: &World) -> BlockPos {
        let state = world.get_block_state(self.get_block_pos());
        let facing: Direction = state.get_value(&BlockStateProperties::FACING);
        self.get_block_pos().relative(facing.opposite())
    }
}

impl BlockEntity for CommandBlockBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn tick(&self, world: &Arc<World>) {
        self.poll_pending(world);
    }

    fn get_update_tag(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        self.save_additional(&mut nbt);
        Some(nbt)
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let nbt_view: NbtCompoundView<'_, '_> = nbt.into();
        let mut state = self.state.lock();
        state.command = nbt_view
            .string("Command")
            .map_or_else(String::new, |command| command.to_str().into_owned());
        state.success_count = nbt_view.int_value("SuccessCount").unwrap_or(0);
        state.track_output = nbt_view
            .byte_value("TrackOutput")
            .is_none_or(|value| value != 0);
        // Vanilla only restores the output while tracking it.
        state.last_output = if state.track_output {
            nbt_view
                .string("LastOutput")
                .map(|output| TextComponent::plain(output.to_str().into_owned()))
        } else {
            None
        };
        state.update_last_execution = nbt_view
            .byte_value("UpdateLastExecution")
            .is_none_or(|value| value != 0);
        state.last_execution = if state.update_last_execution {
            nbt_view.long_value("LastExecution").unwrap_or(-1)
        } else {
            -1
        };
        state.powered = nbt_view
            .byte_value("powered")
            .is_some_and(|value| value != 0);
        state.condition_met = nbt_view
            .byte_value("conditionMet")
            .is_some_and(|value| value != 0);
        // Vanilla `CommandBlockEntity.loadAdditional` calls
        // `setAutomatic(input.getBooleanOr("auto", false))`, so the stored bit
        // always wins over the class-level flag that `newBlockEntity` applied —
        // including when it is false. Loading must not schedule a tick, which is
        // why this assigns the field instead of going through
        // `set_automatic_and_schedule`.
        state.automatic = nbt_view.byte_value("auto").is_some_and(|value| value != 0);
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        let state = self.state.lock();
        nbt.insert("Command", state.command.clone());
        nbt.insert("SuccessCount", state.success_count);
        nbt.insert("TrackOutput", u8::from(state.track_output));
        if state.track_output
            && let Some(output) = &state.last_output
        {
            nbt.insert("LastOutput", output.to_string());
        }
        nbt.insert("UpdateLastExecution", u8::from(state.update_last_execution));
        if state.update_last_execution && state.last_execution != -1 {
            nbt.insert("LastExecution", state.last_execution);
        }
        nbt.insert("powered", u8::from(state.powered));
        nbt.insert("conditionMet", u8::from(state.condition_met));
        nbt.insert("auto", u8::from(state.automatic));
    }
}

impl CommandBlockBlockEntity {
    /// Vanilla `BaseCommandBlock.performCommand`, minus the inline execution.
    ///
    /// Mirrors the vanilla return value exactly: `false` only when the block
    /// already ran in this tick (`gameTime == lastExecution`). An empty command,
    /// a disabled `command_blocks_work` rule or a full queue still return `true`,
    /// because vanilla still consumed the run and `executeChain` must continue
    /// down the chain. The `"Searge"` easter egg is kept as well.
    pub(crate) fn try_perform_command(&self, world: &Arc<World>) -> bool {
        let game_time = world.game_time();
        {
            let mut state = self.state.lock();
            if state.last_execution == game_time {
                return false;
            }
            if state.command.eq_ignore_ascii_case("Searge") {
                state.last_output = Some(TextComponent::plain("#itzlipofutzli"));
                state.success_count = 1;
                return true;
            }
            // Vanilla falls through to this branch for an empty command and for
            // a disabled rule: the success count resets and the run is consumed.
            state.success_count = 0;
            let runnable =
                !state.command.trim().is_empty() && world.get_game_rule(&COMMAND_BLOCKS_WORK);
            state.last_output = None;
            if state.update_last_execution {
                state.last_execution = game_time;
            } else {
                state.last_execution = -1;
            }
            if !runnable {
                return true;
            }
        }

        let command = self.command();
        let Some(server) = world.server() else {
            return true;
        };
        // A full command queue drops this run, exactly like a command that
        // produced no output: vanilla still reports the block as consumed.
        let Ok(receiver) = server.execute_command_for_block(Arc::clone(world), command) else {
            return true;
        };
        *self.pending.lock() = Some(PendingRun {
            receiver,
            queued_at: game_time,
        });
        true
    }

    /// Polls a queued run, storing its result like vanilla's callback.
    fn poll_pending(&self, world: &Arc<World>) {
        let mut pending = self.pending.lock();
        let Some(run) = pending.as_mut() else {
            return;
        };
        match run.receiver.try_recv() {
            Ok(result) => {
                let queued_at = run.queued_at;
                *pending = None;
                drop(pending);
                self.apply_result(world, result, queued_at);
                // Vanilla `BaseCommandBlock.onUpdated` → `sendBlockUpdated`.
                world.broadcast_block_entity_if_needed(self.get_block_pos());
            }
            Err(Empty) => {}
            Err(Closed) => {
                *pending = None;
            }
        }
    }

    /// Applies a finished command's outcome.
    fn apply_result(&self, world: &Arc<World>, result: BackgroundCommandResult, _queued_at: i64) {
        let mut state = self.state.lock();
        // Vanilla's callback increments the success count once per successful
        // command; Steel runs exactly one command per block run.
        state.success_count = i32::from(result.success);
        if let Some(output) = result.output
            && state.track_output
        {
            // Vanilla prefixes the store line with the wall-clock time.
            let stamp = Local::now().format(TIME_FORMAT);
            state.last_output = Some(TextComponent::plain(format!("[{stamp}] {output}")));
        }
        drop(state);

        // The chain is walked by `CommandBlock::tick` through the block behavior,
        // which owns the mode rules; here only the comparator value changes.
        world.update_neighbor_for_output_signal(
            self.get_block_pos(),
            self.base.block_state().get_block(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity() -> CommandBlockBlockEntity {
        steel_registry::init_vanilla_registry();
        CommandBlockBlockEntity::new(
            Weak::new(),
            BlockPos::new(0, 64, 0),
            vanilla_blocks::COMMAND_BLOCK.default_state(),
        )
    }

    #[test]
    fn a_new_block_is_empty_and_unpowered() {
        let block = entity();
        assert!(block.command_is_empty());
        assert!(!block.is_automatic());
        assert!(!block.is_powered());
        assert!(!block.was_condition_met());
        assert_eq!(block.success_count(), 0);
        assert!(block.tracks_output());
    }

    #[test]
    fn setting_the_command_clears_the_success_count() {
        let block = entity();
        block.set_success_count(3);
        block.set_command("say hi");
        assert_eq!(block.command(), "say hi");
        assert_eq!(block.success_count(), 0);
    }

    #[test]
    fn disabling_output_tracking_drops_the_last_output() {
        let block = entity();
        block.set_last_output(Some(TextComponent::plain("old")));
        assert_ne!(block.last_output().to_string(), "");
        block.set_track_output(false);
        assert_eq!(block.last_output().to_string(), "");
    }

    #[test]
    fn the_packet_ordinals_match_vanilla() {
        for (ordinal, mode) in [
            (0, CommandBlockMode::Sequence),
            (1, CommandBlockMode::Auto),
            (2, CommandBlockMode::Redstone),
            (7, CommandBlockMode::Redstone),
        ] {
            assert_eq!(CommandBlockMode::from_packet_ordinal(ordinal), mode);
        }
        assert_eq!(CommandBlockMode::Sequence.packet_ordinal(), 0);
        assert_eq!(CommandBlockMode::Auto.packet_ordinal(), 1);
        assert_eq!(CommandBlockMode::Redstone.packet_ordinal(), 2);
    }

    #[test]
    fn the_mode_comes_from_the_block_identity() {
        assert_eq!(
            CommandBlockMode::for_block_key(&vanilla_blocks::COMMAND_BLOCK.key),
            CommandBlockMode::Redstone
        );
        assert_eq!(
            CommandBlockMode::for_block_key(&vanilla_blocks::REPEATING_COMMAND_BLOCK.key),
            CommandBlockMode::Auto
        );
        assert_eq!(
            CommandBlockMode::for_block_key(&vanilla_blocks::CHAIN_COMMAND_BLOCK.key),
            CommandBlockMode::Sequence
        );
    }
}
