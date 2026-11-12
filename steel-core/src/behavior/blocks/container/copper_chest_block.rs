//! Copper chest behavior.
//!
//! Vanilla `ChestBlock` for the copper chest family: facing from the player's
//! view, `waterlogged` from the placement fluid, and the vanilla double-chest
//! connection rule that pairs two chests facing the same direction.
//!
//! Remaining work (tracked in `PARITY-26.3-RU.md`): the chest block entity with
//! its loot table and lock, the combined double-chest menu, viewer tracking and
//! open/close block events, comparator fullness, pathfinding rejection, and the
//! copper-block conversion/normalization rules.

use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::blocks::properties::{BlockStateProperties, ChestType};
use steel_registry::vanilla_block_tags::BlockTag;
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId, Direction};

use std::sync::Arc;

use crate::behavior::BlockPlaceContext;
use crate::behavior::PlacementSource;
use crate::behavior::block::{BlockBehavior, schedule_water_tick_if_waterlogged};
use crate::behavior::blocks::{WeatherState, WeatheringCopper};
use crate::block_entity::entities::ChestBlockEntity;
use crate::world::{ScheduledTickAccess, World};

/// Copper chest behaviour
#[block_behavior]
pub struct CopperChestBlock {
    block: BlockRef,
}

impl CopperChestBlock {
    /// creates a new chest behaviour
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

/// Vanilla `ChestBlock.getStateForPlacement`: `waterlogged` from the placement
/// fluid, `facing` from the player's horizontal look direction, and a single
/// chest half; the neighbour update connects the other half afterwards.
fn chest_state_for_placement(block: BlockRef, context: &BlockPlaceContext<'_>) -> BlockStateId {
    block
        .default_state()
        .set_value(
            &BlockStateProperties::WATERLOGGED,
            context.is_water_source(),
        )
        .set_value(
            &BlockStateProperties::FACING,
            context.horizontal_direction(),
        )
        .set_value(&BlockStateProperties::CHEST_TYPE, ChestType::Single)
}

/// Vanilla `ChestBlock` connector: pairs this half with a same-facing chest on
/// the side the player did not place against.
///
/// Mirrors the vanilla placement flow where the freshly placed half inspects its
/// two horizontal neighbours, and the matched neighbour is re-typed through its
/// own `updateShape`. Both halves keep their own `waterlogged` state.
fn connect_double_chest(state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
    if state.get_value(&BlockStateProperties::CHEST_TYPE) != ChestType::Single {
        return;
    }
    let facing = state.get_value(&BlockStateProperties::FACING);
    // Vanilla `ChestBlock.getConnectedDirection` for a single half: the two
    // sides perpendicular to `facing`.
    let left = facing.rotate_y_clockwise();
    let right = facing.rotate_y_counter_clockwise();
    for side in [left, right] {
        let other_pos = side.relative(pos);
        let other = world.get_block_state(other_pos);
        if !other.get_block().has_tag(&BlockTag::COPPER_CHESTS) {
            continue;
        }
        if other.get_value(&BlockStateProperties::FACING) != facing
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

impl BlockBehavior for CopperChestBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(chest_state_for_placement(self.block, context))
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
        update_shape(state, direction, neighbor_state)
    }
}

/// Copper chest behaviour
#[block_behavior]
pub struct WeatheringCopperChestBlock {
    block: BlockRef,
    #[json_arg(r#enum = "WeatherState", json = "weather_state")]
    weathering: WeatheringCopper,
}

impl WeatheringCopperChestBlock {
    /// creates a new chest behaviour
    #[must_use]
    pub const fn new(block: BlockRef, weather_state: WeatherState) -> Self {
        Self {
            block,
            weathering: WeatheringCopper::new(weather_state),
        }
    }
}

impl BlockBehavior for WeatheringCopperChestBlock {
    fn random_tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        // Vanilla `WeatheringCopperChestBlock.changeOverTime` only weathers the
        // left half, so a double chest oxidises once instead of twice, and it
        // skips chests someone is looking into so an open chest never changes
        // under the player.
        if state.get_value(&BlockStateProperties::CHEST_TYPE) == ChestType::Right {
            return;
        }
        if world
            .get_block_entity(pos)
            .and_then(|entity| entity.downcast_ref::<ChestBlockEntity>())
            .is_some_and(|chest| chest.openers_counter().openers() > 0)
        {
            return;
        }
        self.weathering.change_over_time(state, world, pos);
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(chest_state_for_placement(self.block, context))
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
        update_shape(state, direction, neighbor_state)
    }
}

fn update_shape(
    mut state: BlockStateId,
    direction: Direction,
    neighbor_state: BlockStateId,
) -> BlockStateId {
    let neighbor_type = neighbor_state
        .get_block()
        .has_tag(&BlockTag::COPPER_CHESTS)
        .then(|| neighbor_state.try_get_value(&BlockStateProperties::CHEST_TYPE))
        .flatten();
    if let Some(neighbor_type) = &neighbor_type
        && direction.is_horizontal()
    {
        if state.get_value(&BlockStateProperties::CHEST_TYPE) == ChestType::Single
            && *neighbor_type != ChestType::Single
            && state.get_value(&BlockStateProperties::FACING)
                == neighbor_state.get_value(&BlockStateProperties::FACING)
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

fn connected_direction(state: BlockStateId) -> Direction {
    let facing = state.get_value(&BlockStateProperties::FACING);
    if state.get_value(&BlockStateProperties::CHEST_TYPE) == ChestType::Left {
        facing.rotate_y_clockwise()
    } else {
        facing.rotate_y_counter_clockwise()
    }
}
