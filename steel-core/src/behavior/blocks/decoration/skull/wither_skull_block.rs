use crate::behavior::blocks::SkullBlock;
use crate::behavior::blocks::decoration::skull::abstract_skull_block::AbstractSkullBlock;
use crate::behavior::{BlockBehavior, BlockEntityCreation, BlockLootContext, BlockPlaceContext};
use crate::block_entity::SharedBlockEntity;
use crate::entity::ai::path::PathComputationType;
use crate::entity::{EntitySpawnReason, Mob as _};
use crate::world::World;
use std::sync::{Arc, Weak};
use steel_macros::block_behavior;
use glam::DVec3;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_blocks;
use steel_registry::vanilla_entities;
use steel_utils::{BlockPos, BlockStateId};


/// Behavior for wither skull blocks.
#[block_behavior]
pub struct WitherSkullBlock {
    base: SkullBlock,
}

impl WitherSkullBlock {
    /// Creates a new wither skull block behavior for the given block.
    pub const fn new(block: BlockRef) -> Self {
        Self {
            base: SkullBlock::new(block),
        }
    }

    /// Vanilla `WitherSkullBlock.checkSpawn`: looks for the T-shaped pattern
    /// (three skulls above two soul sand/soil blocks) and, when found, removes
    /// the skulls and spawns the wither.
    ///
    /// Called after a skull is placed. Returns whether a wither was spawned.
    pub(crate) fn check_spawn(world: &Arc<World>, skull_pos: BlockPos) -> bool {
        // The skull must be the middle top block of the pattern.
        let base = skull_pos.below();
        if !is_soul_base(world, base) {
            return false;
        }
        if !is_soul_base(world, base.below()) {
            return false;
        }

        // All three skull positions must hold wither skulls.
        let skull_positions = [
            skull_pos,
            skull_pos.offset(-1, 0, 0),
            skull_pos.offset(1, 0, 0),
        ];
        for pos in skull_positions {
            if !is_wither_skull(world, pos) {
                return false;
            }
        }

        // Vanilla consumes the three skulls, then spawns the wither at the base.
        for pos in skull_positions {
            world.remove_block(pos, false);
        }
        spawn_wither(world, base);
        true
    }
}

/// Returns whether `pos` holds soul sand or soul soil.
fn is_soul_base(world: &Arc<World>, pos: BlockPos) -> bool {
    let block = world.get_block_state(pos).get_block();
    block == &vanilla_blocks::SOUL_SAND || block == &vanilla_blocks::SOUL_SOIL
}

/// Returns whether `pos` holds a wither skeleton skull, floor or wall variant.
fn is_wither_skull(world: &Arc<World>, pos: BlockPos) -> bool {
    let block = world.get_block_state(pos).get_block();
    block == &vanilla_blocks::WITHER_SKELETON_SKULL
        || block == &vanilla_blocks::WITHER_SKELETON_WALL_SKULL
}

/// Creates and adds the wither entity above the soul-sand base.
fn spawn_wither(world: &Arc<World>, base: BlockPos) {
    let position = DVec3::new(
        f64::from(base.x()) + 0.5,
        f64::from(base.y()) + 1.0,
        f64::from(base.z()) + 0.5,
    );

    let Some(entity) = crate::entity::ENTITIES.create(
        &vanilla_entities::WITHER,
        crate::entity::next_entity_id(),
        position,
        Arc::downgrade(world),
    ) else {
        return;
    };

    if let Some(mob) = entity.as_mob() {
        let _ = mob.finalize_spawn(world, EntitySpawnReason::Event, None);
    }
    let _ = world.try_add_entity(entity);
}

impl BlockBehavior for WitherSkullBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        self.base.get_state_for_placement(context)
    }

    fn on_place(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _old_state: BlockStateId,
        moved_by_piston: bool,
    ) {
        if moved_by_piston {
            return;
        }
        Self::check_spawn(world, pos);
    }

    fn handle_neighbor_changed(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        source_block: BlockRef,
        moved_by_piston: bool,
    ) {
        self.base
            .handle_neighbor_changed(state, world, pos, source_block, moved_by_piston);
    }

    fn is_pathfindable(&self, state: BlockStateId, computation_type: PathComputationType) -> bool {
        self.base.is_pathfindable(state, computation_type)
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        self.base.new_block_entity(level, pos, state)
    }

    fn get_clone_item_stack(
        &self,
        block: BlockRef,
        state: BlockStateId,
        block_entity: Option<SharedBlockEntity>,
        include_data: bool,
    ) -> Option<ItemStack> {
        self.base
            .get_clone_item_stack(block, state, block_entity, include_data)
    }

    fn get_drops(
        &self,
        state: BlockStateId,
        context: &BlockLootContext<'_>,
    ) -> Option<Vec<ItemStack>> {
        self.base.get_drops(state, context)
    }
}

impl AbstractSkullBlock for WitherSkullBlock {
    fn state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        self.base.state_for_placement(context)
    }
}
