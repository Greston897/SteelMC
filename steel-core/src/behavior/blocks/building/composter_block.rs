//! Composter interaction, maturation, extraction and analog output.
//! Item layer counts come from the MC 26.3 compostable context provider.

use std::sync::Arc;

use steel_macros::block_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::{REGISTRY, vanilla_blocks, vanilla_items, sound_events, vanilla_game_events, vanilla_stat_types};
use steel_registry::data_components::vanilla_components::COMPOSTABLE;
use steel_registry::context_provider::ProviderEvalContext;
use steel_registry::item_stack::ItemStack;
use steel_utils::random::{Random, legacy_random::LegacyRandom};
use steel_utils::types::{UpdateFlags, InteractionHand};
use crate::behavior::{InventoryAccess, InteractionResult, BlockHitResult};
use crate::player::Player;
use crate::world::{ScheduledTickAccess, game_event::GameEventContext};
use steel_registry::blocks::{
    BlockRef,
    block_state_ext::BlockStateExt as _,
    properties::{BlockStateProperties, Direction, IntProperty},
};
use steel_utils::{BlockPos, BlockStateId};

use crate::{
    behavior::{BlockBehavior, BlockPlaceContext},
    entity::ai::path::PathComputationType,
    world::World,
};

/// State-driven composter; automation uses the same insert/empty operations.
#[block_behavior]
pub struct ComposterBlock {
    block: BlockRef,
}

const LEVEL_COMPOSTER: &IntProperty = &BlockStateProperties::LEVEL_COMPOSTER;

impl ComposterBlock {
    /// Creates composter behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
    /// Attempts one automated insertion. Accepted items are consumed by the caller,
    /// including unsuccessful probabilistic layer rolls, as in vanilla.
    pub(crate) fn insert(world: &Arc<World>, pos: BlockPos, stack: &ItemStack) -> bool {
        let mut random = LegacyRandom::new(rand::random());
        Self::insert_with_random(world, pos, stack, &mut random)
    }

    fn insert_with_random(world: &Arc<World>, pos: BlockPos, stack: &ItemStack, random: &mut impl Random) -> bool {
        let state = world.get_block_state(pos);
        let level = state.get_value(LEVEL_COMPOSTER);
        if state.get_block() != &vanilla_blocks::COMPOSTER || level >= 7 || stack.is_empty() {
            return false;
        }
        let Some(component) = stack.get(COMPOSTABLE) else { return false; };
        let level_text = level.to_string();
        let properties = [("level", level_text.as_str())];
        let context = ProviderEvalContext::for_block_state(&state.get_block().key, &properties);
        let layers = REGISTRY.context_int_providers.sample(&component.layers, &context, random).unwrap_or(0);
        if layers > 0 {
            let next_level = (i32::from(level).saturating_add(layers)).clamp(0, 7) as u8;
            let next = state.set_value(LEVEL_COMPOSTER, next_level);
            world.set_block(pos, next, UpdateFlags::UPDATE_ALL);
            world.game_event(&vanilla_game_events::BLOCK_CHANGE, pos, &GameEventContext::new(None, Some(next)));
            if next_level == 7 {
                world.schedule_block_tick_default(pos, state.get_block(), 20);
            }
        }
        world.level_event(1500, pos, i32::from(layers > 0), None);
        true
    }

    /// Empties a ready composter without spawning loot (hopper extraction).
    pub(crate) fn empty(world: &Arc<World>, pos: BlockPos) -> bool {
        let state = world.get_block_state(pos);
        if state.get_block() != &vanilla_blocks::COMPOSTER || state.get_value(LEVEL_COMPOSTER) != 8 {
            return false;
        }
        let next = state.set_value(LEVEL_COMPOSTER, 0);
        world.set_block(pos, next, UpdateFlags::UPDATE_ALL);
        world.game_event(&vanilla_game_events::BLOCK_CHANGE, pos, &GameEventContext::new(None, Some(next)));
        true
    }
}

impl BlockBehavior for ComposterBlock {
    fn on_place(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos, _old_state: BlockStateId, _moved_by_piston: bool) {
        if state.get_value(LEVEL_COMPOSTER) == 7 {
            world.schedule_block_tick_default(pos, self.block, 20);
        }
    }

