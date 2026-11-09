//! Cushion placement (`CushionItem`).
//!
//! All 16 coloured cushions share one vanilla behaviour class: the wool colour
//! travels in the stack's `minecraft:cushion/color` component and is copied to
//! the spawned `Cushion` entity. Placement follows the vanilla rules the wiki
//! documents for 26.3: the cushion needs a solid support directly underneath,
//! its own volume must be free of blocks and other cushions, and the placed
//! entity keeps the colour of the consumed item.

use std::sync::Arc;

use steel_macros::item_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::data_components::vanilla_components::CUSHION_COLOR;
use steel_registry::dye_color::DyeColor;
use steel_registry::item_stack::ItemStack;
use steel_registry::{sound_events, vanilla_entities, vanilla_game_events};
use steel_utils::{BlockPos, Direction, WorldAabb};

use crate::behavior::context::{InteractionResult, UseOnContext};
use crate::behavior::item::ItemBehavior;
use crate::entity::entities::CushionEntity;
use crate::entity::{Entity, SharedEntity, next_entity_id};
use crate::physics::collision::{WorldCollisionProvider, has_block_collision};
use crate::world::game_event::GameEventContext;
use crate::world::World;

/// Vertical size of a vanilla cushion, in blocks.
const CUSHION_HEIGHT: f64 = 0.25;

/// Placement behaviour shared by all 16 cushion colours.
#[item_behavior(class = "CushionItem")]
pub struct CushionItem;

impl CushionItem {
    /// The centre position the cushion entity would occupy above `support`.
    fn placement_position(support: BlockPos) -> glam::DVec3 {
        CushionEntity::support_top(support)
    }

    /// The bounding box of the cushion as it would be placed above `support`.
    fn placement_box(support: BlockPos) -> WorldAabb {
        let position = Self::placement_position(support);
        WorldAabb::entity_box(position.x, position.y, position.z, 0.5, CUSHION_HEIGHT)
    }

    /// Vanilla placement preconditions for the block the cushion rests on.
    fn can_place_on(
        context: &UseOnContext<'_>,
        stack: &ItemStack,
        side: Direction,
        target: BlockPos,
    ) -> bool {
        target.y() < context.world.max_build_height()
            && context.player.can_place_on(context.world, target, side, stack)
    }

    /// Whether the cushion volume is free of blocks and of other cushions.
    fn space_is_free(world: &Arc<World>, support: BlockPos) -> bool {
        let cushion_box = Self::placement_box(support);
        let collision = WorldCollisionProvider::new(world);
        if has_block_collision(&collision, cushion_box) {
            return false;
        }

        !world.has_entity_in_aabb_matching(&cushion_box, |entity| {
            entity.entity_type() == &vanilla_entities::CUSHION
        })
    }
}

