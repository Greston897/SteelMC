//! Item frame and glow item frame placement (`ItemFrameItem`).
//!
//! Mirrors vanilla `ItemFrameItem` / `DecorationItem.useOnBlock`: the frame is
//! spawned on the side of the clicked block, refuses to appear when its support
//! check fails, copies the stack's custom name, plays the vanilla place sound,
//! emits `ENTITY_PLACE` and consumes exactly one item.

use std::sync::Arc;

use steel_macros::item_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::{vanilla_entities, vanilla_game_events, vanilla_items};
use steel_utils::Direction;

use crate::behavior::context::{InteractionResult, UseOnContext};
use crate::behavior::item::ItemBehavior;
use crate::entity::entities::ItemFrameEntity;
use crate::entity::{Entity, SharedEntity, next_entity_id};
use crate::world::game_event::GameEventContext;

/// Placement behavior shared by the item frame and the glow item frame.
#[item_behavior(class = "ItemFrameItem")]
pub struct ItemFrameItem;

impl ItemFrameItem {
    /// Vanilla picks the entity type from the item that places the frame.
    fn entity_type(stack: &steel_registry::item_stack::ItemStack) -> EntityTypeRef {
        if stack.is(&vanilla_items::GLOW_ITEM_FRAME) {
            &vanilla_entities::GLOW_ITEM_FRAME
        } else {
            &vanilla_entities::ITEM_FRAME
        }
    }

    /// Vanilla `ItemFrameItem#canPlaceOn`: inside the height limit and allowed by
    /// the player's placement rules.
    fn can_place_on(
        context: &UseOnContext<'_>,
        stack: &steel_registry::item_stack::ItemStack,
        side: Direction,
        target: steel_utils::BlockPos,
    ) -> bool {
        target.y() < context.world.max_build_height()
            && context
                .player
                .can_place_on(context.world, target, side, stack)
    }
}

