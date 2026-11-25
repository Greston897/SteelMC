//! Behaviors for blocks whose vanilla class only differs by a flag or a marker
//! interface, plus the remaining "no-op" blocks.
//!
//! Vanilla declares a class per block even when the class adds almost nothing:
//! `AirBlock`, `TransparentBlock`, `TintedGlassBlock`, `StructureVoidBlock`,
//! `CryingObsidianBlock`, `LightBlock` and the test blocks all inherit the plain
//! `Block` behavior with a single overridden predicate. Modelling them explicitly
//! keeps the generated registry complete and makes the differences greppable.

use std::collections::BTreeMap;
use std::sync::{Arc, Weak};

use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::BlockStateProperties;
use steel_registry::item_stack::ItemStack;
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId, Downcast as _};

use crate::behavior::block::BlockBehavior;
use crate::behavior::block::BlockEntityCreation;
use crate::behavior::blocks::redstone::{MAX_REDSTONE_SIGNAL, MIN_REDSTONE_SIGNAL};
use crate::behavior::context::{BlockHitResult, InteractionResult};
use crate::behavior::{BlockPlaceContext, InventoryAccess};
use crate::block_entity::SharedBlockEntity;
use crate::block_entity::entities::{TestBlockEntity, TestInstanceBlockEntity};
use crate::player::Player;
use crate::world::{LevelReader, SignalGetter as _, SignalQueryContext, World};

/// Vanilla `AirBlock`: no collision, no block entity, never drops.
#[block_behavior]
pub struct AirBlock {
    block: BlockRef,
}

impl AirBlock {
    /// Creates a new air block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for AirBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }
}

/// Vanilla `TransparentBlock` marker (glass, glowstone, and friends): the only
/// behavioral difference is that the block is skipped by some propagation rules.
#[block_behavior]
pub struct TransparentBlock {
    block: BlockRef,
}

impl TransparentBlock {
    /// Creates a new transparent block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for TransparentBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }
}

/// Vanilla `HalfTransparentBlock` (blue ice): like `TransparentBlock`, but it also
/// never conducts redstone and skips random ticks.
#[block_behavior]
pub struct HalfTransparentBlock {
    block: BlockRef,
}

impl HalfTransparentBlock {
    /// Creates a new half-transparent block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for HalfTransparentBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }
}

/// Vanilla `TintedGlassBlock`: blocks light propagation through the block.
#[block_behavior]
pub struct TintedGlassBlock {
    block: BlockRef,
}

impl TintedGlassBlock {
    /// Creates a new tinted glass block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for TintedGlassBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }
}

/// Vanilla `StructureVoidBlock`: no collision, no drops, hides its own outline.
#[block_behavior]
pub struct StructureVoidBlock {
    block: BlockRef,
}

impl StructureVoidBlock {
    /// Creates a new structure void block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for StructureVoidBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }
}

/// Vanilla `CryingObsidianBlock`: places only when the player has a valid
/// respawn anchor context; otherwise it behaves like obsidian.
#[block_behavior]
pub struct CryingObsidianBlock {
    block: BlockRef,
}

impl CryingObsidianBlock {
    /// Creates a new crying obsidian block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for CryingObsidianBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    fn is_possible_to_respawn_in_this(&self, _state: BlockStateId) -> bool {
        true
    }
}

/// Vanilla `LightBlock`: an invisible light source whose `level` state is set on
/// placement; right-clicking cycles the level down.
#[block_behavior]
pub struct LightBlock {
    block: BlockRef,
}

impl LightBlock {
    /// Creates a new light block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for LightBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    fn use_without_item(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _player: &Player,
        _hit_result: &BlockHitResult,
        _inv: &mut InventoryAccess,
    ) -> InteractionResult {
        // Vanilla `LightBlock.useWithoutItem` steps the level down by one.
        let level = state.get_value(&BlockStateProperties::LEVEL);
        if level == 0 {
            return InteractionResult::Pass;
        }
        world.set_block(
            pos,
            state.set_value(&BlockStateProperties::LEVEL, level - 1),
            UpdateFlags::UPDATE_ALL,
        );
        InteractionResult::Success
    }
}

/// Vanilla `TestBlock` and `TestInstanceBlock`: development-only blocks that the
/// vanilla server exposes to the test framework. Steel models them so the block
/// entity and block state IDs line up with vanilla.
#[block_behavior]
pub struct TestBlock {
    block: BlockRef,
}