impl ItemBehavior for CushionItem {
    fn use_on(&self, context: &mut UseOnContext) -> InteractionResult {
        let side = context.hit_result.direction;
        let (dx, dy, dz) = side.offset();
        let target = context.hit_result.block_pos.offset(dx, dy, dz);
        let support = BlockPos::new(target.x(), target.y() - 1, target.z());

        let stack = context.inv.with_item(|stack: &mut ItemStack| stack.clone());
        let color = stack.get(CUSHION_COLOR).copied().unwrap_or(DyeColor::White);

        if !Self::can_place_on(context, &stack, side, target) {
            return InteractionResult::Fail;
        }

        // The cushion needs a solid block directly underneath it.
        if !context.world.get_block_state(support).is_solid() {
            return InteractionResult::Consume;
        }

        // No blocks or other cushions may overlap the cushion's own volume.
        if !Self::space_is_free(context.world, support) {
            return InteractionResult::Consume;
        }

        let cushion = CushionEntity::new_on_support(
            &vanilla_entities::CUSHION,
            next_entity_id(),
            support,
            color,
            Arc::downgrade(context.world),
        );

        if let Some(custom_name) = stack.custom_name() {
            cushion.set_custom_name(Some(custom_name.into_owned()));
        }

        let entity: SharedEntity = Arc::new(cushion);
        if context.world.try_add_entity(entity).is_err() {
            return InteractionResult::Fail;
        }

        context.world.play_sound(
            &sound_events::ENTITY_CUSHION_PLACE,
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
    use steel_registry::data_components::vanilla_components::{CUSHION_COLOR, CUSTOM_NAME};
    use steel_registry::dye_color::DyeColor;
    use steel_registry::items::item::BlockHitResult;
    use steel_registry::item_stack::ItemStack;
    use steel_registry::{
        init_vanilla_registry, vanilla_blocks, vanilla_items,
    };
    use steel_utils::types::{InteractionHand, UpdateFlags};
    use steel_utils::{BlockPos, ChunkPos, Direction, WorldAabb};
    use steel_utils::Downcast as _;
    use text_components::TextComponent;

    use super::CushionItem;
    use crate::behavior::ItemBehavior as _;
    use crate::behavior::context::{InteractionResult, UseOnContext};
    use crate::behavior::init_behaviors;
    use crate::entity::entities::CushionEntity;
    use crate::entity::SharedEntity;
    use crate::entity::Entity as _;
    use crate::test_support::{
        TestPlayerBuilder, TestWorld, fresh_test_world, insert_ready_full_chunk,
    };
    use crate::world::World;

    /// Builds a world with a stone support at `(0, 64, 0)`.
    fn support_fixture(name: &str) -> (TestWorld, Arc<crate::player::Player>, BlockPos) {
        init_vanilla_registry();
        init_behaviors();

        let world_fixture = fresh_test_world(name);
        let world = Arc::clone(&world_fixture.world);
        let support = BlockPos::new(0, 64, 0);
        insert_ready_full_chunk(&world, ChunkPos::from_block_pos(support));
        assert!(world.set_block(
            support,
            vanilla_blocks::STONE.default_state(),
            UpdateFlags::UPDATE_ALL
        ));

        let player = TestPlayerBuilder::new(Arc::clone(&world), "CushionTester", 1).build();
        player.set_client_loaded(true);

        (world_fixture, player, support)
    }

    fn place_context<'a>(
        world: &'a Arc<World>,
        player: &'a Arc<crate::player::Player>,
        support: BlockPos,
    ) -> UseOnContext<'a> {
        UseOnContext::new(
            player,
            InteractionHand::MainHand,
            BlockHitResult {
                location: DVec3::new(0.5, 65.0, 0.5),
                direction: Direction::Up,
                block_pos: support,
                miss: false,
                inside: false,
                world_border_hit: false,
            },
            world,
            Arc::clone(&player.inventory),
        )
    }

    fn cushion_area() -> WorldAabb {
        WorldAabb::new(-2.0, 62.0, -2.0, 2.0, 68.0, 2.0)
    }

    fn cushions_in_world(world: &Arc<World>) -> Vec<SharedEntity> {
        world
            .get_entities_in_aabb(&cushion_area())
            .into_iter()
            .filter(|entity| entity.downcast_ref::<CushionEntity>().is_some())
            .collect()
    }

    fn as_cushion(entity: &SharedEntity) -> &CushionEntity {
        entity
            .downcast_ref::<CushionEntity>()
            .expect("entity should be a cushion")
    }

    fn held_stack(player: &Arc<crate::player::Player>) -> ItemStack {
        player.inventory.lock().get_selected_item().clone()
    }

    #[test]
    fn placing_a_white_cushion_spawns_it_on_the_support() {
        let (fixture, player, support) = support_fixture("cushion_place_ok");
        let world = &fixture.world;
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::with_count(&vanilla_items::WHITE_CUSHION, 3));

        let mut context = place_context(world, &player, support);
        assert_eq!(CushionItem.use_on(&mut context), InteractionResult::Success);

