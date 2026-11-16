//! Behaviors for the technical blocks that only own a block entity.
//!
//! Command blocks, jigsaw blocks, structure blocks and the sculk family have
//! server side state but no player-facing container: their behavior supplies the
//! placement state and the block entity factory, and the rest is driven by the
//! block entity and the redstone/game-event systems.

use std::sync::{Arc, Weak};

use rand::{Rng, RngExt};
use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{BlockStateProperties, FrontAndTop, StructureMode};
use steel_registry::{
    vanilla_block_entity_types, vanilla_blocks, vanilla_damage_types, vanilla_game_rules,
};
use steel_utils::Downcast as _;
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId};

use crate::behavior::BLOCK_BEHAVIORS;
use crate::behavior::block::{BlockBehavior, BlockEntityCreation};
use crate::behavior::blocks::vegetation::bonemealable::Bonemealable;
use crate::behavior::context::{
    BlockHitResult, BlockPlaceContext, InteractionResult, InventoryAccess, PlacementSource,
};
use crate::block_entity::entities::{
    CommandBlockBlockEntity, CommandBlockMode, CreakingHeartBlockEntity,
    SCULK_SENSOR_COOLDOWN_TICKS, SculkSensorBlockEntity, SculkShriekerBlockEntity,
    StructureBlockEntity, VaultBlockEntity, VaultUnlockOutcome,
};
use crate::block_entity::{BLOCK_ENTITIES, BlockEntity as _, BlockEntityTicker};
use crate::entity::Entity;
use crate::entity::damage::DamageSource;
use crate::player::Player;
use crate::world::LevelReader;
use crate::world::{SignalGetter as _, SignalQueryContext, World};

/// Maps a clicked facing onto vanilla's `front_and_top_to_orientation` for a
/// jigsaw placed against that face.
const fn orientation_for(front: steel_utils::Direction) -> FrontAndTop {
    use steel_utils::Direction;
    match front {
        Direction::Down => FrontAndTop::DownEast,
        Direction::Up => FrontAndTop::UpEast,
        Direction::North => FrontAndTop::NorthUp,
        Direction::South => FrontAndTop::SouthUp,
        Direction::West => FrontAndTop::WestUp,
        Direction::East => FrontAndTop::EastUp,
    }
}

/// Vanilla `CommandBlock` and its repeating/chain variants.
#[block_behavior]
pub struct CommandBlock {
    block: BlockRef,
}

