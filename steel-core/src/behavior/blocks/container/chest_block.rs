//! Chest and trapped-chest behavior.
//!
//! Vanilla `ChestBlock` for the wooden chest family: `facing` from the player's
//! view, `waterlogged` from the placement fluid, the double-chest connection
//! rule that pairs two same-facing chests, the 27-slot menu backed by the chest
//! block entity, open/close sound events, and the comparator fullness signal.

use std::sync::{Arc, Weak};

use steel_macros::block_behavior;
use steel_registry::block_entity_type::BlockEntityTypeRef;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::blocks::properties::{BlockStateProperties, ChestType};
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_block_entity_types;
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId, Direction, Downcast as _};
use text_components::TextComponent;

use crate::behavior::block::{
    BlockBehavior, BlockEntityCreation, schedule_water_tick_if_waterlogged,
};
use crate::behavior::context::{BlockHitResult, BlockPlaceContext, InteractionResult};
use crate::behavior::{InventoryAccess, PlacementSource};
use crate::block_entity::{BLOCK_ENTITIES, BlockEntityTicker, entities::ChestBlockEntity};
use crate::inventory::container::calculate_redstone_signal_from_container;
use crate::inventory::lock::{ContainerLockGuard, ContainerRef};
use crate::inventory::menu::kinds::{chest, double_chest};
use crate::player::Player;
use crate::world::{ScheduledTickAccess, World};

/// Behavior for the wooden chest family (plain and trapped).
#[block_behavior]
pub struct ChestBlock {
    block: BlockRef,
    #[json_arg(sound_events, json = "open_sound")]
    open_sound: SoundEventRef,
    #[json_arg(sound_events, json = "close_sound")]
    close_sound: SoundEventRef,
}

impl ChestBlock {
    /// Returns the close sound configured for this chest block.
    #[must_use]
    pub const fn close_sound(&self) -> SoundEventRef {
        self.close_sound
    }

    /// Creates a new chest block behavior.
    #[must_use]
    pub const fn new(
        block: BlockRef,
        open_sound: SoundEventRef,
        close_sound: SoundEventRef,
    ) -> Self {
        Self {
            block,
            open_sound,
            close_sound,
        }
    }

    /// Vanilla `ChestBlock.getStateForPlacement`: `waterlogged` from the
    /// placement fluid and `facing` toward the placer.
    fn state_for_placement(&self, context: &BlockPlaceContext<'_>) -> BlockStateId {
        self.block
            .default_state()
            .set_value(
                &BlockStateProperties::WATERLOGGED,
                context.is_water_source(),
            )
            .set_value(
                &BlockStateProperties::HORIZONTAL_FACING,
                context.horizontal_direction().opposite(),
            )
            .set_value(&BlockStateProperties::CHEST_TYPE, ChestType::Single)
    }

    fn open_menu(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
        player: &Player,
    ) -> InteractionResult {
        let Some(container_ref) = world
            .get_block_entity(pos)
            .and_then(ContainerRef::from_block_entity)
        else {
            return InteractionResult::Pass;
        };

        // Vanilla tracks viewers on the block entity so a double chest opens both
        // halves and the comparator sees a consistent count.
        let block_entity = world.get_block_entity(pos);
        if let Some(chest) = block_entity
            .as_ref()
            .and_then(|entity| entity.downcast_ref::<ChestBlockEntity>())
        {
            // Vanilla `ChestBlock.useWithoutItem` → `unpackLootTable` before the
            // viewer count moves, so the opener sees the rolled contents.
            chest.unpack_loot();
        } else {
            world.play_block_sound(self.open_sound, pos, 0.5, 0.5, None);
        }

        let inventory = Arc::clone(&player.inventory);
        // Vanilla `ChestBlock.use`: a double chest opens one six-row menu backed
        // by both halves, a single chest opens three rows.
        let second_half = if state.get_value(&BlockStateProperties::CHEST_TYPE) == ChestType::Single
        {
            None
        } else {
            let (dx, dy, dz) = connected_direction(state).offset();
            world
                .get_block_entity(pos.offset(dx, dy, dz))
                .and_then(ContainerRef::from_block_entity)
        };
        player.open_menu(
            // Vanilla titles this menu with `container.chest`.
            TextComponent::plain("Chest"),
            move |context| match &second_half {
                Some(second) => double_chest(
                    inventory,
                    context.container_id,
                    container_ref,
                    second.clone(),
                ),
                None => chest(inventory, context.container_id, container_ref, 3),
            },
        );

        InteractionResult::Success
    }
}

