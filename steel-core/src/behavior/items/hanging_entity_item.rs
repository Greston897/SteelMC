//! Painting placement (`HangingEntityItem`).
//!
//! Mirrors vanilla `DecorationItem#useOnBlock` for the painting entity type: the
//! painting is spawned on the side of the clicked block, the variant is chosen by
//! vanilla `PaintingEntity.placePainting`, the vanilla place sound and
//! `ENTITY_PLACE` game event are emitted, and exactly one item is consumed.

use std::sync::Arc;

use steel_macros::item_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::item_stack::ItemStack;
use steel_registry::{sound_events, vanilla_entities, vanilla_game_events};
use steel_utils::{BlockPos, Direction};

use crate::behavior::context::{InteractionResult, UseOnContext};
use crate::behavior::item::ItemBehavior;
use crate::entity::entities::PaintingEntity;
use crate::entity::{Entity, SharedEntity, next_entity_id};
use crate::world::game_event::GameEventContext;

/// Placement behavior of the painting item.
#[item_behavior(class = "HangingEntityItem")]
pub struct HangingEntityItem;

impl HangingEntityItem {
    /// Vanilla `DecorationItem#canPlaceOn`: hanging entities only attach to the
    /// four horizontal faces and must respect the player's placement rules.
    fn can_place_on(
        context: &UseOnContext<'_>,
        stack: &ItemStack,
        side: Direction,
        target: BlockPos,
    ) -> bool {
        side.is_horizontal()
            && target.y() < context.world.max_build_height()
            && context
                .player
                .can_place_on(context.world, target, side, stack)
    }
}

impl ItemBehavior for HangingEntityItem {
    fn use_on(&self, context: &mut UseOnContext) -> InteractionResult {
        let side = context.hit_result.direction;
        let (dx, dy, dz) = side.offset();
        let target = context.hit_result.block_pos.offset(dx, dy, dz);

        let stack = context
            .inv
            .with_item(|stack: &mut steel_registry::item_stack::ItemStack| stack.clone());

        if !Self::can_place_on(context, &stack, side, target) {
            return InteractionResult::Fail;
        }

        // Vanilla `placePainting` also filters variants that do not fit.
        let Some(painting) = PaintingEntity::place_painting(
            &vanilla_entities::PAINTING,
            next_entity_id(),
            target,
            side,
            Arc::downgrade(context.world),
        ) else {
            return InteractionResult::Consume;
        };

        if let Some(custom_name) = stack.custom_name() {
            painting.set_custom_name(Some(custom_name.into_owned()));
        }

        // Vanilla returns `CONSUME` when the decoration cannot stay attached.
        if !painting.can_stay_attached() {
            return InteractionResult::Consume;
        }

        let entity: SharedEntity = Arc::new(painting);
        if context.world.try_add_entity(entity).is_err() {
            return InteractionResult::Fail;
        }

        context.world.play_sound(
            &sound_events::ENTITY_PAINTING_PLACE,
            SoundSource::Neutral,
            target,
            1.0,
            1.0,
            None,
        );
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
    use steel_registry::item_stack::ItemStack;
    use steel_registry::items::item::BlockHitResult;
    use steel_registry::{init_vanilla_registry, vanilla_blocks, vanilla_items};
    use steel_utils::types::{InteractionHand, UpdateFlags};
    use steel_utils::{BlockPos, ChunkPos, Direction, Downcast as _, WorldAabb};
    use text_components::TextComponent;

    use super::HangingEntityItem;
    use crate::behavior::ItemBehavior as _;
    use crate::behavior::context::{InteractionResult, UseOnContext};
    use crate::behavior::init_behaviors;
    use crate::entity::SharedEntity;
    use crate::entity::entities::PaintingEntity;
    use crate::test_support::{
        TestPlayerBuilder, TestWorld, fresh_test_world, insert_ready_full_chunk,
    };
    use crate::world::World;

    /// Builds a world with a stone wall at `(0, 64, 0)`.
    fn wall_fixture(name: &str) -> (TestWorld, Arc<crate::player::Player>, BlockPos) {
        init_vanilla_registry();
        init_behaviors();

        let world_fixture = fresh_test_world(name);
        let world = Arc::clone(&world_fixture.world);
        let wall = BlockPos::new(0, 64, 0);
        insert_ready_full_chunk(&world, ChunkPos::from_block_pos(wall));
        assert!(world.set_block(
            wall,
            vanilla_blocks::STONE.default_state(),
            UpdateFlags::UPDATE_ALL
        ));

        let player = TestPlayerBuilder::new(Arc::clone(&world), "Painter", 1).build();
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
                location: DVec3::new(1.0, 64.5, 0.5),
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

    fn paintings_in_world(world: &Arc<World>) -> Vec<SharedEntity> {
        world
            .get_entities_in_aabb(&WorldAabb::new(-4.0, 60.0, -4.0, 4.0, 72.0, 4.0))
            .into_iter()
            .filter(|entity| entity.downcast_ref::<PaintingEntity>().is_some())
            .collect()
    }

    #[test]
    fn placing_a_painting_consumes_one_item() {
        let (fixture, player, wall) = wall_fixture("painting_place_ok");
        let world = &fixture.world;
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::with_count(&vanilla_items::PAINTING, 2));

        let mut context = place_context(world, &player, wall, Direction::East);
        assert_eq!(
            HangingEntityItem.use_on(&mut context),
            InteractionResult::Success
        );

        let paintings = paintings_in_world(world);
        assert_eq!(paintings.len(), 1);
        assert_eq!(player.inventory.lock().get_selected_item().count(), 1);
    }

    #[test]
    fn paintings_are_not_placed_on_vertical_faces() {
        let (fixture, player, wall) = wall_fixture("painting_place_vertical");
        let world = &fixture.world;
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::new(&vanilla_items::PAINTING));

        let mut context = place_context(world, &player, wall, Direction::Up);
        assert_eq!(
            HangingEntityItem.use_on(&mut context),
            InteractionResult::Fail
        );
        assert!(paintings_in_world(world).is_empty());
        assert_eq!(player.inventory.lock().get_selected_item().count(), 1);
    }

    #[test]
    fn paintings_are_not_placed_without_a_wall() {
        let (fixture, player, wall) = wall_fixture("painting_place_no_wall");
        let world = &fixture.world;
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::new(&vanilla_items::PAINTING));

        assert!(world.set_block(
            wall,
            vanilla_blocks::AIR.default_state(),
            UpdateFlags::UPDATE_ALL
        ));

        let mut context = place_context(world, &player, wall, Direction::East);
        assert_eq!(
            HangingEntityItem.use_on(&mut context),
            InteractionResult::Consume
        );
        assert!(paintings_in_world(world).is_empty());
        assert_eq!(player.inventory.lock().get_selected_item().count(), 1);
    }

    #[test]
    fn painting_placement_copies_the_stack_custom_name() {
        let (fixture, player, wall) = wall_fixture("painting_place_name");
        let world = &fixture.world;

        let mut stack = ItemStack::new(&vanilla_items::PAINTING);
        stack.set(CUSTOM_NAME, TextComponent::plain("The Wall"));
        player.inventory.lock().set_selected_item(stack);

        let mut context = place_context(world, &player, wall, Direction::East);
        assert_eq!(
            HangingEntityItem.use_on(&mut context),
            InteractionResult::Success
        );

        let paintings = paintings_in_world(world);
        assert_eq!(paintings.len(), 1);
        assert_eq!(
            paintings[0].custom_name(),
            Some(TextComponent::plain("The Wall"))
        );
    }
}
