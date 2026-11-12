//! Empty map item behaviour.
//!
//! Port of vanilla `EmptyMapItem`: using an empty map in air consumes one item,
//! plays `ui.cartography_table.take_result`, awards `minecraft:used` and turns
//! the consumed slot into a fresh scale-0 filled map centred on the player.

use steel_macros::item_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::item_stack::ItemStack;
use steel_registry::sound_events;
use steel_registry::stat::vanilla_stat_types;

use crate::behavior::items::filled_map::FilledMapItem;
use crate::behavior::{InteractionResult, ItemBehavior, UseItemContext};
use crate::entity::Entity as _;
use crate::inventory::container::Container as _;
use crate::player::player_inventory::Prediction;

/// Vanilla `EmptyMapItem`.
#[item_behavior(class = "EmptyMapItem")]
pub struct EmptyMapItem;

impl ItemBehavior for EmptyMapItem {
    fn use_item(&self, context: &mut UseItemContext) -> InteractionResult {
        let world = context.world;
        let player = context.player;
        let has_infinite_materials = player.has_infinite_materials();
        let hand = context.hand;
        let used_item = context.inv.with_item(|stack| stack.item());
        let position = player.block_position();

        world.play_sound(
            &sound_events::UI_CARTOGRAPHY_TABLE_TAKE_RESULT,
            SoundSource::Players,
            position,
            1.0,
            1.0,
            None,
        );

        // Vanilla creates the map before touching the inventory so a full
        // inventory still consumes the empty map and drops the new one.
        let filled_map =
            FilledMapItem::create_map(world, position.x(), position.z(), 0, true, false);

        let leftover = context.inv.with_inventory(|inventory| {
            // Consumed stack is discarded: an empty map has no use remainder.
            let _consumed = inventory.mutate_item_in_hand(hand, |stack| {
                stack.consume_and_return(1, has_infinite_materials)
            });

            let mut filled_map = filled_map;
            if inventory.get_item_in_hand(hand).is_empty() {
                // Vanilla `ActionResult.SUCCESS.withNewHandStack(filledMap)`.
                inventory.set_item_in_hand(hand, filled_map);
                return ItemStack::empty();
            }

            if inventory.add(&mut filled_map) {
                ItemStack::empty()
            } else {
                filled_map
            }
        });

        if !leftover.is_empty() {
            let _ = player.drop_item(leftover, false, Prediction::Predicted);
        }

        player.award_stat(&vanilla_stat_types::ITEM_USED, used_item);
        InteractionResult::Success
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use steel_registry::data_components::vanilla_components::MAP_ID;
    use steel_registry::{init_vanilla_registry, vanilla_items};
    use steel_utils::ChunkPos;
    use steel_utils::types::InteractionHand;

    use super::EmptyMapItem;
    use crate::behavior::InteractionResult;
    use crate::behavior::ItemBehavior as _;
    use crate::behavior::init_behaviors;
    use crate::test_support::{TestPlayerBuilder, fresh_test_world, insert_ready_full_chunk};
    use steel_registry::item_stack::ItemStack;

    #[test]
    fn empty_map_becomes_a_filled_map_in_the_same_hand() {
        init_vanilla_registry();
        init_behaviors();
        let world_fixture = fresh_test_world("empty_map_single");
        let world = &world_fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));

        let player = TestPlayerBuilder::new(Arc::clone(world), "MapTester", 1).build();
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::new(&vanilla_items::MAP));

        let mut context = crate::behavior::UseItemContext::new(
            &player,
            InteractionHand::MainHand,
            world,
            Arc::clone(&player.inventory),
        );

        let result = EmptyMapItem.use_item(&mut context);
        assert_eq!(result, InteractionResult::Success);

        let hand = player.inventory.lock().get_selected_item().clone();
        assert!(hand.is(&vanilla_items::FILLED_MAP));
        let map_id = hand.get(MAP_ID).expect("filled map carries an id").id();

        let (scale, center_x, center_z, stored) = {
            let maps = world.map_states.lock();
            let state = maps.get(map_id).expect("map state registered");
            (
                state.scale(),
                state.center_x(),
                state.center_z(),
                maps.len(),
            )
        };
        assert_eq!(scale, 0);
        assert_eq!(stored, 1);
        // Empty maps centre on the player's block and snap to the scale grid.
        assert_eq!(center_x % 128, 0);
        assert_eq!(center_z % 128, 0);
    }

    #[test]
    fn stacked_empty_maps_keep_the_remainder_and_insert_the_new_map() {
        init_vanilla_registry();
        init_behaviors();
        let world_fixture = fresh_test_world("empty_map_stack");
        let world = &world_fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));

        let player = TestPlayerBuilder::new(Arc::clone(world), "MapStacker", 1).build();
        {
            let mut inventory = player.inventory.lock();
            let mut stack = ItemStack::new(&vanilla_items::MAP);
            stack.set_count(2);
            inventory.set_selected_item(stack);
        }

        let mut context = crate::behavior::UseItemContext::new(
            &player,
            InteractionHand::MainHand,
            world,
            Arc::clone(&player.inventory),
        );
        let result = EmptyMapItem.use_item(&mut context);
        assert_eq!(result, InteractionResult::Success);

        let inventory = player.inventory.lock();
        assert!(inventory.get_selected_item().is(&vanilla_items::MAP));
        assert_eq!(inventory.get_selected_item().count(), 1);
        let filled_maps = inventory
            .get_items()
            .iter()
            .filter(|stack| stack.is(&vanilla_items::FILLED_MAP))
            .count();
        assert_eq!(filled_maps, 1);
    }
}
