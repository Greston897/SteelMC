//! Vanilla command block minecart.
//!
//! A `MinecartCommandBlock` stores a command string plus the last output it
//! produced. Like vanilla, the cart only runs its command when an activator rail
//! powers it (`activateMinecart`), at most once every four ticks, through the
//! shared command queue with console permissions and no chat output.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::vanilla_entity_data::MinecartCommandBlockEntityData;
use steel_utils::BlockStateId;
use steel_utils::locks::SyncMutex;
use steel_utils::nbt::NbtValueInput as _;
use steel_utils::types::InteractionHand;
use steel_utils::{BlockPos, DowncastType, DowncastTypeKey};
use text_components::TextComponent;
use steel_registry::vanilla_game_rules::COMMAND_BLOCKS_WORK;
use tokio::sync::oneshot::Receiver;
use tokio::sync::oneshot::error::TryRecvError::Closed;
use tokio::sync::oneshot::error::TryRecvError::Empty;

use super::abstract_minecart::{AbstractMinecart, AbstractMinecartBase};
use crate::behavior::InteractionResult;
use crate::command::BackgroundCommandResult;
use crate::entity::{
    Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, EntitySyncedData, SharedEntity,
};
use crate::player::Player;
use crate::world::World;

/// Vanilla `MinecartCommandBlock` re-runs its command every `4` ticks.
const COMMAND_BLOCK_MINECART_TICK_DELAY: i32 = 4;

#[entity_behavior(class = "MinecartCommandBlock")]
/// Vanilla command block minecart.
pub struct CommandBlockMinecartEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    entity_data: SyncMutex<MinecartCommandBlockEntityData>,
    minecart_base: AbstractMinecartBase,
    state: SyncMutex<CommandBlockMinecartState>,
    /// Reply channel of a command still queued on the server.
    pending:
        SyncMutex<Option<Receiver<BackgroundCommandResult>>>,
}

#[derive(Debug)]
struct CommandBlockMinecartState {
    /// The command to run, mirrored into the synced `command_name` field.
    command: String,
    /// Vanilla `MinecartCommandBlock.lastActivated`: the `tickCount` of the last
    /// activator-rail trigger.
    last_activated: i32,
    /// Whether the cart is currently on a powered activator rail.
    powered: bool,
    /// Vanilla `BaseCommandBlock.trackOutput`.
    track_output: bool,
}

impl Default for CommandBlockMinecartState {
    fn default() -> Self {
        Self {
            command: String::new(),
            last_activated: -COMMAND_BLOCK_MINECART_TICK_DELAY,
            powered: false,
            track_output: true,
        }
    }
}

// SAFETY: This key is owned by Steel and uniquely identifies `CommandBlockMinecartEntity`.
unsafe impl DowncastType for CommandBlockMinecartEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/command_block_minecart");
}

impl CommandBlockMinecartEntity {
    /// Creates a new command block minecart entity.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            entity_data: SyncMutex::new(MinecartCommandBlockEntityData::new()),
            minecart_base: AbstractMinecartBase::new(),
            state: SyncMutex::new(CommandBlockMinecartState::default()),
            pending: SyncMutex::new(None),
        }
    }

    /// Creates a command block minecart entity from saved data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            entity_data: SyncMutex::new(MinecartCommandBlockEntityData::new()),
            minecart_base: AbstractMinecartBase::new(),
            state: SyncMutex::new(CommandBlockMinecartState::default()),
            pending: SyncMutex::new(None),
        }
    }

    /// The current command string.
    #[must_use]
    pub fn command(&self) -> String {
        self.state.lock().command.clone()
    }

    /// Replaces the command, like vanilla `MinecartCommandBlock.setCommand`.
    pub fn set_command(&self, command: impl Into<String>) {
        let command = command.into();
        self.state.lock().command.clone_from(&command);
        self.entity_data
            .lock()
            .minecart_command_block_mut()
            .id_command_name
            .set(command);
    }

    /// Vanilla `MinecartCommandBlock.lastActivated`, the `tickCount` of the last
    /// activator-rail run.
    #[must_use]
    pub fn last_activated(&self) -> i32 {
        self.state.lock().last_activated
    }

    /// Vanilla `BaseCommandBlock.setTrackOutput`.
    pub fn set_track_output(&self, track_output: bool) {
        self.state.lock().track_output = track_output;
    }

    /// Vanilla `BaseCommandBlock.isTrackOutput`.
    #[must_use]
    pub fn tracks_output(&self) -> bool {
        self.state.lock().track_output
    }

    /// Mirrors vanilla `MinecartCommandBlock`'s `lastOutput` component.
    pub fn set_last_output(&self, output: TextComponent) {
        self.entity_data
            .lock()
            .minecart_command_block_mut()
            .id_last_output
            .set(Box::new(output));
    }

    /// Vanilla `MinecartCommandBlock.activateMinecart`: the command runs only
    /// while an activator rail powers the cart, and at most once every four
    /// ticks (`tickCount - lastActivated >= 4`).
    fn run_command(&self) {
        let (command, tick_count) = {
            let state = self.state.lock();
            if !state.powered || state.command.is_empty() {
                return;
            }
            (state.command.clone(), self.base.tick_count())
        };
        if tick_count - self.state.lock().last_activated < COMMAND_BLOCK_MINECART_TICK_DELAY {
            return;
        }
        self.state.lock().last_activated = tick_count;

        let Some(world) = self.level() else {
            return;
        };
        // Vanilla `BaseCommandBlock.performCommand` is shared with the block, so
        // the "Searge" easter egg and the disabled-rule path behave identically.
        if command.eq_ignore_ascii_case("Searge") {
            self.set_last_output(TextComponent::plain("#itzlipofutzli"));
            return;
        }
        if !world.get_game_rule(&COMMAND_BLOCKS_WORK) {
            return;
        }
        let Some(server) = world.server() else {
            return;
        };
        let Ok(receiver) = server.execute_command_for_block(Arc::clone(&world), command) else {
            return;
        };
        *self.pending.lock() = Some(receiver);
    }

    /// Polls a running command and stores its last output, like vanilla
    /// `MinecartCommandBlock`'s `lastOutput` component.
    fn poll_command(&self) {
        let mut pending = self.pending.lock();
        let Some(receiver) = pending.as_mut() else {
            return;
        };
        match receiver.try_recv() {
            Ok(result) => {
                *pending = None;
                drop(pending);
                if result.output_allowed
                    && self.tracks_output()
                    && let Some(output) = result.output
                {
                    self.set_last_output(TextComponent::plain(output));
                }
            }
            Err(Empty) => {}
            Err(Closed) => {
                *pending = None;
            }
        }
    }
}

