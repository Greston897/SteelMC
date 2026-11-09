//! Writable and written book item behaviors.
//!
//! Mirrors vanilla `WritableBookItem` and `WrittenBookItem`: both award the
//! `item_used` statistic and use the book, and only a stack carrying
//! `written_book_content` makes the server open the reading screen. A writable
//! book's editor is opened by the client itself, which is why the server sends
//! no packet for it.

use steel_macros::item_behavior;
use steel_protocol::packets::game::COpenBook;
use steel_registry::data_components::vanilla_components::WRITTEN_BOOK_CONTENT;
use steel_registry::stat::vanilla_stat_types;

use crate::behavior::context::{InteractionResult, UseItemContext};
use crate::behavior::item::ItemBehavior;

/// Behavior for the `writable_book` item (`WritableBookItem`).
#[item_behavior(class = "WritableBookItem")]
pub struct WritableBookItem;

impl ItemBehavior for WritableBookItem {
    fn use_item(&self, context: &mut UseItemContext) -> InteractionResult {
        // Vanilla `ServerPlayerEntity.useBook` ignores writable books; the
        // client opens the editor locally after the successful use.
        context.inv.with_item(|item| {
            context
                .player
                .award_stat(&vanilla_stat_types::ITEM_USED, item.item());
        });
        InteractionResult::Success
    }
}

/// Behavior for the `written_book` item (`WrittenBookItem`).
#[item_behavior(class = "WrittenBookItem")]
pub struct WrittenBookItem;

impl ItemBehavior for WrittenBookItem {
    fn use_item(&self, context: &mut UseItemContext) -> InteractionResult {
        // Vanilla `WrittenBookItem.use` -> `ServerPlayerEntity.useBook`, which
        // opens the reading screen only when the stack carries written content.
        let has_content = context
            .inv
            .with_item(|item| item.has(WRITTEN_BOOK_CONTENT));
        if has_content {
            context.player.send_packet(COpenBook::new(context.hand));
        }
        context.inv.with_item(|item| {
            context
                .player
                .award_stat(&vanilla_stat_types::ITEM_USED, item.item());
        });
        InteractionResult::Success
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use steel_registry::item_stack::ItemStack;
    use steel_registry::{init_vanilla_registry, vanilla_items};
    use steel_utils::ChunkPos;
    use steel_utils::types::InteractionHand;

    use crate::behavior::{InteractionResult, ITEM_BEHAVIORS, UseItemContext, init_behaviors};
    use crate::inventory::container::Container as _;
    use crate::player::Player;
    use crate::test_support::{
        TestPlayerBuilder, TestWorld, fresh_test_world, insert_ready_full_chunk,
    };
    use crate::world::World;

    fn fixture(world_name: &'static str) -> (TestWorld, Arc<Player>) {
        init_vanilla_registry();
        init_behaviors();
        let world_fixture = fresh_test_world(world_name);
        insert_ready_full_chunk(&world_fixture.world, ChunkPos::new(0, 0));
        let player = TestPlayerBuilder::new(Arc::clone(&world_fixture.world), "Writer", 1).build();
        (world_fixture, player)
    }

    fn use_main_hand(
        world: &Arc<World>,
        player: &Arc<Player>,
        stack: ItemStack,
        item: steel_registry::items::ItemRef,
    ) -> InteractionResult {
        player
            .inventory
            .lock()
            .set_item_in_hand(InteractionHand::MainHand, stack);
        let behavior = ITEM_BEHAVIORS.get_behavior(item);
        let inventory = Arc::clone(&player.inventory);
        let mut context = UseItemContext::new(player, InteractionHand::MainHand, world, inventory);
        behavior.use_item(&mut context)
    }

    fn held_count(player: &Arc<Player>) -> i32 {
        player
            .inventory
            .lock()
            .get_item_in_hand(InteractionHand::MainHand)
            .count()
    }

    #[test]
    fn writable_book_use_succeeds_without_consuming_the_stack() {
        let (fixture, player) = fixture("writable_book_use");
        let result = use_main_hand(
            &fixture.world,
            &player,
            ItemStack::new(&vanilla_items::WRITABLE_BOOK),
            &vanilla_items::WRITABLE_BOOK,
        );

        assert_eq!(result, InteractionResult::Success);
        assert_eq!(held_count(&player), 1);
    }

    #[test]
    fn written_book_use_succeeds_without_consuming_the_stack() {
        let (fixture, player) = fixture("written_book_use");
        let result = use_main_hand(
            &fixture.world,
            &player,
            ItemStack::new(&vanilla_items::WRITTEN_BOOK),
            &vanilla_items::WRITTEN_BOOK,
        );

        assert_eq!(result, InteractionResult::Success);
        assert_eq!(held_count(&player), 1);
    }
}
