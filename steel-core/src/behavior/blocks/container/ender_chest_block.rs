//! Ender chest block behavior implementation.

use std::sync::{Arc, Weak};

use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::BlockStateProperties;
use steel_registry::{vanilla_block_entity_types, vanilla_custom_stats};
use steel_utils::{BlockPos, BlockStateId, Direction, translations};
use text_components::TextComponent;

use crate::behavior::InventoryAccess;
use crate::behavior::block::{
    BlockBehavior, BlockEntityCreation, schedule_water_tick_if_waterlogged,
};
use crate::behavior::context::{BlockHitResult, BlockPlaceContext, InteractionResult};
use crate::block_entity::BLOCK_ENTITIES;
use crate::inventory::menu::kinds::ender_chest;
use crate::player::Player;
use crate::world::{ScheduledTickAccess, World, is_redstone_conductor};

/// The ender chest block behavior.
#[block_behavior]
pub struct EnderChestBlock {
    block: BlockRef,
}

impl EnderChestBlock {
    /// Creates a new ender chest block.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for EnderChestBlock {
    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        _direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        schedule_water_tick_if_waterlogged(state, world, pos);
        state
    }

    fn on_place(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _old_state: BlockStateId,
        _moved_by_piston: bool,
    ) {
        schedule_water_tick_if_waterlogged(state, world.as_ref(), pos);
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        let facing = context.horizontal_direction().opposite();

        Some(
            self.block
                .default_state()
                .set_value(&BlockStateProperties::FACING, facing)
                .set_value(
                    &BlockStateProperties::WATERLOGGED,
                    context.is_water_source(),
                ),
        )
    }

    fn allows_spectator_menu(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
    ) -> bool {
        // Vanilla `allowsSpectatorMenu` for container blocks.
        let _ = (state, world, pos);
        true
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
            return InteractionResult::Success;
        };
        if block_entity.get_type() != &vanilla_block_entity_types::ENDER_CHEST {
            return InteractionResult::Success;
        }

        let above_pos = pos.above();
        if is_redstone_conductor(world.as_ref(), world.get_block_state(above_pos), above_pos) {
            return InteractionResult::Success;
        }

        let inventory = Arc::clone(&player.inventory);
        let container = Arc::clone(&player.ender_chest_inventory);
        let chest = Arc::downgrade(&block_entity);
        player.open_menu(
            TextComponent::translated(translations::CONTAINER_ENDERCHEST.msg()),
            move |context| {
                // open only after active chest has closed, since
                // closing an ender chest menu clears the active chest.
                container.lock().set_active_chest(chest);
                ender_chest(inventory, context.container_id, container)
            },
        );

        player.award_custom_stat(&vanilla_custom_stats::OPEN_ENDERCHEST);
        super::anger_nearby_piglins(world, pos, player);

        InteractionResult::Success
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::ENDER_CHEST,
            level,
            pos,
            state,
        ))
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_blocks, vanilla_fluids};

    use super::*;
    use crate::test_support::TestLevel;

    #[test]
    fn waterlogged_ender_chest_schedules_water_on_neighbor_update() {
        init_vanilla_registry();
        let behavior = EnderChestBlock::new(&vanilla_blocks::ENDER_CHEST);
        let state = vanilla_blocks::ENDER_CHEST
            .default_state()
            .set_value(&BlockStateProperties::WATERLOGGED, true);
        let level = TestLevel::default();
        let pos = BlockPos::new(0, 64, 0);
        assert_eq!(
            behavior.update_shape(
                state,
                &level,
                pos,
                Direction::East,
                pos.east(),
                vanilla_blocks::AIR.default_state(),
            ),
            state
        );
        assert_eq!(
            level
                .scheduled_fluid_ticks
                .borrow()
                .iter()
                .map(|tick| (tick.pos, tick.fluid, tick.delay))
                .collect::<Vec<_>>(),
            vec![(pos, &vanilla_fluids::WATER, 5)]
        );
    }
}