impl CommandBlock {
    /// Creates a new command block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for CommandBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        // Vanilla `CommandBlock.getStateForPlacement` uses the *nearest looking*
        // direction, so a block placed under the player's crosshair faces them
        // instead of following the clicked face.
        Some(
            self.block
                .default_state()
                .set_value(
                    &BlockStateProperties::FACING,
                    context.get_nearest_looking_direction().opposite(),
                )
                .set_value(&BlockStateProperties::CONDITIONAL, false),
        )
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        // Vanilla `CommandBlock.newBlockEntity` sets the automatic flag that the
        // block's constructor carries, so a repeating block is automatic from the
        // moment its block entity exists.
        let creation = BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::COMMAND_BLOCK,
            level,
            pos,
            state,
        ));
        if let Some(entity) = creation.created()
            && let Some(block) = entity.downcast_ref::<CommandBlockBlockEntity>()
        {
            block.set_automatic(self.is_automatic());
        }
        creation
    }

    fn get_block_entity_ticker(
        &self,
        _world: &Arc<World>,
        _state: BlockStateId,
        block_entity_type: steel_registry::block_entity_type::BlockEntityTypeRef,
    ) -> Option<BlockEntityTicker> {
        // Vanilla ticks command blocks from `CommandBlock.tick` (a scheduled
        // block tick), not from a block entity ticker; the ticker only finishes
        // commands whose result arrives from the command queue.
        BlockEntityTicker::for_matching_entity_tick(
            block_entity_type,
            &vanilla_block_entity_types::COMMAND_BLOCK,
        )
    }

    fn set_placed_by(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        source: &crate::behavior::PlacementSource<'_>,
    ) {
        // Vanilla `CommandBlock.setPlacedBy`: a stack carrying block entity data
        // keeps that data untouched (the block entity was restored from it), so
        // the automatic flag and the output tracking are only applied to a
        // freshly placed block. Both branches then run `setPoweredAndUpdate`.
        let Some(entity) = world.get_block_entity(pos) else {
            return;
        };
        let Some(block) = entity.downcast_ref::<CommandBlockBlockEntity>().cloned() else {
            return;
        };

        let carries_block_entity_data = source.with_item(|item| {
            item.get(steel_registry::data_components::vanilla_components::BLOCK_ENTITY_DATA)
                .is_some()
        });
        if !carries_block_entity_data {
            block.set_track_output(world.get_game_rule(&vanilla_game_rules::SEND_COMMAND_FEEDBACK));
            block.set_automatic(self.is_automatic());
        }

        // Vanilla `setPoweredAndUpdate`: the placed block needs a tick when it
        // is placed into a powered circuit, even though no neighbour changed.
        let powered = world.has_neighbor_signal(pos);
        if powered == block.is_powered() {
            return;
        }
        block.set_powered(powered);
        if !powered {
            return;
        }
        if block.is_automatic() || block.mode() == CommandBlockMode::Sequence {
            return;
        }
        block.schedule_tick(world, 1);
        let _ = state;
    }

    fn handle_neighbor_changed(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _source_block: BlockRef,
        _moved_by_piston: bool,
    ) {
        // Vanilla `CommandBlock.neighborChanged` → `setPoweredAndUpdate`.
        let Some(entity) = world.get_block_entity(pos) else {
            return;
        };
        let Some(block) = entity.downcast_ref::<CommandBlockBlockEntity>().cloned() else {
            return;
        };
        let powered = world.has_neighbor_signal(pos);
        if powered == block.is_powered() {
            return;
        }
        block.set_powered(powered);
        if !powered {
            return;
        }
        if block.is_automatic() || block.mode() == CommandBlockMode::Sequence {
            return;
        }
        block.schedule_tick(world, 1);
    }

    fn tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        // Vanilla `CommandBlock.tick`.
        let Some(entity) = world.get_block_entity(pos) else {
            return;
        };
        let Some(block) = entity.downcast_ref::<CommandBlockBlockEntity>().cloned() else {
            return;
        };
        let was_condition_met = block.was_condition_met();
        let mode = block.mode();
        match mode {
            CommandBlockMode::Auto => {
                let _ = block.mark_condition_met();
                if was_condition_met {
                    self.execute(&block, world);
                } else {
                    block.clear_success_count_if_conditional();
                }
                if block.is_powered() || block.is_automatic() {
                    world.schedule_block_tick_default(pos, state.get_block(), 1);
                }
            }
            CommandBlockMode::Redstone => {
                if was_condition_met {
                    self.execute(&block, world);
                } else {
                    block.clear_success_count_if_conditional();
                }
            }
            // A sequence block runs from the chain of the block behind it.
            CommandBlockMode::Sequence => {}
        }
        world.update_neighbor_for_output_signal(pos, state.get_block());
    }

    fn has_analog_output_signal(&self, _state: BlockStateId) -> bool {
        true
    }

    fn get_analog_output_signal(
        &self,
        _state: BlockStateId,
        world: &World,
        pos: BlockPos,
        _direction: steel_utils::Direction,
    ) -> i32 {
        // Vanilla `CommandBlock.getAnalogOutputSignal` returns the success count,
        // which is also what a comparator reads.
        world
            .get_block_entity(pos)
            .and_then(|entity| entity.downcast_ref::<CommandBlockBlockEntity>().cloned())
            .map_or(0, |block| block.success_count())
    }
}

impl CommandBlock {
    /// The `automatic` flag of the vanilla `CommandBlock` variant.
    ///
    /// Vanilla keeps this as a block constructor argument; Steel derives it from
    /// the block identity, exactly as `CommandBlockEntity.getMode` does. Only the
    /// repeating variant is automatic.
    fn is_automatic(&self) -> bool {
        self.block.key == vanilla_blocks::REPEATING_COMMAND_BLOCK.key
    }

