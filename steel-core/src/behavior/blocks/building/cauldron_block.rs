//! State-driven cauldron behavior needed by comparators.
//!
//! Cauldron item interactions and drip filling remain separate block mechanics;
//! this module implements the complete vanilla analog-output contract shared by
//! empty, water, and powder-snow cauldrons.

use std::sync::Arc;

use steel_macros::block_behavior;
use steel_registry::{
    blocks::{
        BlockRef,
        block_state_ext::BlockStateExt as _,
        properties::{BlockStateProperties, Direction, IntProperty},
    },
    level_events, vanilla_blocks, vanilla_fluids, vanilla_game_events,
};
use steel_utils::{BlockPos, BlockStateId, types::UpdateFlags};

use crate::{
    behavior::{BlockBehavior, BlockPlaceContext, blocks::vegetation},
    entity::ai::path::PathComputationType,
    world::{World, game_event::GameEventContext},
};

/// Vanilla empty cauldron behavior.
#[block_behavior]
pub struct CauldronBlock {
    block: BlockRef,
}

const LEVEL_CAULDRON: &IntProperty = &BlockStateProperties::LEVEL_CAULDRON;

impl CauldronBlock {
    /// Creates empty cauldron behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for CauldronBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    fn use_item_on(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos,
        player: &crate::player::Player, hand: steel_utils::types::InteractionHand,
        _hit: &crate::behavior::BlockHitResult, inv: &mut crate::behavior::InventoryAccess,
    ) -> crate::behavior::InteractionResult {
        cauldron_item_interaction(state, world, pos, player, hand, inv)
    }

    fn has_analog_output_signal(&self, _state: BlockStateId) -> bool {
        true
    }

    fn get_analog_output_signal(
        &self,
        _state: BlockStateId,
        _world: &World,
        _pos: BlockPos,
        _direction: Direction,
    ) -> i32 {
        0
    }

    fn is_pathfindable(
        &self,
        _state: BlockStateId,
        _computation_type: PathComputationType,
    ) -> bool {
        false
    }

    fn tick(&self, _state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        let Some(stalactite_pos) =
            vegetation::find_stalactite_tip_above_cauldron(world.as_ref(), pos)
        else {
            return;
        };
        let Some(fluid) = vegetation::get_cauldron_fill_fluid_type(world, stalactite_pos) else {
            return;
        };

        if fluid == &vanilla_fluids::WATER {
            let new_state = vanilla_blocks::WATER_CAULDRON
                .default_state()
                .set_value(LEVEL_CAULDRON, 1);
            world.set_block(pos, new_state, UpdateFlags::UPDATE_ALL);
            world.game_event(
                &vanilla_game_events::BLOCK_CHANGE,
                pos,
                &GameEventContext::new(None, Some(new_state)),
            );
            world.level_event(level_events::SOUND_DRIP_WATER_INTO_CAULDRON, pos, 0, None);
        } else if fluid == &vanilla_fluids::LAVA {
            let new_state = vanilla_blocks::LAVA_CAULDRON.default_state();
            world.set_block(pos, new_state, UpdateFlags::UPDATE_ALL);
            world.game_event(
                &vanilla_game_events::BLOCK_CHANGE,
                pos,
                &GameEventContext::new(None, Some(new_state)),
            );
            world.level_event(level_events::SOUND_DRIP_LAVA_INTO_CAULDRON, pos, 0, None);
        }
    }
}

/// Vanilla layered water and powder-snow cauldron behavior.
#[block_behavior]
pub struct LayeredCauldronBlock {
    block: BlockRef,
}

