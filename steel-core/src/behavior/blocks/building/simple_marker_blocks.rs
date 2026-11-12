//! Behaviors for blocks whose vanilla class only differs by a flag or a marker
//! interface, plus the remaining "no-op" blocks.
//!
//! Vanilla declares a class per block even when the class adds almost nothing:
//! `AirBlock`, `TransparentBlock`, `TintedGlassBlock`, `StructureVoidBlock`,
//! `CryingObsidianBlock`, `LightBlock` and the test blocks all inherit the plain
//! `Block` behavior with a single overridden predicate. Modelling them explicitly
//! keeps the generated registry complete and makes the differences greppable.

use std::sync::Arc;

use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::BlockStateProperties;
use steel_utils::{BlockPos, BlockStateId};

use crate::behavior::block::BlockBehavior;
use crate::behavior::context::{BlockHitResult, InteractionResult};
use crate::behavior::{BlockPlaceContext, InventoryAccess};
use crate::player::Player;
use crate::world::World;

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
            steel_utils::types::UpdateFlags::UPDATE_ALL,
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
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
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
}