        let cushions = cushions_in_world(world);
        assert_eq!(cushions.len(), 1);
        assert_eq!(as_cushion(&cushions[0]).color(), DyeColor::White);
        assert_eq!(as_cushion(&cushions[0]).support_pos(), support);
        assert_eq!(held_stack(&player).count(), 2);
    }

    #[test]
    fn the_entity_takes_the_colour_of_the_placed_item() {
        let (fixture, player, support) = support_fixture("cushion_place_red");
        let world = &fixture.world;

        let mut stack = ItemStack::new(&vanilla_items::RED_CUSHION);
        stack.set(CUSHION_COLOR, DyeColor::Red);
        player.inventory.lock().set_selected_item(stack);

        let mut context = place_context(world, &player, support);
        assert_eq!(CushionItem.use_on(&mut context), InteractionResult::Success);

        let cushions = cushions_in_world(world);
        assert_eq!(cushions.len(), 1);
        assert_eq!(as_cushion(&cushions[0]).color(), DyeColor::Red);
        assert_eq!(
            as_cushion(&cushions[0]).as_item_stack().get(CUSHION_COLOR),
            Some(&DyeColor::Red)
        );
    }

    #[test]
    fn a_cushion_is_not_placed_without_solid_support() {
        let (fixture, player, support) = support_fixture("cushion_place_no_support");
        let world = &fixture.world;
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::with_count(&vanilla_items::WHITE_CUSHION, 2));

        // A slab does not count as vanilla `isSolid`.
        assert!(world.set_block(
            support,
            vanilla_blocks::OAK_SLAB.default_state(),
            UpdateFlags::UPDATE_ALL
        ));

        let mut context = place_context(world, &player, support);
        assert_eq!(
            CushionItem.use_on(&mut context),
            InteractionResult::Consume
        );

        assert!(cushions_in_world(world).is_empty());
        assert_eq!(held_stack(&player).count(), 2);
    }

    #[test]
    fn a_cushion_is_not_placed_inside_a_block() {
        let (fixture, player, support) = support_fixture("cushion_place_occupied");
        let world = &fixture.world;
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::new(&vanilla_items::WHITE_CUSHION));

        assert!(world.set_block(
            support.offset(0, 1, 0),
            vanilla_blocks::STONE.default_state(),
            UpdateFlags::UPDATE_ALL
        ));

        let mut context = place_context(world, &player, support);
        assert_eq!(
            CushionItem.use_on(&mut context),
            InteractionResult::Consume
        );
        assert!(cushions_in_world(world).is_empty());
        assert_eq!(held_stack(&player).count(), 1);
    }

    #[test]
    fn a_second_cushion_is_refused_on_an_existing_one() {
        let (fixture, player, support) = support_fixture("cushion_place_stack");
        let world = &fixture.world;
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::with_count(&vanilla_items::WHITE_CUSHION, 2));

        let mut first = place_context(world, &player, support);
        assert_eq!(CushionItem.use_on(&mut first), InteractionResult::Success);

        let mut second = place_context(world, &player, support);
        assert_eq!(
            CushionItem.use_on(&mut second),
            InteractionResult::Consume
        );

        assert_eq!(cushions_in_world(world).len(), 1);
        assert_eq!(held_stack(&player).count(), 1);
    }

    #[test]
    fn placement_copies_the_stack_custom_name() {
        let (fixture, player, support) = support_fixture("cushion_place_custom_name");
        let world = &fixture.world;

        let mut stack = ItemStack::new(&vanilla_items::BLUE_CUSHION);
        stack.set(CUSHION_COLOR, DyeColor::Blue);
        stack.set(CUSTOM_NAME, TextComponent::plain("Reading Nook"));
        player.inventory.lock().set_selected_item(stack);

        let mut context = place_context(world, &player, support);
        assert_eq!(CushionItem.use_on(&mut context), InteractionResult::Success);

        let cushions = cushions_in_world(world);
        assert_eq!(cushions.len(), 1);
        assert_eq!(
            as_cushion(&cushions[0]).custom_name(),
            Some(TextComponent::plain("Reading Nook"))
        );
    }
}