    /// Vanilla `CommandBlock.execute`.
    ///
    /// The command itself is queued; `execute_chain` runs right after, exactly as
    /// vanilla does it after `performCommand`.
    fn execute(&self, block: &CommandBlockBlockEntity, world: &Arc<World>) {
        let _ = block.try_perform_command(world);
        Self::execute_chain(block, world);
    }

    /// Vanilla `CommandBlock.executeChain`: walk the facing direction and run
    /// every powered (or automatic) chain block whose condition holds, stopping
    /// at the first failure or at `max_command_sequence_length` links.
    fn execute_chain(block: &CommandBlockBlockEntity, world: &Arc<World>) {
        let mut pos = block.get_block_pos();
        let mut facing: steel_utils::Direction = world
            .get_block_state(pos)
            .get_value(&BlockStateProperties::FACING);
        let mut remaining = world
            .get_game_rule(&vanilla_game_rules::MAX_COMMAND_SEQUENCE_LENGTH)
            .max(0);

        while remaining > 0 {
            remaining -= 1;
            pos = pos.relative(facing);
            let state = world.get_block_state(pos);
            if state.get_block().key != vanilla_blocks::CHAIN_COMMAND_BLOCK.key {
                break;
            }
            let Some(entity) = world.get_block_entity(pos) else {
                break;
            };
            let Some(next) = entity.downcast_ref::<CommandBlockBlockEntity>() else {
                break;
            };
            if next.mode() != CommandBlockMode::Sequence {
                break;
            }
            if next.is_powered() || next.is_automatic() {
                if next.mark_condition_met() {
                    if !next.try_perform_command(world) {
                        break;
                    }
                    world.update_neighbor_for_output_signal(pos, state.get_block());
                } else {
                    next.clear_success_count_if_conditional();
                }
            }
            // Vanilla continues in the direction each chain block faces.
            facing = state.get_value(&BlockStateProperties::FACING);
        }

        if remaining <= 0 {
            // Vanilla `CommandBlock.executeChain` warns once a chain hits the
            // `max_command_sequence_length` limit instead of silently stopping.
            log::warn!(
                "Command Block chain tried to execute more than {} steps!",
                world
                    .get_game_rule(&vanilla_game_rules::MAX_COMMAND_SEQUENCE_LENGTH)
                    .max(0)
            );
        }
    }
}

/// Vanilla `JigsawBlock`.
#[block_behavior]
pub struct JigsawBlock {
    block: BlockRef,
}

impl JigsawBlock {
    /// Creates a new jigsaw block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for JigsawBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        // Vanilla `JigsawBlock.getStateForPlacement` derives `orientation` from
        // the clicked face (front) with `north_up` as the top.
        Some(self.block.default_state().set_value(
            &BlockStateProperties::ORIENTATION,
            orientation_for(context.clicked_face().opposite()),
        ))
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::JIGSAW,
            level,
            pos,
            state,
        ))
    }

    /// Vanilla `JigsawBlock.useWithoutItem`: only a game master opens the jigsaw
    /// editor; everybody else gets `PASS`.
    fn use_without_item(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &Player,
        _hit_result: &BlockHitResult,
        _inv: &mut InventoryAccess,
    ) -> InteractionResult {
        if world.get_block_entity(pos).is_none() || !player.has_game_master_permissions() {
            return InteractionResult::Pass;
        }
        // Vanilla `player.openJigsawBlock(blockEntity)` opens a client screen;
        // `true` is the complete server answer.
        InteractionResult::Success
    }
}

/// Vanilla `StructureBlock`.
#[block_behavior]
pub struct StructureBlock {
    block: BlockRef,
}

