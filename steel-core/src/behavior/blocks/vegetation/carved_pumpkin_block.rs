use std::sync::Arc;

use crate::behavior::{BlockBehavior, BlockPlaceContext, BlockRef};
use crate::entity::{EntitySpawnPlacement, EntitySpawnReason, EntitySpawnRequest, spawn_entity};
use crate::world::{LevelReader, World};
use steel_macros::block_behavior;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{BlockStateProperties, Direction, EnumProperty};
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::{vanilla_blocks, vanilla_entities};
use steel_utils::{BlockPos, BlockStateId};

const HORIZONTAL_FACING: &EnumProperty<Direction> = &BlockStateProperties::HORIZONTAL_FACING;

/// Behavior for carved pumpkins.
#[block_behavior]
pub struct CarvedPumpkinBlock {
    block: BlockRef,
}

impl CarvedPumpkinBlock {
    /// Creates a carved pumpkin block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }

    /// Returns the golem the pumpkin pattern below this pumpkin spawns, if any.
    ///
    /// Vanilla `CarvedPumpkinBlock.canSpawnGolem` only fires for a pumpkin that
    /// was just placed, so this runs from the placement hook and never on a
    /// world load.
    fn golem_for_pattern(world: &World, pos: BlockPos) -> Option<EntityTypeRef> {
        let below1 = pos.offset(0, -1, 0);
        let below2 = pos.offset(0, -2, 0);

        let has_arms = |center: BlockPos, block: &BlockRef| {
            world.get_block_state(center.offset(-1, 0, 0)).get_block() == block
                && world.get_block_state(center.offset(1, 0, 0)).get_block() == block
        };

        // Snow golem: two snow blocks below the pumpkin.
        if world.get_block_state(below1).get_block() == &*vanilla_blocks::SNOW_BLOCK
            && world.get_block_state(below2).get_block() == &*vanilla_blocks::SNOW_BLOCK
        {
            return Some(&vanilla_entities::SNOW_GOLEM);
        }

        // Iron and copper golems: two metal blocks below plus a 3-wide arm row.
        for (block, entity_type) in [
            (&vanilla_blocks::IRON_BLOCK, &vanilla_entities::IRON_GOLEM),
            (&vanilla_blocks::COPPER_BLOCK, &vanilla_entities::COPPER_GOLEM),
        ] {
            if world.get_block_state(below1).get_block() == block
                && world.get_block_state(below2).get_block() == block
                && has_arms(below2, block)
            {
                return Some(entity_type);
            }
        }
        None
    }
}

impl BlockBehavior for CarvedPumpkinBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(
            self.block
                .default_state()
                .set_value(HORIZONTAL_FACING, context.horizontal_direction().opposite()),
        )
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
        let Some(entity_type) = Self::golem_for_pattern(world, pos) else {
            return;
        };

        let request = EntitySpawnRequest {
            entity_type,
            placement: EntitySpawnPlacement::Block {
                pos,
                try_move_down: false,
                moved_up: false,
            },
            reason: EntitySpawnReason::Structure,
            finalize_spawn: true,
            play_ambient_sound: false,
            item_stack: None,
            user_is_operator: false,
        };
        let _ = spawn_entity(world, request);
    }
}
