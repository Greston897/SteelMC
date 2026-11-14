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

use glam::DVec3;
use steel_registry::item_stack::ItemStack;
use steel_registry::{vanilla_entities, vanilla_items};

use crate::behavior::BlockBehavior;
use crate::behavior::block::BlockEntityCreation;
use crate::block_entity::SharedBlockEntity;
use crate::block_entity::entities::CopperGolemStatueBlockEntity;
use crate::entity::{ENTITIES, next_entity_id};
use crate::behavior::BlockPlaceContext;
use crate::world::{LevelReader, ScheduledTickAccess, World};

/// Vanilla `InfestedBlock`: breaking it spawns a silverfish instead of dropping
/// the block itself.
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

impl InfestedBlock {
    /// Vanilla `InfestedBlock.spawnInfestation`: a silverfish pops out of the
    /// broken block.
    pub(crate) fn spawn_infestation(world: &Arc<World>, pos: BlockPos) {
        let position = DVec3::new(
            f64::from(pos.x()) + 0.5,
            f64::from(pos.y()) + 0.5,
            f64::from(pos.z()) + 0.5,
        );
        if let Some(silverfish) = ENTITIES.create(
            &vanilla_entities::SILVERFISH,
            next_entity_id(),
            position,
            Arc::downgrade(world),
        ) && world.try_add_entity(silverfish).is_ok()
        {
            return;
        }
        log::warn!("failed to spawn a silverfish from infested block at {pos:?}");
    }
}

impl BlockBehavior for InfestedBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    /// Vanilla `InfestedBlock.spawnAfterBreak`: the block leaves no drop, only a
    /// silverfish.
    fn spawn_after_break(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _tool: &ItemStack,
        _drop_experience: bool,
    ) {
        Self::spawn_infestation(world, pos);
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
    fn spawn_after_break(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _tool: &ItemStack,
        _drop_experience: bool,
    ) {
        InfestedBlock::spawn_infestation(world, pos);
    }

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

    /// Vanilla `CopperGolemStatueBlock.updatePose`: plays the statue sound, cycles
    /// the pose and fires the `BLOCK_CHANGE` game event.
    pub fn update_pose(world: &Arc<World>, state: BlockStateId, pos: BlockPos, player: &crate::player::Player) {
        world.play_sound_at(
            &steel_registry::sound_events::ENTITY_COPPER_GOLEM_BECOME_STATUE,
            steel_protocol::packets::game::SoundSource::Blocks,
            glam::DVec3::new(
                f64::from(pos.x()) + 0.5,
                f64::from(pos.y()) + 0.5,
                f64::from(pos.z()) + 0.5,
            ),
            1.0,
            1.0,
            None,
        );
        world.set_block(
            pos,
            state.set_value(
                &BlockStateProperties::COPPER_GOLEM_POSE,
                next_statue_pose(state.get_value(&BlockStateProperties::COPPER_GOLEM_POSE)),
            ),
            UpdateFlags::UPDATE_ALL,
        );
        world.game_event(
            &steel_registry::vanilla_game_events::BLOCK_CHANGE,
            pos,
            &crate::world::game_event::GameEventContext::new(
                Some(player as &dyn crate::entity::Entity),
                Some(state),
            ),
        );
    }
}