impl ItemBehavior for ItemFrameItem {
    fn use_on(&self, context: &mut UseOnContext) -> InteractionResult {
        let side = context.hit_result.direction;
        let (dx, dy, dz) = side.offset();
        let target = context.hit_result.block_pos.offset(dx, dy, dz);

        let stack = context.inv.with_item(|stack| stack.copy_with_count(1));

        if !Self::can_place_on(context, &stack, side, target) {
            return InteractionResult::Fail;
        }

        let frame = ItemFrameEntity::new_attached(
            Self::entity_type(&stack),
            next_entity_id(),
            target,
            side,
            Arc::downgrade(context.world),
        );

        if let Some(custom_name) = stack.custom_name() {
            frame.set_custom_name(Some(custom_name.into_owned()));
        }

        // Vanilla returns `CONSUME` instead of spawning when the frame cannot stay.
        if !frame.can_stay_attached() {
            return InteractionResult::Consume;
        }

        let sound = frame.place_sound();
        let entity: SharedEntity = Arc::new(frame);
        if context.world.try_add_entity(entity).is_err() {
            return InteractionResult::Fail;
        }

        context
            .world
            .play_sound(sound, SoundSource::Neutral, target, 1.0, 1.0, None);
        context.world.game_event(
            &vanilla_game_events::ENTITY_PLACE,
            target,
            &GameEventContext::new(Some(&**context.player as &dyn Entity), None),
        );
        context.inv.with_item(|stack| stack.consume_one(false));

        InteractionResult::Success
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use glam::DVec3;
    use steel_registry::data_components::vanilla_components::CUSTOM_NAME;
    use steel_registry::items::item::BlockHitResult;
    use steel_registry::{
        init_vanilla_registry, item_stack::ItemStack, vanilla_blocks, vanilla_items,
    };
    use steel_utils::types::{InteractionHand, UpdateFlags};
    use steel_utils::{BlockPos, ChunkPos, Direction};
    use text_components::TextComponent;

    use super::ItemFrameItem;
    use crate::behavior::ItemBehavior as _;
    use crate::behavior::context::{InteractionResult, UseOnContext};
    use crate::behavior::init_behaviors;

    use crate::test_support::{TestPlayerBuilder, fresh_test_world, insert_ready_full_chunk};
    use crate::world::World;

    const WALL_X: i32 = 0;

    /// Builds a world with a stone wall at `WALL_X` and returns it with the player.
    fn wall_fixture(
        name: &str,
    ) -> (
        crate::test_support::TestWorld,
        Arc<crate::player::Player>,
        BlockPos,
    ) {
        init_vanilla_registry();
        init_behaviors();

        let world_fixture = fresh_test_world(name);
        let world = Arc::clone(&world_fixture.world);
        let wall = BlockPos::new(WALL_X, 64, 0);
        // A real server keeps the neighbouring chunks loaded; frames can be
        // placed on any side of the support block, so load the 3x3 area.
        for dx in -1..=1 {
            for dz in -1..=1 {
                insert_ready_full_chunk(
                    &world,
                    ChunkPos::from_block_pos(wall.offset(dx * 16, 0, dz * 16)),
                );
            }
        }
        assert!(world.set_block(
            wall,
            vanilla_blocks::STONE.default_state(),
            UpdateFlags::UPDATE_ALL
        ));

        let player = TestPlayerBuilder::new(Arc::clone(&world), "FrameTester", 1).build();
        player.set_client_loaded(true);

        (world_fixture, player, wall)
    }

    fn place_context<'a>(
        world: &'a Arc<World>,
        player: &'a Arc<crate::player::Player>,
        wall: BlockPos,
        side: Direction,
    ) -> UseOnContext<'a> {
        UseOnContext::new(
            player,
            InteractionHand::MainHand,
            BlockHitResult {
                location: DVec3::new(0.5, 64.5, 0.5),
                direction: side,
                block_pos: wall,
                miss: false,
                inside: false,
                world_border_hit: false,
            },
            world,
            Arc::clone(&player.inventory),
        )
    }

    fn held_stack(player: &Arc<crate::player::Player>) -> ItemStack {
        player.inventory.lock().get_selected_item().clone()
    }

    fn frames_in_world(world: &Arc<World>) -> usize {
        world
            .get_entities_in_aabb(&steel_utils::WorldAabb::new(
                -2.0, 62.0, -2.0, 2.0, 66.0, 2.0,
            ))
            .iter()
            .filter(|entity| entity.as_item_frame().is_some())
            .count()
    }

    #[test]
    fn placing_a_frame_spawns_it_on_the_clicked_face_and_consumes_one() {
        let (fixture, player, wall) = wall_fixture("frame_place_ok");
        let world = &fixture.world;
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::with_count(&vanilla_items::ITEM_FRAME, 3));

        let mut context = place_context(world, &player, wall, Direction::Up);
        assert_eq!(
            ItemFrameItem.use_on(&mut context),
            InteractionResult::Success
        );

        assert_eq!(frames_in_world(world), 1);
        assert_eq!(held_stack(&player).count(), 2);

        let frame = world
            .get_entities_in_aabb(&steel_utils::WorldAabb::new(
                -2.0, 62.0, -2.0, 2.0, 66.0, 2.0,
            ))
            .into_iter()
            .find(|entity| entity.as_item_frame().is_some())
            .expect("frame spawned");
        let frame = frame.as_item_frame().expect("frame downcast");
        assert_eq!(frame.direction(), Direction::Up);
    }

    #[test]
    fn glow_frames_use_the_glow_entity_type() {
        let (fixture, player, wall) = wall_fixture("frame_place_glow");
        let world = &fixture.world;
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::new(&vanilla_items::GLOW_ITEM_FRAME));

        let mut context = place_context(world, &player, wall, Direction::North);
        assert_eq!(
            ItemFrameItem.use_on(&mut context),
            InteractionResult::Success
        );

        let entity = world
            .get_entities_in_aabb(&steel_utils::WorldAabb::new(
                -2.0, 62.0, -2.0, 2.0, 66.0, 2.0,
            ))
            .into_iter()
            .find(|entity| entity.as_item_frame().is_some())
            .expect("frame spawned");
        assert_eq!(
            entity.entity_type(),
            &steel_registry::vanilla_entities::GLOW_ITEM_FRAME
        );
        assert_eq!(held_stack(&player).count(), 0);
    }

    #[test]
    fn frames_are_not_placed_without_solid_support() {
        let (fixture, player, wall) = wall_fixture("frame_place_no_support");
        let world = &fixture.world;
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::with_count(&vanilla_items::ITEM_FRAME, 2));

        // Vanilla `BlockState#isSolid` needs a full cube; a slab is not solid.
        assert!(world.set_block(
            wall,
            vanilla_blocks::OAK_SLAB.default_state(),
            UpdateFlags::UPDATE_ALL
        ));

        let mut context = place_context(world, &player, wall, Direction::Up);
        assert_eq!(
            ItemFrameItem.use_on(&mut context),
            InteractionResult::Consume
        );

        assert_eq!(frames_in_world(world), 0);
        assert_eq!(held_stack(&player).count(), 2);
    }

    #[test]
    fn frames_are_not_placed_into_occupied_space() {
        let (fixture, player, wall) = wall_fixture("frame_place_occupied");
        let world = &fixture.world;
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::new(&vanilla_items::ITEM_FRAME));

        let occupied = wall.offset(0, 1, 0);
        assert!(world.set_block(
            occupied,
            vanilla_blocks::STONE.default_state(),
            UpdateFlags::UPDATE_ALL
        ));

        let mut context = place_context(world, &player, wall, Direction::Up);
        assert_eq!(
            ItemFrameItem.use_on(&mut context),
            InteractionResult::Consume
        );
        assert_eq!(frames_in_world(world), 0);
        assert_eq!(held_stack(&player).count(), 1);
    }

    #[test]
    fn frames_copy_the_stack_custom_name() {
        let (fixture, player, wall) = wall_fixture("frame_place_custom_name");
        let world = &fixture.world;

        let mut stack = ItemStack::new(&vanilla_items::ITEM_FRAME);
        stack.set(CUSTOM_NAME, TextComponent::plain("Wall Frame"));
        player.inventory.lock().set_selected_item(stack);

        let mut context = place_context(world, &player, wall, Direction::Up);
        assert_eq!(
            ItemFrameItem.use_on(&mut context),
            InteractionResult::Success
        );

        let entity = world
            .get_entities_in_aabb(&steel_utils::WorldAabb::new(
                -2.0, 62.0, -2.0, 2.0, 66.0, 2.0,
            ))
            .into_iter()
            .find(|entity| entity.as_item_frame().is_some())
            .expect("frame spawned");
        assert_eq!(
            entity.custom_name(),
            Some(TextComponent::plain("Wall Frame"))
        );
    }
}