    fn tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        if state.get_value(LEVEL_COMPOSTER) == 7 {
            world.set_block(pos, state.set_value(LEVEL_COMPOSTER, 8), UpdateFlags::UPDATE_ALL);
            world.play_sound(&sound_events::BLOCK_COMPOSTER_READY, SoundSource::Blocks, pos, 1.0, 1.0, None);
        }
    }

    fn use_item_on(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos, player: &Player, _hand: InteractionHand, _hit: &BlockHitResult, inv: &mut InventoryAccess) -> InteractionResult {
        let stack = inv.with_item(|stack| stack.clone());
        let level = state.get_value(LEVEL_COMPOSTER);
        if level < 8 && stack.get(COMPOSTABLE).is_some() {
            if level < 7 && Self::insert(world, pos, &stack) {
                inv.with_item(|stack| stack.consume_one(player.has_infinite_materials()));
                player.award_stat(&vanilla_stat_types::ITEM_USED, stack.item());
            }
            return InteractionResult::Success;
        }
        InteractionResult::TryEmptyHandInteraction
    }

    fn use_without_item(&self, _state: BlockStateId, world: &Arc<World>, pos: BlockPos, _player: &Player, _hit: &BlockHitResult, _inv: &mut InventoryAccess) -> InteractionResult {
        if !Self::empty(world, pos) {
            return InteractionResult::Pass;
        }
        world.drop_item_stack(pos.above(), ItemStack::new(&vanilla_items::BONE_MEAL));
        world.play_sound(&sound_events::BLOCK_COMPOSTER_EMPTY, SoundSource::Blocks, pos, 1.0, 1.0, None);
        InteractionResult::Success
    }

    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
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
        i32::from(state.get_value(LEVEL_COMPOSTER))
    }

    fn is_pathfindable(
        &self,
        _state: BlockStateId,
        _computation_type: PathComputationType,
    ) -> bool {
        false
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

    use steel_utils::{ChunkPos, Downcast as _};
    use crate::bootstrap::init_globals;
    use crate::block_entity::BlockEntity as _;
    use crate::block_entity::entities::HopperBlockEntity;
    use crate::inventory::container::Container as _;
    use crate::test_support::{insert_ready_full_chunk, tick_test_world, TestWorld};

    const POS: BlockPos = BlockPos::new(8, 64, 8);

    fn fixture(name: &str, level: u8) -> TestWorld {
        init_globals();
        let fixture = fresh_test_world(name);
        insert_ready_full_chunk(&fixture.world, ChunkPos::from_block_pos(POS));
        fixture.world.set_block(POS, vanilla_blocks::COMPOSTER.default_state().set_value(LEVEL_COMPOSTER, level), UpdateFlags::UPDATE_ALL);
        fixture
    }

    fn level(world: &World) -> u8 {
        world.get_block_state(POS).get_value(LEVEL_COMPOSTER)
    }

    #[test]
    fn first_layer_uses_context_provider_for_low_chance_compostable_items() {
        let fixture = fixture("composter_first_layer", 0);
        let stack = ItemStack::new(&vanilla_items::WHEAT_SEEDS);
        for seed in 0..64 {
            fixture.world.set_block(POS, vanilla_blocks::COMPOSTER.default_state(), UpdateFlags::UPDATE_ALL);
            assert!(ComposterBlock::insert_with_random(&fixture.world, POS, &stack, &mut LegacyRandom::new(seed)));
            assert_eq!(level(&fixture.world), 1, "first layer at random seed {seed}");
        }
    }

    #[test]
    fn cake_fills_seven_layers_then_waits_exactly_twenty_world_ticks() {
        let fixture = fixture("composter_maturation", 0);
        let world = &fixture.world;
        let cake = ItemStack::new(&vanilla_items::CAKE);
        for expected in 1..=7 {
            assert!(ComposterBlock::insert(world, POS, &cake));
            assert_eq!(level(world), expected);
        }
        assert!(!ComposterBlock::insert(world, POS, &cake));
        for tick in 0..19 {
            tick_test_world(world, tick, true);
            assert_eq!(level(world), 7);
        }
        tick_test_world(world, 19, true);
        assert_eq!(level(world), 8);
        assert!(!ComposterBlock::insert(world, POS, &cake));
        assert!(ComposterBlock::empty(world, POS));
        assert_eq!(level(world), 0);
        assert!(!ComposterBlock::empty(world, POS));
    }

    #[test]
    fn non_compostable_item_is_not_accepted_at_any_level() {
        let fixture = fixture("composter_rejects_stone", 0);
        let stone = ItemStack::new(&vanilla_items::STONE);
        for input in 0..=8 {
            fixture.world.set_block(POS, vanilla_blocks::COMPOSTER.default_state().set_value(LEVEL_COMPOSTER, input), UpdateFlags::UPDATE_ALL);
            assert!(!ComposterBlock::insert(&fixture.world, POS, &stone));
            assert_eq!(level(&fixture.world), input);
        }
    }

    #[test]
    fn hopper_above_composts_one_item_and_hopper_below_extracts_once() {
        let fixture = fixture("composter_hopper_pipeline", 0);
        let world = &fixture.world;
        let top = POS.above();
        world.set_block(top, vanilla_blocks::HOPPER.default_state(), UpdateFlags::UPDATE_ALL);
        let entity = world.get_block_entity(top).unwrap();
        let hopper = entity.downcast_ref::<HopperBlockEntity>().unwrap();
        hopper.container_ref().unwrap().lock().set_item(0, ItemStack::with_count(&vanilla_items::CAKE, 8));
        for expected in 1..=7 {
            assert!(hopper.try_move_items(world, top));
            assert_eq!(level(world), expected);
        }
        assert!(!hopper.try_move_items(world, top));
        assert_eq!(hopper.container_ref().unwrap().lock().get_item(0).count(), 1);
        let behavior = BLOCK_BEHAVIORS.get_behavior(&vanilla_blocks::COMPOSTER);
        behavior.tick(world.get_block_state(POS), world, POS);
        let bottom = POS.below();
        world.set_block(bottom, vanilla_blocks::HOPPER.default_state(), UpdateFlags::UPDATE_ALL);
        let entity = world.get_block_entity(bottom).unwrap();
        let hopper = entity.downcast_ref::<HopperBlockEntity>().unwrap();
        assert!(hopper.suck_in_items(world, bottom));
        assert_eq!(level(world), 0);
        let container = hopper.container_ref().unwrap();
        assert_eq!(container.lock().get_item(0).count(), 1);
        assert!(container.lock().get_item(0).is(&vanilla_items::BONE_MEAL));
        assert!(!hopper.suck_in_items(world, bottom));
    }

    #[test]
    fn full_hopper_cannot_delete_composter_produce() {
        let fixture = fixture("composter_full_hopper", 8);
        let world = &fixture.world;
        let bottom = POS.below();
        world.set_block(bottom, vanilla_blocks::HOPPER.default_state(), UpdateFlags::UPDATE_ALL);
        let entity = world.get_block_entity(bottom).unwrap();
        let hopper = entity.downcast_ref::<HopperBlockEntity>().unwrap();
        let container = hopper.container_ref().unwrap();
        for slot in 0..5 { container.lock().set_item(slot, ItemStack::with_count(&vanilla_items::STONE, 64)); }
        assert!(!hopper.suck_in_items(world, bottom));
        assert_eq!(level(world), 8);
    }

    #[test]
    fn horizontal_hopper_does_not_compost_through_side() {
        let fixture = fixture("composter_side_hopper", 0);
        let world = &fixture.world;
        let side = POS.west();
        world.set_block(side, vanilla_blocks::HOPPER.default_state().set_value(&BlockStateProperties::FACING_HOPPER, Direction::East), UpdateFlags::UPDATE_ALL);
        let entity = world.get_block_entity(side).unwrap();
        let hopper = entity.downcast_ref::<HopperBlockEntity>().unwrap();
        hopper.container_ref().unwrap().lock().set_item(0, ItemStack::new(&vanilla_items::CAKE));
        assert!(!hopper.try_move_items(world, side));
        assert_eq!(level(world), 0);
    }

    #[test]
    fn registered_composter_outputs_its_full_state_level() {
        init_vanilla_registry();
        init_behaviors();
        let world = fresh_test_world("composter_analog_output");
        let state = vanilla_blocks::COMPOSTER
            .default_state()
            .set_value(LEVEL_COMPOSTER, LEVEL_COMPOSTER.max);
        let behavior = BLOCK_BEHAVIORS.get_behavior(state.get_block());

        assert!(behavior.has_analog_output_signal(state));
        assert_eq!(
            behavior.get_analog_output_signal(
                state,
                world.world.as_ref(),
                BlockPos::ZERO,
                Direction::North,
            ),
            i32::from(LEVEL_COMPOSTER.max),
        );
    }
}