impl BlockBehavior for ChestBlock {
    fn tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        if let Some(entity) = world.get_block_entity(pos) {
            if let Some(container) = entity.downcast_ref::<ChestBlockEntity>() {
                container.openers_counter().recheck_openers(world, pos, state, state.get_block());
            }
        }
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.state_for_placement(context))
    }

    fn set_placed_by(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _source: &PlacementSource<'_>,
    ) {
        connect_double_chest(state, world, pos);
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        direction: Direction,
        _neighbor_pos: BlockPos,
        neighbor_state: BlockStateId,
    ) -> BlockStateId {
        schedule_water_tick_if_waterlogged(state, world, pos);
        update_double_chest(state, direction, neighbor_state)
    }

    fn use_without_item(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &Player,
        _hit_result: &BlockHitResult,
        _inv: &mut InventoryAccess,
    ) -> InteractionResult {
        self.open_menu(world, pos, state, player)
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::CHEST,
            level,
            pos,
            state,
        ))
    }

    /// Vanilla `ChestBlock.getTicker` returns `ChestBlockEntity::lidAnimateTick`.
    ///
    /// Chests have no `open` block state, so without this ticker the lid never
    /// animates and the chest looks permanently shut even while it is open.
    fn get_block_entity_ticker(
        &self,
        _world: &Arc<World>,
        _state: BlockStateId,
        block_entity_type: BlockEntityTypeRef,
    ) -> Option<BlockEntityTicker> {
        // Vanilla registers the same ticker for the plain and trapped chest.
        if block_entity_type == &vanilla_block_entity_types::CHEST
            || block_entity_type == &vanilla_block_entity_types::TRAPPED_CHEST
        {
            Some(BlockEntityTicker::for_entity_tick(block_entity_type))
        } else {
            None
        }
    }

    fn has_analog_output_signal(&self, _state: BlockStateId) -> bool {
        true
    }

    fn get_analog_output_signal(
        &self,
        _state: BlockStateId,
        world: &World,
        pos: BlockPos,
        _direction: Direction,
    ) -> i32 {
        container_signal(world, pos)
    }
}

/// Behavior for trapped chests: identical to a chest, but opening emits a
/// redstone signal and the block only powers while a player is detected.
#[block_behavior]
pub struct TrappedChestBlock {
    block: BlockRef,
    #[json_arg(sound_events, json = "open_sound")]
    open_sound: SoundEventRef,
    #[json_arg(sound_events, json = "close_sound")]
    close_sound: SoundEventRef,
}

impl TrappedChestBlock {
    /// Returns the close sound configured for this trapped chest block.
    #[must_use]
    pub const fn close_sound(&self) -> SoundEventRef {
        self.close_sound
    }

    /// Creates a new trapped chest block behavior.
    #[must_use]
    pub const fn new(
        block: BlockRef,
        open_sound: SoundEventRef,
        close_sound: SoundEventRef,
    ) -> Self {
        Self {
            block,
            open_sound,
            close_sound,
        }
    }

    /// Vanilla `ChestBlock.getStateForPlacement`, shared with the plain chest.
    fn state_for_placement(&self, context: &BlockPlaceContext<'_>) -> BlockStateId {
        self.block
            .default_state()
            .set_value(
                &BlockStateProperties::WATERLOGGED,
                context.is_water_source(),
            )
            .set_value(
                &BlockStateProperties::HORIZONTAL_FACING,
                context.horizontal_direction().opposite(),
            )
            .set_value(&BlockStateProperties::CHEST_TYPE, ChestType::Single)
    }
}

impl BlockBehavior for TrappedChestBlock {
    fn tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        if let Some(entity) = world.get_block_entity(pos) {
            if let Some(chest) = entity.downcast_ref::<ChestBlockEntity>() {
                chest.openers_counter().recheck_openers(world, pos, state, state.get_block());
            }
        }
    }

    fn is_signal_source(&self, _state: BlockStateId, _context: crate::world::SignalQueryContext) -> bool {
        true
    }

    fn get_own_signal(&self, _state: BlockStateId, world: &dyn crate::world::LevelReader, pos: BlockPos,
        _context: crate::world::SignalQueryContext) -> i32 {
        world.get_block_entity(pos).and_then(|entity| entity.downcast_ref::<ChestBlockEntity>()
            .map(|chest| chest.openers_counter().openers().clamp(0, 15))).unwrap_or(0)
    }

    fn get_direct_signal(&self, state: BlockStateId, world: &dyn crate::world::LevelReader, pos: BlockPos,
        direction: Direction, context: crate::world::SignalQueryContext) -> i32 {
        if direction == Direction::Up { self.get_own_signal(state, world, pos, context) } else { 0 }
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.state_for_placement(context))
    }

    fn set_placed_by(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _source: &PlacementSource<'_>,
    ) {
        connect_double_chest(state, world, pos);
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        direction: Direction,
        _neighbor_pos: BlockPos,
        neighbor_state: BlockStateId,
    ) -> BlockStateId {
        schedule_water_tick_if_waterlogged(state, world, pos);
        update_double_chest(state, direction, neighbor_state)
    }

    fn use_without_item(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &Player,
        _hit_result: &BlockHitResult,
        _inv: &mut InventoryAccess,
    ) -> InteractionResult {
        ChestBlock::new(self.block, self.open_sound, self.close_sound).open_menu(world, pos, state, player)
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::TRAPPED_CHEST,
            level,
            pos,
            state,
        ))
    }

    fn get_block_entity_ticker(
        &self,
        _world: &Arc<World>,
        _state: BlockStateId,
        block_entity_type: BlockEntityTypeRef,
    ) -> Option<BlockEntityTicker> {
        if block_entity_type == &vanilla_block_entity_types::TRAPPED_CHEST
            || block_entity_type == &vanilla_block_entity_types::CHEST
        {
            Some(BlockEntityTicker::for_entity_tick(block_entity_type))
        } else {
            None
        }
    }

    fn has_analog_output_signal(&self, _state: BlockStateId) -> bool {
        true
    }

    fn get_analog_output_signal(
        &self,
        _state: BlockStateId,
        world: &World,
        pos: BlockPos,
        _direction: Direction,
    ) -> i32 {
        container_signal(world, pos)
    }
}