impl StructureBlock {
    /// Creates a new structure block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for StructureBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::STRUCTURE_BLOCK,
            level,
            pos,
            state,
        ))
    }

    fn use_without_item(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &Player,
        _hit_result: &BlockHitResult,
        _inv: &mut InventoryAccess,
    ) -> InteractionResult {
        let Some(block_entity) = world.get_block_entity(pos) else {
            return InteractionResult::Pass;
        };
        let Some(structure) = block_entity.downcast_ref::<StructureBlockEntity>() else {
            return InteractionResult::Pass;
        };
        if structure.used_by(player) {
            InteractionResult::Success
        } else {
            InteractionResult::Pass
        }
    }

    fn set_placed_by(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        source: &PlacementSource<'_>,
    ) {
        let Some(player) = source.player() else {
            return;
        };
        let author = player.plain_text_name();
        let Some(block_entity) = world.get_block_entity(pos) else {
            return;
        };
        if let Some(structure) = block_entity.downcast_ref::<StructureBlockEntity>() {
            structure.created_by(&author);
        }
    }

    fn handle_neighbor_changed(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _source_block: BlockRef,
        _moved_by_piston: bool,
    ) {
        let Some(block_entity) = world.get_block_entity(pos) else {
            return;
        };
        let Some(structure) = block_entity.downcast_ref::<StructureBlockEntity>() else {
            return;
        };

        let should_trigger = world.has_neighbor_signal(pos);
        let is_powered = structure.is_powered();
        if should_trigger && !is_powered {
            structure.set_powered(true);
            if structure.mode() != StructureMode::Data {
                log::warn!(
                    "structure block at {pos:?} triggered {:?} but Steel has no runtime structure manager; the action was skipped",
                    structure.mode()
                );
            }
        } else if !should_trigger && is_powered {
            structure.set_powered(false);
        }
    }
}

/// Vanilla `CreakingHeartBlockEntity.removeCreaking`: destroying the heart takes
/// its creaking with it, which is the only way to kill a creaking.
fn kill_linked_creaking(world: &Arc<World>, pos: BlockPos) {
    let Some(block_entity) = world.get_block_entity(pos) else {
        return;
    };
    let Some(heart) = block_entity.downcast_ref::<CreakingHeartBlockEntity>() else {
        return;
    };
    let Some(creaking_uuid) = heart.creaking() else {
        return;
    };
    heart.set_creaking(None);
    if let Some(entity) = world.get_entity_by_uuid(&uuid::Uuid::from_u128(creaking_uuid)) {
        let damage = DamageSource::environment(&vanilla_damage_types::GENERIC_KILL);
        entity.hurt(world, &damage, f32::MAX);
    }
}

/// Vanilla `CreakingHeartBlock`.
#[block_behavior]
pub struct CreakingHeartBlock {
    block: BlockRef,
}

impl CreakingHeartBlock {
    /// Creates a new creaking heart block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for CreakingHeartBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        let state = self
            .block
            .default_state()
            .set_value(&BlockStateProperties::AXIS, context.clicked_face().axis());
        let heart_state = CreakingHeartBlockEntity::compute_heart_state(
            context.world,
            context.place_pos(),
            state,
        );
        Some(state.set_value(&BlockStateProperties::CREAKING_HEART_STATE, heart_state))
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::CREAKING_HEART,
            level,
            pos,
            state,
        ))
    }

    fn get_block_entity_ticker(
        &self,
        _world: &Arc<World>,
        _state: BlockStateId,
        block_entity_type: steel_registry::block_entity_type::BlockEntityTypeRef,
    ) -> Option<BlockEntityTicker> {
        BlockEntityTicker::for_matching_entity_tick(
            block_entity_type,
            &vanilla_block_entity_types::CREAKING_HEART,
        )
    }

    fn affect_neighbors_after_removal(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _moved_by_piston: bool,
    ) {
        kill_linked_creaking(world, pos);
    }

    fn has_analog_output_signal(&self, _state: BlockStateId) -> bool {
        true
    }

    fn get_analog_output_signal(
        &self,
        _state: BlockStateId,
        world: &World,
        pos: BlockPos,
        _direction: steel_utils::Direction,
    ) -> i32 {
        world
            .get_block_entity(pos)
            .and_then(|be| {
                be.downcast_ref::<CreakingHeartBlockEntity>()
                    .map(|h| h.analog_output_signal(world))
            })
            .unwrap_or(0)
    }
}