impl Entity for CommandBlockMinecartEntity {
    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn movement_emission(&self) -> EntityMovementEmission {
        EntityMovementEmission::Events
    }

    fn synced_data(&self) -> Option<&dyn EntitySyncedData> {
        Some(&self.entity_data)
    }

    fn interact(
        &self,
        _player: &Player,
        _hand: InteractionHand,
        _location: DVec3,
    ) -> InteractionResult {
        InteractionResult::Pass
    }

    fn tick(self: Arc<Self>) {
        if let Some(minecart) = self.as_abstract_minecart() {
            minecart.tick_minecart(&(Arc::clone(&self) as SharedEntity));
        }
        self.poll_command();
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        let state = self.state.lock();
        nbt.insert("Command", state.command.clone());
        nbt.insert("Powered", u8::from(state.powered));
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        if let Some(command) = nbt.string("Command") {
            self.set_command(command.to_str().into_owned());
        }
        if let Some(powered) = nbt.byte_value("Powered") {
            self.state.lock().powered = powered != 0;
        }
    }
}

impl AbstractMinecart for CommandBlockMinecartEntity {
    fn minecart_base(&self) -> &AbstractMinecartBase {
        &self.minecart_base
    }

    fn activate_minecart(&self, _world: &World, _pos: BlockPos, powered: bool) {
        self.state.lock().powered = powered;
        if powered {
            self.run_command();
        }
    }

    fn custom_display_block(&self) -> Option<BlockStateId> {
        *self
            .entity_data
            .lock()
            .abstract_minecart()
            .id_custom_display_block
            .get()
    }

    fn set_custom_display_block(&self, value: Option<BlockStateId>) {
        self.entity_data
            .lock()
            .abstract_minecart_mut()
            .id_custom_display_block
            .set(value);
    }

    fn display_offset(&self) -> i32 {
        *self
            .entity_data
            .lock()
            .abstract_minecart()
            .id_display_offset
            .get()
    }

    fn set_display_offset(&self, value: i32) {
        self.entity_data
            .lock()
            .abstract_minecart_mut()
            .id_display_offset
            .set(value);
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::init_vanilla_registry;
    use steel_registry::vanilla_entities;

    use super::*;

    fn cart() -> CommandBlockMinecartEntity {
        CommandBlockMinecartEntity::new(
            &vanilla_entities::COMMAND_BLOCK_MINECART,
            1,
            DVec3::new(0.5, 4.0, 0.5),
            Weak::new(),
        )
    }

    #[test]
    fn new_command_block_minecarts_have_no_command() {
        init_vanilla_registry();
        let cart = cart();
        assert!(cart.command().is_empty());
    }

    #[test]
    fn setting_the_command_mirrors_it_into_synced_data() {
        init_vanilla_registry();
        let cart = cart();
        cart.set_command("say hi");
        assert_eq!(cart.command(), "say hi");
        assert_eq!(
            *cart
                .entity_data
                .lock()
                .minecart_command_block_mut()
                .id_command_name
                .get(),
            "say hi"
        );
    }

    #[test]
    fn command_block_minecart_round_trips_its_command() {
        init_vanilla_registry();
        let cart = cart();
        cart.set_command("time set day");
        let mut nbt = NbtCompound::new();
        cart.save_additional(&mut nbt);
        let mut encoded = Vec::new();
        nbt.write(&mut encoded);
        assert!(!encoded.is_empty());
        assert_eq!(
            nbt.string("Command").map(ToString::to_string),
            Some("time set day".to_owned())
        );
    }
}