impl BlockBehavior for CopperGolemStatueBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        // Vanilla faces the statue toward the player and waterlogs it when it is
        // placed into water.
        let state = self.block.default_state();
        let Some(facing) = state.try_get_value(&BlockStateProperties::HORIZONTAL_FACING) else {
            return Some(state);
        };
        let _ = facing;
        Some(
            state
                .set_value(
                    &BlockStateProperties::HORIZONTAL_FACING,
                    context.horizontal_direction().opposite(),
                )
                .set_value(&BlockStateProperties::WATERLOGGED, context.is_water_source()),
        )
    }

    /// Vanilla `CopperGolemStatueBlock.useItemOn`: an axe passes (so it can
    /// scrape the statue), anything else cycles the pose and plays the sound.
    fn use_item_on(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &crate::player::Player,
        _hand: steel_utils::types::InteractionHand,
        _hit_result: &crate::behavior::context::BlockHitResult,
        inv: &mut crate::behavior::context::InventoryAccess,
    ) -> InteractionResult {
        use steel_registry::vanilla_item_tags::ItemTag;

        if inv.with_item(|item| item.item().has_tag(&ItemTag::AXES)) {
            return InteractionResult::Pass;
        }
        Self::update_pose(world, state, pos, player);
        InteractionResult::Success
    }

    /// Vanilla `CopperGolemStatueBlock.updateShape`: a waterlogged statue
    /// schedules the water tick when its neighbour changes.
    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        _direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        crate::behavior::block::schedule_water_tick_if_waterlogged(state, world, pos);
        state
    }

    /// Vanilla `CopperGolemStatueBlock.affectNeighborsAfterRemoval` updates the
    /// comparator outputs of the surrounding blocks.
    fn affect_neighbors_after_removal(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _moved_by_piston: bool,
    ) {
        world.update_neighbour_for_output_signal(pos, state.get_block());
    }

    /// Vanilla `CopperGolemStatueBlock.newBlockEntity`.
    fn new_block_entity(
        &self,
        level: std::sync::Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::Created(std::sync::Arc::new(
            CopperGolemStatueBlockEntity::new(level, pos, state),
        ))
    }

    /// Vanilla `CopperGolemStatueBlock.isPathfindable`: only water inside a
    /// waterlogged statue is pathfindable.
    fn is_pathfindable(
        &self,
        state: BlockStateId,
        computation_type: crate::entity::ai::path::PathComputationType,
    ) -> bool {
        computation_type == crate::entity::ai::path::PathComputationType::Water
            && state.get_value(&BlockStateProperties::WATERLOGGED)
    }

    /// Vanilla `CopperGolemStatueBlock.getCloneItemStack`: the picked stack keeps
    /// the statue's stored components and its current pose.
    fn get_clone_item_stack(
        &self,
        block: BlockRef,
        state: BlockStateId,
        block_entity: Option<SharedBlockEntity>,
        include_data: bool,
    ) -> Option<ItemStack> {
        use steel_registry::data_components::components::BlockItemStateProperties;
        use steel_registry::data_components::vanilla_components::BLOCK_STATE;

        let mut stack = ItemStack::new(REGISTRY.items.by_block(block));
        let Some(statue) = block_entity
            .as_ref()
            .and_then(|entity| entity.downcast_ref::<CopperGolemStatueBlockEntity>())
        else {
            return Some(stack);
        };

        if include_data {
            // Vanilla `itemStack.applyComponents(collectComponents())`.
            let mut patch = stack.patch().clone();
            for (key, data) in statue.collect_components().iter() {
                patch.set_raw(key.clone(), data.clone());
            }
            stack = ItemStack::with_count_and_patch(stack.item(), stack.count(), patch);
        }

        let pose = state.get_value(&BlockStateProperties::COPPER_GOLEM_POSE);
        let mut properties = std::collections::BTreeMap::new();
        properties.insert(
            "copper_golem_pose".to_owned(),
            steel_registry::blocks::properties::PropertyEnum::as_str(&pose).to_owned(),
        );
        stack.set(BLOCK_STATE, BlockItemStateProperties::new(properties));
        Some(stack)
    }

    /// Vanilla `CopperGolemStatueBlock.hasAnalogOutputSignal`.
    fn has_analog_output_signal(&self, _state: BlockStateId) -> bool {
        true
    }

    /// Vanilla `CopperGolemStatueBlock.getAnalogOutputSignal`: the pose ordinal
    /// plus one, so standing reads 1 and star reads 4.
    fn get_analog_output_signal(
        &self,
        state: BlockStateId,
        _world: &World,
        _pos: BlockPos,
        _direction: steel_utils::Direction,
    ) -> i32 {
        use steel_registry::blocks::properties::Pose;

        match state.get_value(&BlockStateProperties::COPPER_GOLEM_POSE) {
            Pose::Standing => 1,
            Pose::Sitting => 2,
            Pose::Running => 3,
            Pose::Star => 4,
        }
    }
}

/// Vanilla `CopperGolemStatueBlock.Pose.getNextPose`: `ByIdMap.continuous`
/// wraps past the last pose instead of clamping.
fn next_statue_pose(
    pose: steel_registry::blocks::properties::Pose,
) -> steel_registry::blocks::properties::Pose {
    use steel_registry::blocks::properties::Pose;

    match pose {
        Pose::Standing => Pose::Sitting,
        Pose::Sitting => Pose::Running,
        Pose::Running => Pose::Star,
        Pose::Star => Pose::Standing,
    }
}

/// Vanilla `WeatheringCopperGolemStatueBlock`: an unwaxed copper golem statue,
/// which oxidises over random ticks.
#[block_behavior]
pub struct WeatheringCopperGolemStatueBlock {
    block: BlockRef,
    #[json_arg(r#enum = "WeatherState", json = "weathering_state")]
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
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        // Inherited from `CopperGolemStatueBlock`.
        let state = self.block.default_state();
        Some(
            state
                .set_value(
                    &BlockStateProperties::HORIZONTAL_FACING,
                    context.horizontal_direction().opposite(),
                )
                .set_value(&BlockStateProperties::WATERLOGGED, context.is_water_source()),
        )
    }

    fn random_tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        self.weathering.change_over_time(state, world, pos);
    }

    /// Vanilla `WeatheringCopperGolemStatueBlock.useItemOn`: honeycomb passes so
    /// waxing can happen, any other non-axe item cycles the pose, and an axe
    /// scrapes an unaffected statue back into a live copper golem.
    fn use_item_on(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &crate::player::Player,
        hand: steel_utils::types::InteractionHand,
        _hit_result: &crate::behavior::context::BlockHitResult,
        inv: &mut crate::behavior::context::InventoryAccess,
    ) -> InteractionResult {
        use steel_registry::vanilla_item_tags::ItemTag;

        let Some(block_entity) = world.get_block_entity(pos) else {
            return InteractionResult::Pass;
        };
        let Some(statue) = block_entity.downcast_ref::<CopperGolemStatueBlockEntity>() else {
            return InteractionResult::Pass;
        };

        let is_axe = inv.with_item(|item| item.item().has_tag(&ItemTag::AXES));
        if !is_axe {
            let is_honeycomb = inv.with_item(|item| item.is(&vanilla_items::HONEYCOMB));
            if is_honeycomb {
                return InteractionResult::Pass;
            }
            CopperGolemStatueBlock::update_pose(world, state, pos, player);
            return InteractionResult::Success;
        }

        if self.weathering.weather_state() != WeatherState::Unaffected {
            return InteractionResult::Pass;
        }

        let Some(golem) = statue.remove_statue(state) else {
            return InteractionResult::Pass;
        };
        inv.with_item_mut(|item| {
            item.hurt_and_break(1, player.has_infinite_materials());
        });
        if world.try_add_entity(golem).is_ok() {
            world.remove_block(pos, false);
            return InteractionResult::Success;
        }
        InteractionResult::Pass
    }
}