/// Vanilla `TrialSpawnerBlock`.
#[block_behavior]
pub struct TrialSpawnerBlock {
    block: BlockRef,
}

impl TrialSpawnerBlock {
    /// Creates a new trial spawner block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for TrialSpawnerBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::TRIAL_SPAWNER,
            level,
            pos,
            state,
        ))
    }

    fn get_block_entity_ticker(
        &self,
        _world: &Arc<World>,
        _state: BlockStateId,
        block_entity_type: steel_registry::block_entity_type::BlockEntityTypeRef,
    ) -> Option<BlockEntityTicker> {
        BlockEntityTicker::for_matching_entity_tick(
            block_entity_type,
            &vanilla_block_entity_types::TRIAL_SPAWNER,
        )
    }
}

/// Vanilla `VaultBlock`.
#[block_behavior]
pub struct VaultBlock {
    block: BlockRef,
}

impl VaultBlock {
    /// Creates a new vault block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for VaultBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state().set_value(
            &BlockStateProperties::HORIZONTAL_FACING,
            context.horizontal_direction().opposite(),
        ))
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::VAULT,
            level,
            pos,
            state,
        ))
    }

    fn get_block_entity_ticker(
        &self,
        _world: &Arc<World>,
        _state: BlockStateId,
        block_entity_type: steel_registry::block_entity_type::BlockEntityTypeRef,
    ) -> Option<BlockEntityTicker> {
        BlockEntityTicker::for_matching_entity_tick(
            block_entity_type,
            &vanilla_block_entity_types::VAULT,
        )
    }

    fn use_item_on(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &Player,
        _hand: steel_utils::types::InteractionHand,
        _hit_result: &BlockHitResult,
        inv: &mut InventoryAccess,
    ) -> InteractionResult {
        let held = inv.with_item(|stack| stack.clone());
        if held.is_empty() {
            return InteractionResult::TryEmptyHandInteraction;
        }
        let Some(be) = world.get_block_entity(pos) else {
            return InteractionResult::TryEmptyHandInteraction;
        };
        let Some(vault) = be.downcast_ref::<VaultBlockEntity>() else {
            return InteractionResult::TryEmptyHandInteraction;
        };
        match vault.try_insert_key(world, pos, state, player, &held) {
            VaultUnlockOutcome::Unlocked => {
                if !player.has_infinite_materials() {
                    inv.with_item(|stack| stack.shrink(1));
                }
                InteractionResult::Success
            }
            VaultUnlockOutcome::AlreadyRewarded | VaultUnlockOutcome::WrongKey => {
                InteractionResult::TryEmptyHandInteraction
            }
        }
    }
}

fn step_sculk_sensor_phase(
    state: BlockStateId,
    world: &Arc<World>,
    pos: BlockPos,
    calibrated: bool,
) {
    use steel_protocol::packets::game::SoundSource;
    use steel_registry::blocks::properties::SculkSensorPhase;
    use steel_registry::sound_events;

    let Some(phase) = state.try_get_value(&BlockStateProperties::SCULK_SENSOR_PHASE) else {
        return;
    };
    let block = state.get_block();
    match phase {
        SculkSensorPhase::Active => {
            let next = state
                .set_value(
                    &BlockStateProperties::SCULK_SENSOR_PHASE,
                    SculkSensorPhase::Cooldown,
                )
                .set_value(&BlockStateProperties::POWER, 0);
            world.set_block(pos, next, UpdateFlags::UPDATE_ALL);
            world.schedule_block_tick_default(pos, block, SCULK_SENSOR_COOLDOWN_TICKS);
            world.update_neighbors_at(pos, block);
            world.update_neighbors_at(pos.below(), block);
            world.update_neighbor_for_output_signal(pos, block);
            if !state
                .try_get_value(&BlockStateProperties::WATERLOGGED)
                .unwrap_or(false)
            {
                world.play_sound(
                    &sound_events::BLOCK_SCULK_SENSOR_CLICKING_STOP,
                    SoundSource::Blocks,
                    pos,
                    1.0,
                    if calibrated { 1.5 } else { 1.0 },
                    None,
                );
            }
        }
        SculkSensorPhase::Cooldown => {
            world.set_block(
                pos,
                state.set_value(
                    &BlockStateProperties::SCULK_SENSOR_PHASE,
                    SculkSensorPhase::Inactive,
                ),
                UpdateFlags::UPDATE_ALL,
            );
        }
        SculkSensorPhase::Inactive => {}
    }
}

