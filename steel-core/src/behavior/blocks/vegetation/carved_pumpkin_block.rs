use std::sync::Arc;

use crate::behavior::{BlockBehavior, BlockPlaceContext, BlockRef};
use crate::entity::{create_entity_instance, add_spawned_entity};
use steel_utils::types::UpdateFlags;
use glam::DVec3;
use crate::world::World;
use steel_macros::block_behavior;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{BlockStateProperties, Direction, EnumProperty};
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::{vanilla_blocks, vanilla_entities};
use steel_utils::{BlockPos, BlockStateId, Downcast as _};
use crate::entity::entities::{IronGolemEntity, CopperGolemEntity};

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
    fn golem_for_pattern(world: &World, pos: BlockPos) -> Option<(EntityTypeRef, Vec<BlockPos>)> {
        let below1 = pos.offset(0, -1, 0);
        let below2 = pos.offset(0, -2, 0);

        if world.get_block_state(below1).get_block() == &vanilla_blocks::SNOW_BLOCK
            && world.get_block_state(below2).get_block() == &vanilla_blocks::SNOW_BLOCK
        {
            return Some((&vanilla_entities::SNOW_GOLEM, vec![pos, below1, below2]));
        }

        // Arms belong to the upper iron row, not the feet row. Both horizontal
        // orientations are legal, with air at all four corners of the pattern.
        if world.get_block_state(below1).get_block() == &vanilla_blocks::IRON_BLOCK
            && world.get_block_state(below2).get_block() == &vanilla_blocks::IRON_BLOCK
        {
            for (dx, dz) in [(1, 0), (0, 1)] {
                let arms = [below1.offset(dx, 0, dz), below1.offset(-dx, 0, -dz)];
                let corners = [
                    pos.offset(dx, 0, dz), pos.offset(-dx, 0, -dz),
                    below2.offset(dx, 0, dz), below2.offset(-dx, 0, -dz),
                ];
                if arms.iter().all(|&p| world.get_block_state(p).get_block() == &vanilla_blocks::IRON_BLOCK)
                    && corners.iter().all(|&p| world.get_block_state(p).is_air())
                {
                    let mut blocks = vec![pos, below1, below2];
                    blocks.extend(arms);
                    return Some((&vanilla_entities::IRON_GOLEM, blocks));
                }
            }
        }
        if Self::copper_variant(world.get_block_state(below1).get_block()).is_some() {
            return Some((&vanilla_entities::COPPER_GOLEM, vec![pos, below1]));
        }
        None
    }

    fn copper_variant(block: BlockRef) -> Option<(BlockRef, i32)> {
        use steel_registry::vanilla_blocks::*;
        [
            (&COPPER_BLOCK, &COPPER_CHEST, 0),
            (&EXPOSED_COPPER, &EXPOSED_COPPER_CHEST, 1),
            (&WEATHERED_COPPER, &WEATHERED_COPPER_CHEST, 2),
            (&OXIDIZED_COPPER, &OXIDIZED_COPPER_CHEST, 3),
            (&WAXED_COPPER_BLOCK, &WAXED_COPPER_CHEST, 0),
            (&WAXED_EXPOSED_COPPER, &WAXED_EXPOSED_COPPER_CHEST, 1),
            (&WAXED_WEATHERED_COPPER, &WAXED_WEATHERED_COPPER_CHEST, 2),
            (&WAXED_OXIDIZED_COPPER, &WAXED_OXIDIZED_COPPER_CHEST, 3),
        ].into_iter().find_map(|(source, chest, age)| (block == source).then_some((chest, age)))
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
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        old_state: BlockStateId,
        _moved_by_piston: bool,
    ) {
        if old_state.get_block() == state.get_block() {
            return;
        }
        let Some((entity_type, blocks)) = Self::golem_for_pattern(world, pos) else {
            return;
        };
        let copper = Self::copper_variant(world.get_block_state(pos.offset(0, -1, 0)).get_block());
        let feet = if copper.is_some() { pos } else { pos.offset(0, -2, 0) };
        let location = DVec3::new(f64::from(feet.x()) + 0.5,
            f64::from(feet.y()) + 0.05, f64::from(feet.z()) + 0.5);
        let Ok(entity) = create_entity_instance(world, entity_type, location) else {
            return;
        };
        if let Some(golem) = entity.downcast_ref::<IronGolemEntity>() {
            golem.set_player_created(true);
        }
        if let Some((_, age)) = copper
            && let Some(golem) = entity.downcast_ref::<CopperGolemEntity>()
        {
            golem.set_spawn_weather_state(age);
        }
        // Publish successfully before consuming a pattern. A failed entity
        // insertion must not destroy the player's construction.
        if add_spawned_entity(world, entity).is_err() {
            return;
        }
        for &block_pos in &blocks {
            world.set_block(block_pos, vanilla_blocks::AIR.default_state(), UpdateFlags::UPDATE_CLIENTS);
        }
        if let Some((chest, _)) = copper {
            world.set_block(pos.offset(0, -1, 0), chest.default_state()
                .set_value(HORIZONTAL_FACING, state.get_value(HORIZONTAL_FACING)), UpdateFlags::UPDATE_CLIENTS);
        }
        // All pattern blocks must be gone before any neighbour sees the change.
        for block_pos in blocks {
            world.update_neighbors_at(block_pos, &vanilla_blocks::AIR);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{fresh_test_world, insert_ready_full_chunk};
    use steel_utils::ChunkPos;

    #[test]
    fn iron_pattern_requires_upper_arms_and_clear_corners_in_both_axes() {
        steel_registry::init_vanilla_registry();
        let fixture = fresh_test_world("iron_golem_pattern");
        let world = &fixture.world;
        let head = BlockPos::new(8, 66, 8);
        let _holder = insert_ready_full_chunk(world, ChunkPos::from_block_pos(head));
        for (dx, dz) in [(1, 0), (0, 1)] {
            for y in [-1, -2] {
                world.set_block(head.offset(0, y, 0), vanilla_blocks::IRON_BLOCK.default_state(), UpdateFlags::UPDATE_NONE);
            }
            for sign in [-1, 1] {
                world.set_block(head.offset(sign * dx, -1, sign * dz), vanilla_blocks::IRON_BLOCK.default_state(), UpdateFlags::UPDATE_NONE);
            }
            assert_eq!(CarvedPumpkinBlock::golem_for_pattern(world, head).unwrap().0, &vanilla_entities::IRON_GOLEM);
            let corner = head.offset(dx, 0, dz);
            world.set_block(corner, vanilla_blocks::STONE.default_state(), UpdateFlags::UPDATE_NONE);
            assert!(CarvedPumpkinBlock::golem_for_pattern(world, head).is_none());
            world.set_block(corner, vanilla_blocks::AIR.default_state(), UpdateFlags::UPDATE_NONE);
            for sign in [-1, 1] {
                world.set_block(head.offset(sign * dx, -1, sign * dz), vanilla_blocks::AIR.default_state(), UpdateFlags::UPDATE_NONE);
            }
            assert!(CarvedPumpkinBlock::golem_for_pattern(world, head).is_none());
        }
    }

    #[test]
    fn copper_construction_preserves_wax_and_oxidation_at_both_extremes() {
        steel_registry::init_vanilla_registry();
        assert_eq!(CarvedPumpkinBlock::copper_variant(&vanilla_blocks::COPPER_BLOCK), Some((&vanilla_blocks::COPPER_CHEST, 0)));
        assert_eq!(CarvedPumpkinBlock::copper_variant(&vanilla_blocks::WAXED_OXIDIZED_COPPER), Some((&vanilla_blocks::WAXED_OXIDIZED_COPPER_CHEST, 3)));
        assert!(CarvedPumpkinBlock::copper_variant(&vanilla_blocks::IRON_BLOCK).is_none());
    }
}
