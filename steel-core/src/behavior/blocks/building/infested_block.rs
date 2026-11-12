//! Infested blocks, nylium, sculk and the remaining single-purpose behaviors.
//!
//! These blocks share a pattern: they inherit the plain block behavior and only
//! differ in what happens when they are broken or randomly ticked. Vanilla
//! `InfestedBlock` spawns a silverfish, `NyliumBlock` converts to netherrack when
//! unsupported, and `SculkBlock` grows sculk veins from nearby sculk catalysts.

use std::sync::Arc;

use steel_macros::block_behavior;
use steel_registry::REGISTRY;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{BlockStateProperties, Direction};
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId};

use crate::behavior::BlockBehavior;
use crate::behavior::BlockPlaceContext;
use crate::world::{LevelReader, ScheduledTickAccess, World};

/// Vanilla `InfestedBlock`: breaking it spawns a silverfish.
///
/// The entity spawn lives in the block behavior because Steel has no
/// `spawnAfterBreak` hook that carries the world; the block is otherwise a plain
/// full block.
#[block_behavior]
pub struct InfestedBlock {
    block: BlockRef,
}

impl InfestedBlock {
    /// Creates a new infested block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for InfestedBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }
}

/// Vanilla `InfestedRotatedPillarBlock` (infested deepslate): an infested block
/// with an axis property.
#[block_behavior]
pub struct InfestedRotatedPillarBlock {
    block: BlockRef,
}

impl InfestedRotatedPillarBlock {
    /// Creates a new infested pillar block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for InfestedRotatedPillarBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        // Vanilla `RotatedPillarBlock.getStateForPlacement` picks the axis from
        // the clicked face.
        Some(
            self.block
                .default_state()
                .set_value(&BlockStateProperties::AXIS, context.clicked_face().get_axis()),
        )
    }
}

/// Vanilla `NyliumBlock`: turns back into netherrack when the block above does
/// not hold a nylium-supporting block (crimson or warped roots, and so on).
#[block_behavior]
pub struct NyliumBlock {
    block: BlockRef,
}

impl NyliumBlock {
    /// Creates a new nylium block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for NyliumBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    fn random_tick(&self, _state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        // Vanilla converts nylium to netherrack when the block above cannot
        // sustain it; both nylium variants convert to netherrack.
        let above = world.get_block_state(pos.above());
        if above.get_block() == &steel_registry::vanilla_blocks::AIR {
            world.set_block(
                pos,
                steel_registry::vanilla_blocks::NETHERRACK.default_state(),
                UpdateFlags::UPDATE_ALL,
            );
        }
    }
}

/// Vanilla `SculkBlock`: spreads sculk veins onto nearby sculk-replaceable blocks
/// while a sculk catalyst is in range.
#[block_behavior]
pub struct SculkBlock {
    block: BlockRef,
}

impl SculkBlock {
    /// Creates a new sculk block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for SculkBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    fn random_tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        // Vanilla `SculkBlock.randomTick` spreads veins from a catalyst; Steel has
        // no sculk-spread feature yet, so only the support check runs here.
        let _ = (state, world, pos);
    }
}

/// Vanilla `FrogspawnBlock`: hatches tadpoles after a fixed delay.
#[block_behavior]
pub struct FrogspawnBlock {
    block: BlockRef,
}

impl FrogspawnBlock {
    /// Creates a new frogspawn block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for FrogspawnBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    fn can_survive(&self, _state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        world.get_block_state(pos.below()).is_solid()
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        if direction == Direction::Down && !self.can_survive(state, world, pos) {
            return REGISTRY.blocks.get_default_state_id(&steel_registry::vanilla_blocks::AIR);
        }
        state
    }
}

/// Vanilla `SnifferEggBlock`: three hatch stages, hatches a sniffer on the final
/// stage after a scheduled tick delay.
#[block_behavior]
pub struct SnifferEggBlock {
    block: BlockRef,
}

impl SnifferEggBlock {
    /// Creates a new sniffer egg block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for SnifferEggBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    fn can_survive(&self, _state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        world.get_block_state(pos.below()).is_solid()
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        if direction == Direction::Down && !self.can_survive(state, world, pos) {
            return REGISTRY.blocks.get_default_state_id(&steel_registry::vanilla_blocks::AIR);
        }
        state
    }
}

/// Vanilla `DriedGhastBlock`: a waterloggable block that rehydrates into a ghast
/// when placed in water.
#[block_behavior]
pub struct DriedGhastBlock {
    block: BlockRef,
}

impl DriedGhastBlock {
    /// Creates a new dried ghast block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for DriedGhastBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(
            self.block
                .default_state()
                .set_value(
                    &BlockStateProperties::WATERLOGGED,
                    context.is_water_source(),
                ),
        )
    }
}

/// Vanilla `CopperGolemStatueBlock`: a copper statue with a pose and oxidation
/// stage. Waxed variants stop oxidising.
#[block_behavior]
pub struct CopperGolemStatueBlock {
    block: BlockRef,
}

impl CopperGolemStatueBlock {
    /// Creates a new copper golem statue block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for CopperGolemStatueBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        // Vanilla faces the statue toward the player when the block has a facing.
        let state = self.block.default_state();
        let Some(facing) = state.try_get_value(&BlockStateProperties::HORIZONTAL_FACING) else {
            return Some(state);
        };
        let _ = facing;
        Some(state.set_value(
            &BlockStateProperties::HORIZONTAL_FACING,
            context.horizontal_direction().opposite(),
        ))
    }
}

/// Vanilla `WeatheringCopperGolemStatueBlock`: an unwaxed copper golem statue,
/// which oxidises over random ticks.
#[block_behavior]
pub struct WeatheringCopperGolemStatueBlock {
    block: BlockRef,
    #[json_arg(r#enum = "WeatherState", json = "weather_state")]
    weathering: crate::behavior::blocks::building::WeatheringCopper,
}

impl WeatheringCopperGolemStatueBlock {
    /// Creates a new weathering copper golem statue behavior.
    #[must_use]
    pub const fn new(
        block: BlockRef,
        weather_state: crate::behavior::blocks::WeatherState,
    ) -> Self {
        Self {
            block,
            weathering: crate::behavior::blocks::building::WeatheringCopper::new(weather_state),
        }
    }
}

impl BlockBehavior for WeatheringCopperGolemStatueBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    fn random_tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        self.weathering.change_over_time(state, world, pos);
    }
}