fn sculk_sensor_comparator_signal(world: &World, pos: BlockPos, state: BlockStateId) -> i32 {
    use steel_registry::blocks::properties::SculkSensorPhase;
    if state.try_get_value(&BlockStateProperties::SCULK_SENSOR_PHASE)
        != Some(SculkSensorPhase::Active)
    {
        return 0;
    }
    world
        .get_block_entity(pos)
        .and_then(|be| {
            be.downcast_ref::<SculkSensorBlockEntity>()
                .map(SculkSensorBlockEntity::last_vibration_frequency)
        })
        .unwrap_or(0)
}

/// Vanilla `SculkSensorBlock`.
#[block_behavior]
pub struct SculkSensorBlock {
    block: BlockRef,
}

impl SculkSensorBlock {
    /// Creates a new sculk sensor block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for SculkSensorBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        let waterlogged = context
            .world
            .get_block_state(context.place_pos())
            .get_block()
            == &vanilla_blocks::WATER;
        Some(
            self.block
                .default_state()
                .set_value(&BlockStateProperties::WATERLOGGED, waterlogged),
        )
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::SCULK_SENSOR,
            level,
            pos,
            state,
        ))
    }

    fn tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        step_sculk_sensor_phase(state, world, pos, false);
    }

    fn is_signal_source(&self, _state: BlockStateId, _context: SignalQueryContext) -> bool {
        true
    }

    fn get_own_signal(
        &self,
        state: BlockStateId,
        _world: &dyn LevelReader,
        _pos: BlockPos,
        _context: SignalQueryContext,
    ) -> i32 {
        i32::from(state.get_value(&BlockStateProperties::POWER))
    }

    fn get_direct_signal(
        &self,
        state: BlockStateId,
        _world: &dyn LevelReader,
        _pos: BlockPos,
        direction: steel_utils::Direction,
        _context: SignalQueryContext,
    ) -> i32 {
        if direction == steel_utils::Direction::Up {
            i32::from(state.get_value(&BlockStateProperties::POWER))
        } else {
            0
        }
    }

    fn has_analog_output_signal(&self, _state: BlockStateId) -> bool {
        true
    }

    fn get_analog_output_signal(
        &self,
        state: BlockStateId,
        world: &World,
        pos: BlockPos,
        _direction: steel_utils::Direction,
    ) -> i32 {
        sculk_sensor_comparator_signal(world, pos, state)
    }
}

/// Vanilla `CalibratedSculkSensorBlock`.
#[block_behavior]
pub struct CalibratedSculkSensorBlock {
    block: BlockRef,
}