impl LayeredCauldronBlock {
    /// Creates layered cauldron behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for LayeredCauldronBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    fn use_item_on(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos,
        player: &crate::player::Player, hand: steel_utils::types::InteractionHand,
        _hit: &crate::behavior::BlockHitResult, inv: &mut crate::behavior::InventoryAccess,
    ) -> crate::behavior::InteractionResult {
        cauldron_item_interaction(state, world, pos, player, hand, inv)
    }

    fn has_analog_output_signal(&self, _state: BlockStateId) -> bool {
        true
    }

    fn get_analog_output_signal(
        &self,
        state: BlockStateId,
        _world: &World,
        _pos: BlockPos,
        _direction: Direction,
    ) -> i32 {
        i32::from(state.get_value(LEVEL_CAULDRON))
    }

    fn is_pathfindable(
        &self,
        _state: BlockStateId,
        _computation_type: PathComputationType,
    ) -> bool {
        false
    }

    fn tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        let Some(stalactite_pos) =
            vegetation::find_stalactite_tip_above_cauldron(world.as_ref(), pos)
        else {
            return;
        };
        let Some(fluid) = vegetation::get_cauldron_fill_fluid_type(world, stalactite_pos) else {
            return;
        };

        if fluid == &vanilla_fluids::WATER {
            let level = state.get_value(LEVEL_CAULDRON);
            if level < LEVEL_CAULDRON.max {
                let new_state = state.set_value(LEVEL_CAULDRON, level + 1);
                world.set_block(pos, new_state, UpdateFlags::UPDATE_ALL);
                world.game_event(
                    &vanilla_game_events::BLOCK_CHANGE,
                    pos,
                    &GameEventContext::new(None, Some(new_state)),
                );
                world.level_event(level_events::SOUND_DRIP_WATER_INTO_CAULDRON, pos, 0, None);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_blocks};

    use super::*;
    use crate::{
        behavior::{BLOCK_BEHAVIORS, init_behaviors},
        test_support::fresh_test_world,
    };

    #[test]
    fn registered_cauldron_behaviors_expose_vanilla_fill_levels() {
        init_vanilla_registry();
        init_behaviors();
        let world = fresh_test_world("cauldron_analog_output");
        let pos = BlockPos::ZERO;

        let empty = vanilla_blocks::CAULDRON.default_state();
        let empty_behavior = BLOCK_BEHAVIORS.get_behavior(empty.get_block());
        assert!(empty_behavior.has_analog_output_signal(empty));
        assert_eq!(
            empty_behavior.get_analog_output_signal(
                empty,
                world.world.as_ref(),
                pos,
                Direction::North,
            ),
            0,
        );

        for level_value in 1..=3 {
            let state = vanilla_blocks::WATER_CAULDRON
                .default_state()
                .set_value(LEVEL_CAULDRON, level_value);
            let behavior = BLOCK_BEHAVIORS.get_behavior(state.get_block());
            assert_eq!(
                behavior.get_analog_output_signal(
                    state,
                    world.world.as_ref(),
                    pos,
                    Direction::North
                ),
                i32::from(level_value),
            );
        }
    }
}