/// Comparator signal from the block entity inventory at `pos`.
fn container_signal(world: &dyn crate::world::LevelReader, pos: BlockPos) -> i32 {
    let Some(container_ref) = world
        .get_block_entity(pos)
        .and_then(ContainerRef::from_block_entity)
    else {
        return 0;
    };
    let guard = ContainerLockGuard::lock_all(&[&container_ref]);
    guard
        .get(container_ref.container_id())
        .map_or(0, |container| {
            calculate_redstone_signal_from_container(container)
        })
}

/// Vanilla `ChestBlock` connector: pairs this half with a same-facing chest on
/// a side perpendicular to the facing.
fn connect_double_chest(state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
    if state.get_value(&BlockStateProperties::CHEST_TYPE) != ChestType::Single {
        return;
    }
    let facing = state.get_value(&BlockStateProperties::HORIZONTAL_FACING);
    let left = facing.rotate_y_clockwise();
    let right = facing.rotate_y_counter_clockwise();
    for side in [left, right] {
        let other_pos = side.relative(pos);
        let other = world.get_block_state(other_pos);
        if other.get_block() != state.get_block() {
            continue;
        }
        if other.get_value(&BlockStateProperties::HORIZONTAL_FACING) != facing
            || other.get_value(&BlockStateProperties::CHEST_TYPE) != ChestType::Single
        {
            continue;
        }
        let this_type = if side == left {
            ChestType::Right
        } else {
            ChestType::Left
        };
        let other_type = match this_type {
            ChestType::Left => ChestType::Right,
            ChestType::Right => ChestType::Left,
            ChestType::Single => ChestType::Single,
        };
        world.set_block(
            pos,
            state.set_value(&BlockStateProperties::CHEST_TYPE, this_type),
            UpdateFlags::UPDATE_ALL,
        );
        world.set_block(
            other_pos,
            other.set_value(&BlockStateProperties::CHEST_TYPE, other_type),
            UpdateFlags::UPDATE_ALL,
        );
        return;
    }
}

/// Vanilla `ChestBlock.updateShape`: adopt or drop the double-chest half.
fn update_double_chest(
    mut state: BlockStateId,
    direction: Direction,
    neighbor_state: BlockStateId,
) -> BlockStateId {
    let neighbor_type = (neighbor_state.get_block() == state.get_block())
        .then(|| neighbor_state.try_get_value(&BlockStateProperties::CHEST_TYPE))
        .flatten();
    if let Some(neighbor_type) = &neighbor_type
        && direction.is_horizontal()
    {
        if state.get_value(&BlockStateProperties::CHEST_TYPE) == ChestType::Single
            && *neighbor_type != ChestType::Single
            && state.get_value(&BlockStateProperties::HORIZONTAL_FACING)
                == neighbor_state.get_value(&BlockStateProperties::HORIZONTAL_FACING)
            && connected_direction(neighbor_state) == direction.opposite()
        {
            let opposite_type = match neighbor_type {
                ChestType::Left => ChestType::Right,
                ChestType::Right => ChestType::Left,
                ChestType::Single => ChestType::Single,
            };
            state = state.set_value(&BlockStateProperties::CHEST_TYPE, opposite_type);
        }
    } else if connected_direction(state) == direction {
        state = state.set_value(&BlockStateProperties::CHEST_TYPE, ChestType::Single);
    }

    if neighbor_type.is_some()
        && state.get_value(&BlockStateProperties::CHEST_TYPE) != ChestType::Single
        && connected_direction(state) == direction
    {
        return neighbor_state.with_properties_of(state);
    }
    state
}

/// Vanilla `ChestBlock.getConnectedDirection`.
fn connected_direction(state: BlockStateId) -> Direction {
    let facing = state.get_value(&BlockStateProperties::HORIZONTAL_FACING);
    if state.get_value(&BlockStateProperties::CHEST_TYPE) == ChestType::Left {
        facing.rotate_y_clockwise()
    } else {
        facing.rotate_y_counter_clockwise()
    }
}