impl CalibratedSculkSensorBlock {
    /// Creates a new calibrated sculk sensor block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for CalibratedSculkSensorBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        let waterlogged = context
            .world
            .get_block_state(context.place_pos())
            .get_block()
            == &vanilla_blocks::WATER;
        // Vanilla `CalibratedSculkSensorBlock`: the amethyst input side faces the player.
        Some(
            self.block
                .default_state()
                .set_value(&BlockStateProperties::WATERLOGGED, waterlogged)
                .set_value(
                    &BlockStateProperties::HORIZONTAL_FACING,
                    context.horizontal_direction(),
                ),
        )
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::CALIBRATED_SCULK_SENSOR,
            level,
            pos,
            state,
        ))
    }

    fn tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        step_sculk_sensor_phase(state, world, pos, true);
    }

    fn is_signal_source(&self, _state: BlockStateId, _context: SignalQueryContext) -> bool {
        true
    }

    fn get_signal(
        &self,
        state: BlockStateId,
        _world: &dyn LevelReader,
        _pos: BlockPos,
        direction: steel_utils::Direction,
        _context: SignalQueryContext,
    ) -> i32 {
        // Vanilla `CalibratedSculkSensorBlock.getSignal`: emits on every face
        // EXCEPT the calibrated input face (`facing`).
        if direction == state.get_value(&BlockStateProperties::HORIZONTAL_FACING) {
            0
        } else {
            i32::from(state.get_value(&BlockStateProperties::POWER))
        }
    }

    fn get_direct_signal(
        &self,
        state: BlockStateId,
        _world: &dyn LevelReader,
        _pos: BlockPos,
        direction: steel_utils::Direction,
        _context: SignalQueryContext,
    ) -> i32 {
        if direction == steel_utils::Direction::Up {
            i32::from(state.get_value(&BlockStateProperties::POWER))
        } else {
            0
        }
    }

    fn has_analog_output_signal(&self, _state: BlockStateId) -> bool {
        true
    }

    fn get_analog_output_signal(
        &self,
        state: BlockStateId,
        world: &World,
        pos: BlockPos,
        _direction: steel_utils::Direction,
    ) -> i32 {
        sculk_sensor_comparator_signal(world, pos, state)
    }
}

/// Vanilla `SculkCatalystBlock`.
#[block_behavior]
pub struct SculkCatalystBlock {
    block: BlockRef,
}

impl SculkCatalystBlock {
    /// Creates a new sculk catalyst block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for SculkCatalystBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::SCULK_CATALYST,
            level,
            pos,
            state,
        ))
    }

    fn tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        if state.get_value(&BlockStateProperties::BLOOM) {
            world.set_block(
                pos,
                state.set_value(&BlockStateProperties::BLOOM, false),
                UpdateFlags::UPDATE_ALL,
            );
        }
    }
}

/// Vanilla `SculkShriekerBlock`.
#[block_behavior]
pub struct SculkShriekerBlock {
    block: BlockRef,
}

impl SculkShriekerBlock {
    /// Creates a new sculk shrieker block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for SculkShriekerBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        let waterlogged = context
            .world
            .get_block_state(context.place_pos())
            .get_block()
            == &vanilla_blocks::WATER;
        Some(
            self.block
                .default_state()
                .set_value(&BlockStateProperties::WATERLOGGED, waterlogged),
        )
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::SCULK_SHRIEKER,
            level,
            pos,
            state,
        ))
    }

    fn get_block_entity_ticker(
        &self,
        _world: &Arc<World>,
        _state: BlockStateId,
        block_entity_type: steel_registry::block_entity_type::BlockEntityTypeRef,
    ) -> Option<BlockEntityTicker> {
        BlockEntityTicker::for_matching_entity_tick(
            block_entity_type,
            &vanilla_block_entity_types::SCULK_SHRIEKER,
        )
    }

    fn tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        if !state.get_value(&BlockStateProperties::SHRIEKING) {
            return;
        }
        world.set_block(
            pos,
            state.set_value(&BlockStateProperties::SHRIEKING, false),
            UpdateFlags::UPDATE_ALL,
        );
        if let Some(shrieker) = world
            .get_block_entity(pos)
            .and_then(|entity| entity.downcast_ref::<SculkShriekerBlockEntity>().cloned())
        {
            shrieker.try_respond(world, pos);
        }
    }

    fn affect_neighbors_after_removal(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _moved_by_piston: bool,
    ) {
        if !state.get_value(&BlockStateProperties::SHRIEKING) {
            return;
        }
        if let Some(shrieker) = world
            .get_block_entity(pos)
            .and_then(|be| be.downcast_ref::<SculkShriekerBlockEntity>().cloned())
        {
            shrieker.try_respond(world, pos);
        }
    }

    fn step_on(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos, entity: &dyn Entity) {
        if state.get_value(&BlockStateProperties::SHRIEKING) {
            return;
        }
        if entity.as_player().is_none()
            && entity.controlling_passenger().is_none()
            && entity.projectile_owner().is_none()
        {
            return;
        }
        let Some(player) = SculkShriekerBlockEntity::try_get_player(entity) else {
            return;
        };
        if let Some(shrieker) = world
            .get_block_entity(pos)
            .and_then(|be| be.downcast_ref::<SculkShriekerBlockEntity>().cloned())
        {
            shrieker.try_shriek(world, pos, Some(&player));
        }
    }
}