/// Vanilla 26.3 CauldronInteractions dispatch, shared by all cauldron types.
pub(super) fn cauldron_item_interaction(
    state: BlockStateId, world: &Arc<World>, pos: BlockPos,
    player: &crate::player::Player, hand: steel_utils::types::InteractionHand,
    inv: &mut crate::behavior::InventoryAccess,
) -> crate::behavior::InteractionResult {
    use crate::{behavior::InteractionResult, fluid::FluidStateExt as _};
    use steel_registry::{vanilla_items as items, vanilla_potions, sound_events as sounds, vanilla_custom_stats as stats};
    use steel_registry::item_stack::ItemStack;
    let held = inv.with_item(|item| item.clone());
    let item = held.item();
    let water = state.get_block() == &vanilla_blocks::WATER_CAULDRON;
    let snow = state.get_block() == &vanilla_blocks::POWDER_SNOW_CAULDRON;
    let empty = state.get_block() == &vanilla_blocks::CAULDRON;
    let level = if water || snow { state.get_value(LEVEL_CAULDRON) } else { 0 };
    let lower = || if level <= 1 { vanilla_blocks::CAULDRON.default_state() }
        else { state.set_value(LEVEL_CAULDRON, level - 1) };
    let (result, next, sound, placing, stat, award_item, creative_limit) = if item == &items::WATER_BUCKET
        || item == &items::LAVA_BUCKET || item == &items::POWDER_SNOW_BUCKET {
        if item != &items::WATER_BUCKET && world.get_block_state(pos.above()).get_fluid_state().is_water() {
            return InteractionResult::Consume;
        }
        let (next, sound) = if item == &items::WATER_BUCKET {
            (vanilla_blocks::WATER_CAULDRON.default_state().set_value(LEVEL_CAULDRON, 3), &sounds::ITEM_BUCKET_EMPTY)
        } else if item == &items::LAVA_BUCKET {
            (vanilla_blocks::LAVA_CAULDRON.default_state(), &sounds::ITEM_BUCKET_EMPTY_LAVA)
        } else {
            (vanilla_blocks::POWDER_SNOW_CAULDRON.default_state().set_value(LEVEL_CAULDRON, 3), &sounds::ITEM_BUCKET_EMPTY_POWDER_SNOW)
        };
        (ItemStack::new(&items::BUCKET), next, Some(sound), true, &stats::FILL_CAULDRON, true, true)
    } else if item == &items::BUCKET && ((water || snow) && level == 3 || state.get_block() == &vanilla_blocks::LAVA_CAULDRON) {
        let (filled, sound) = if water { (&items::WATER_BUCKET, &sounds::ITEM_BUCKET_FILL) }
            else if snow { (&items::POWDER_SNOW_BUCKET, &sounds::ITEM_BUCKET_FILL_POWDER_SNOW) }
            else { (&items::LAVA_BUCKET, &sounds::ITEM_BUCKET_FILL_LAVA) };
        (ItemStack::new(filled), vanilla_blocks::CAULDRON.default_state(), Some(sound), false, &stats::USE_CAULDRON, true, true)
    } else if water && item == &items::GLASS_BOTTLE {
        (steel_registry::data_components::PotionContents::create_item_stack(&items::POTION, &vanilla_potions::WATER), lower(), Some(&sounds::ITEM_BOTTLE_FILL), false, &stats::USE_CAULDRON, true, true)
    } else if (empty || water && level < 3) && item == &items::POTION
        && held.get(steel_registry::data_components::vanilla_components::POTION_CONTENTS).is_some_and(|potion| potion.is(&vanilla_potions::WATER)) {
        (ItemStack::new(&items::GLASS_BOTTLE), vanilla_blocks::WATER_CAULDRON.default_state().set_value(LEVEL_CAULDRON, level + 1), Some(&sounds::ITEM_BOTTLE_EMPTY), true, &stats::USE_CAULDRON, true, true)
    } else if water && item != &items::SHULKER_BOX && item.block.is_some_and(|block| block.has_tag(&steel_registry::vanilla_block_tags::BlockTag::SHULKER_BOXES)) {
        (ItemStack::with_count_and_patch(&items::SHULKER_BOX, 1, held.components_patch().clone()), lower(), None, false, &stats::CLEAN_SHULKER_BOX, false, false)
    } else { return InteractionResult::TryEmptyHandInteraction; };
    crate::behavior::item_utils::create_filled_result_for_hand(player, inv, hand, result, creative_limit);
    if award_item { player.award_stat(&steel_registry::stat::vanilla_stat_types::ITEM_USED, item); }
    player.award_custom_stat(stat);
    world.set_block(pos, next, UpdateFlags::UPDATE_ALL);
    if let Some(sound) = sound {
        world.play_sound(sound, steel_protocol::packets::game::SoundSource::Blocks, pos, 1.0, 1.0, None);
        world.game_event(if placing { &vanilla_game_events::FLUID_PLACE } else { &vanilla_game_events::FLUID_PICKUP }, pos, &GameEventContext::new(None, None));
    }
    InteractionResult::Success
}