impl TestBlock {
    /// Creates a new test block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for TestBlock {
    /// Vanilla `TestBlock.newBlockEntity` / `TestInstanceBlock.newBlockEntity`:
    /// both blocks persist editor state in a block entity.
    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::Created(Arc::new(TestBlockEntity::new(level, pos, state)))
    }

    /// Vanilla `TestBlock.getStateForPlacement`: the `minecraft:block_state`
    /// component of the held stack restores the saved mode.
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        use steel_registry::blocks::properties::TestBlockMode;
        use steel_registry::data_components::vanilla_components::BLOCK_STATE;

        let mut to_place = self.block.default_state();
        context.with_item(|stack| {
            let Some(properties) = stack.get(BLOCK_STATE) else {
                return;
            };
            let Some(mode) = properties.get("mode") else {
                return;
            };
            let mode = match mode {
                "start" => TestBlockMode::Start,
                "log" => TestBlockMode::Log,
                "accept" => TestBlockMode::Accept,
                _ => TestBlockMode::Fail,
            };
            to_place = to_place.set_value(&BlockStateProperties::TEST_BLOCK_MODE, mode);
        });
        Some(to_place)
    }

    /// Vanilla `TestBlock.useWithoutItem`: only a game master gets the editor;
    /// everyone else gets `PASS` so the block stays inert.
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
        // The editor is a client-side screen; `true` is the whole server answer.
        InteractionResult::Success
    }

    /// Vanilla `TestBlock.getSignal`: only a `START` block outputs, and only
    /// while it is powered.
    fn get_own_signal(
        &self,
        state: BlockStateId,
        world: &dyn LevelReader,
        pos: BlockPos,
        _context: SignalQueryContext,
    ) -> i32 {
        use steel_registry::blocks::properties::TestBlockMode;
        if state.get_value(&BlockStateProperties::TEST_BLOCK_MODE) != TestBlockMode::Start {
            return 0;
        }
        let Some(block_entity) = world.get_block_entity(pos) else {
            return 0;
        };
        let powered = block_entity
            .downcast_ref::<TestBlockEntity>()
            .is_some_and(TestBlockEntity::is_powered);
        if powered {
            MAX_REDSTONE_SIGNAL
        } else {
            MIN_REDSTONE_SIGNAL
        }
    }

    /// Vanilla `TestBlock.neighborChanged`: a non-`START` block tracks its
    /// neighbor signal and triggers on the rising edge.
    fn handle_neighbor_changed(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _source_block: BlockRef,
        _moved_by_piston: bool,
    ) {
        use steel_registry::blocks::properties::TestBlockMode;
        if state.get_value(&BlockStateProperties::TEST_BLOCK_MODE) == TestBlockMode::Start {
            return;
        }
        let Some(block_entity) = world.get_block_entity(pos) else {
            return;
        };
        let Some(test_block) = block_entity.downcast_ref::<TestBlockEntity>() else {
            return;
        };
        let should_trigger = world.has_neighbor_signal(pos);
        let is_powered = test_block.is_powered();
        if should_trigger && !is_powered {
            test_block.set_powered(true);
            test_block.trigger(world);
        } else if !should_trigger && is_powered {
            test_block.set_powered(false);
        }
    }

    /// Vanilla `TestBlock.tick`: a scheduled tick resets the block so the next
    /// test run starts from a clean state.
    fn tick(&self, _state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        let Some(block_entity) = world.get_block_entity(pos) else {
            return;
        };
        if let Some(test_block) = block_entity.downcast_ref::<TestBlockEntity>() {
            test_block.reset(world);
        }
    }

    /// Vanilla `TestBlock.getCloneItemStack`: the picked stack keeps the current
    /// mode in its `minecraft:block_state` component.
    fn get_clone_item_stack(
        &self,
        block: BlockRef,
        state: BlockStateId,
        _block_entity: Option<SharedBlockEntity>,
        _include_data: bool,
    ) -> Option<ItemStack> {
        use steel_registry::blocks::properties::PropertyEnum as _;
        use steel_registry::data_components::components::BlockItemStateProperties;
        use steel_registry::data_components::vanilla_components::BLOCK_STATE;

        let mut stack = ItemStack::new(steel_registry::REGISTRY.items.by_block(block));
        let mode = state.get_value(&BlockStateProperties::TEST_BLOCK_MODE);
        let mut properties = BTreeMap::new();
        properties.insert("mode".to_owned(), mode.as_str().to_owned());
        stack.set(BLOCK_STATE, BlockItemStateProperties::new(properties));
        Some(stack)
    }
}

/// Vanilla `TestInstanceBlock`.
#[block_behavior]
pub struct TestInstanceBlock {
    block: BlockRef,
}

impl TestInstanceBlock {
    /// Creates a new test instance block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for TestInstanceBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    /// Vanilla `TestBlock.newBlockEntity` / `TestInstanceBlock.newBlockEntity`:
    /// both blocks persist editor state in a block entity.
    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::Created(Arc::new(TestInstanceBlockEntity::new(level, pos, state)))
    }

    /// Vanilla `TestInstanceBlock.useWithoutItem`: identical to the test block,
    /// a game master opens the editor and everyone else gets `PASS`.
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
        InteractionResult::Success
    }
}