/// Attempts vanilla's moss feature makes to find a valid position.
const MOSS_PLACEMENT_ATTEMPTS: usize = 8;

/// Vanilla `BonemealableFeaturePlacerBlock` (moss and pale moss blocks): bonemeal
/// scatters the block's vegetation feature over the surrounding blocks.
///
/// Vanilla reads the feature key from the block's `FEATURE` data component
/// (`moss_vegetation` for moss, `pale_moss_vegetation` for pale moss) and asks
/// the feature system to place it inside a 3x3x3 area, skipping the source block.
/// Steel places the simple vegetation set directly: vanilla's features are thin
/// wrappers that put one random plant on a random supported position.
#[block_behavior]
pub struct BonemealableFeaturePlacerBlock {
    block: BlockRef,
}

impl BonemealableFeaturePlacerBlock {
    /// Creates a new moss block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for BonemealableFeaturePlacerBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    fn as_bonemealable(&self) -> Option<&dyn Bonemealable> {
        Some(self)
    }
}

impl Bonemealable for BonemealableFeaturePlacerBlock {
    fn is_valid_bonemeal_target(
        &self,
        _state: BlockStateId,
        world: &dyn LevelReader,
        pos: BlockPos,
    ) -> bool {
        // Vanilla `BonemealableFeaturePlacerBlock.isValidBonemealTarget`: any
        // position with room for the feature is fine; the feature's own placement
        // check decides the rest.
        world.get_block_state(pos.above()).is_air()
    }

    fn perform_bonemeal(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        rng: &mut dyn rand::Rng,
        pos: BlockPos,
    ) {
        // Vanilla's moss feature places a *single* random plant: the configured
        // feature picks one weighted entry and tries to put it on a random
        // position inside the 3x3x3 area around the block.
        let plant = self.vegetation_block(rng);
        let plant_state = plant.default_state();
        let behavior = BLOCK_BEHAVIORS.get_behavior(plant);
        for _ in 0..MOSS_PLACEMENT_ATTEMPTS {
            let dx = rng.random_range(-1..=1);
            let dy = rng.random_range(-1..=1);
            let dz = rng.random_range(-1..=1);
            let candidate = pos.offset(dx, dy, dz);
            if candidate == pos {
                continue;
            }
            if world.get_block_state(candidate).is_air()
                && behavior.can_survive(plant_state, world, candidate)
            {
                world.set_block(candidate, plant_state, UpdateFlags::UPDATE_ALL);
                return;
            }
        }
    }
}

impl BonemealableFeaturePlacerBlock {
    /// Vanilla `moss_vegetation` / `pale_moss_vegetation` feature contents: the
    /// weighted plant set the feature scatters.
    fn vegetation_block(&self, rng: &mut dyn Rng) -> BlockRef {
        let pale = self.block.key == vanilla_blocks::PALE_MOSS_BLOCK.key;
        // `vanilla_blocks::X` are `Block` statics, and `BlockRef` is
        // `&'static Block`, so each entry is borrowed here.
        let choices: [BlockRef; 4] = if pale {
            [
                &vanilla_blocks::PALE_HANGING_MOSS,
                &vanilla_blocks::SHORT_GRASS,
                &vanilla_blocks::PALE_MOSS_CARPET,
                &vanilla_blocks::TALL_GRASS,
            ]
        } else {
            [
                &vanilla_blocks::SHORT_GRASS,
                &vanilla_blocks::MOSS_CARPET,
                &vanilla_blocks::TALL_GRASS,
                &vanilla_blocks::AZALEA,
            ]
        };
        let index = rng.random_range(0..choices.len());
        choices[index]
    }
}
