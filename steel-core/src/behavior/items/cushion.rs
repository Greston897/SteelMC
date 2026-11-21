//! Cushion placement (`CushionItem`).
//!
//! All 16 coloured cushions share one vanilla behaviour class: the wool colour
//! travels in the stack's `minecraft:cushion/color` component and is copied to
//! the spawned `Cushion` entity. Placement uses the clicked height, support
//! shape and vanilla's special collision-shape raycast for tagged blocks.

use std::sync::Arc;

use steel_macros::item_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::data_components::vanilla_components::CUSHION_COLOR;
use steel_registry::dye_color::DyeColor;
use steel_registry::item_stack::ItemStack;
use steel_registry::items::item::BlockHitResult;
use steel_registry::vanilla_block_tags::BlockTag;
use steel_registry::{sound_events, vanilla_entities, vanilla_game_events};
use steel_utils::{BlockPos, Direction, WorldAabb};

use crate::behavior::context::{BlockPlaceContext, InteractionResult, PlacementSource, UseOnContext};
use crate::behavior::item::ItemBehavior;
use crate::entity::entities::CushionEntity;
use crate::entity::{Entity, SharedEntity, next_entity_id};
use crate::world::World;
use crate::world::game_event::GameEventContext;

/// Placement behaviour shared by all 16 cushion colours.
#[item_behavior(class = "CushionItem")]
pub struct CushionItem;

impl CushionItem {
    /// Recast tagged container outlines against their collision shape, as vanilla does.
    fn effective_hit(context: &UseOnContext<'_>) -> BlockHitResult {
        let hit = &context.hit_result;
        let state = context.world.get_block_state(hit.block_pos);
        if !state
            .get_block()
            .has_tag(&BlockTag::CUSHION_USES_COLLISION_SHAPE)
        {
            return hit.clone();
        }

        let player_pos = context.player.position();
        let eye = glam::DVec3::new(player_pos.x, context.player.get_eye_y(), player_pos.z);
        let ray = hit.location - eye;
        let length = ray.length();
        let direction = if length < f64::from(1.0e-5_f32) {
            glam::DVec3::ZERO
        } else {
            ray / length
        };
        let to = hit.location + direction * 0.001;
        World::clip_shape(
            hit.block_pos,
            eye,
            to,
            state.get_collision_shape_at(hit.block_pos),
        )
        .map_or_else(
            || hit.clone(),
            |collision_hit| BlockHitResult {
                location: collision_hit.location,
                direction: collision_hit.direction,
                block_pos: collision_hit.block_pos,
                miss: collision_hit.miss,
                inside: collision_hit.inside,
                world_border_hit: collision_hit.world_border_hit,
            },
        )
    }

    /// Vanilla placement preconditions for the block the cushion rests on.
    fn can_place_on(
        context: &UseOnContext<'_>,
        stack: &ItemStack,
        side: Direction,
        target: BlockPos,
    ) -> bool {
        target.y() < context.world.max_build_height()
            && context
                .player
                .can_place_on(context.world, target, side, stack)
    }

    /// Other cushions cannot overlap the placed cushion's bounding box.
    fn space_is_free(world: &Arc<World>, cushion_box: WorldAabb) -> bool {
        !world.has_entity_in_aabb_matching(&cushion_box, |entity| {
            entity.entity_type() == &vanilla_entities::CUSHION
        })
    }
}

impl ItemBehavior for CushionItem {
    fn use_on(&self, context: &mut UseOnContext) -> InteractionResult {
        let hit = Self::effective_hit(context);
        let side = hit.direction;
        if side != Direction::Up {
            return InteractionResult::Fail;
        }
        let target = BlockPlaceContext::new(
            context.world,
            PlacementSource::player_hand(context.player, &context.inv),
            &hit,
        )
        .place_pos();
        let position = glam::DVec3::new(
            f64::from(target.x()) + 0.5,
            hit.location.y,
            f64::from(target.z()) + 0.5,
        );
        let dimensions = vanilla_entities::CUSHION.dimensions;
        let bounding_box = WorldAabb::entity_box(
            position.x,
            position.y,
            position.z,
            f64::from(dimensions.width) / 2.0,
            f64::from(dimensions.height),
        );

        let stack = context.inv.with_item(|stack: &mut ItemStack| stack.clone());
        let color = stack.get(CUSHION_COLOR).copied().unwrap_or(DyeColor::White);

        if !Self::can_place_on(context, &stack, side, target) {
            return InteractionResult::Fail;
        }

        if !CushionEntity::can_be_placed_at(context.world, bounding_box)
            || !Self::space_is_free(context.world, bounding_box)
        {
            return InteractionResult::Fail;
        }

        let cushion = CushionEntity::new(
            &vanilla_entities::CUSHION,
            next_entity_id(),
            position,
            Arc::downgrade(context.world),
        );
        cushion.set_color(color);
        cushion.set_rotation((
            Direction::from_yaw(context.player.rotation().0).to_yaw(),
            0.0,
        ));

        if let Some(custom_name) = stack.custom_name() {
            cushion.set_custom_name(Some(custom_name.into_owned()));
        }

        let entity: SharedEntity = Arc::new(cushion);
        if context.world.try_add_entity(entity).is_err() {
            return InteractionResult::Fail;
        }

        context.world.play_sound(
            &sound_events::ENTITY_CUSHION_PLACE,
            SoundSource::Blocks,
            target,
            0.75,
            0.8,
            None,
        );
        context.world.game_event(
            &vanilla_game_events::ENTITY_PLACE,
            target,
            &GameEventContext::new(Some(&**context.player as &dyn Entity), None),
        );
        let infinite_materials = context.player.has_infinite_materials();
        context
            .inv
            .with_item(|stack| stack.consume_one(infinite_materials));

        InteractionResult::Success
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use glam::DVec3;
    use steel_registry::blocks::block_state_ext::BlockStateExt as _;
    use steel_registry::blocks::properties::{BlockStateProperties, SlabType};
    use steel_registry::data_components::vanilla_components::{CUSHION_COLOR, CUSTOM_NAME};
    use steel_registry::dye_color::DyeColor;
    use steel_registry::item_stack::ItemStack;
    use steel_registry::items::item::BlockHitResult;
    use steel_registry::{init_vanilla_registry, vanilla_blocks, vanilla_items};
    use steel_utils::Downcast as _;
    use steel_utils::types::{GameType, InteractionHand, UpdateFlags};
    use steel_utils::{BlockPos, ChunkPos, Direction, WorldAabb};
    use text_components::TextComponent;

    use super::CushionItem;
    use crate::behavior::ItemBehavior as _;
    use crate::behavior::context::{InteractionResult, UseOnContext};
    use crate::behavior::init_behaviors;
    use crate::entity::Entity as _;
    use crate::entity::SharedEntity;
    use crate::entity::entities::CushionEntity;
    use crate::player::Player;
    use crate::test_support::{
        TestPlayerBuilder, TestWorld, fresh_test_world, insert_ready_full_chunk,
    };
    use crate::world::World;

    /// Builds a world with a stone support at `(0, 64, 0)`.
    fn support_fixture(name: &str) -> (TestWorld, Arc<Player>, BlockPos) {
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
        player: &'a Arc<Player>,
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

    fn held_stack(player: &Arc<Player>) -> ItemStack {
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

        assert!(world.set_block(
            support,
            vanilla_blocks::AIR.default_state(),
            UpdateFlags::UPDATE_ALL
        ));

        let mut context = place_context(world, &player, support);
        assert_eq!(CushionItem.use_on(&mut context), InteractionResult::Fail);

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
        assert_eq!(CushionItem.use_on(&mut context), InteractionResult::Fail);
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
        assert_eq!(CushionItem.use_on(&mut second), InteractionResult::Fail);

        assert_eq!(cushions_in_world(world).len(), 1);
        assert_eq!(held_stack(&player).count(), 1);
    }

    #[test]
    fn composter_hit_is_recast_against_its_collision_shape() {
        let (fixture, player, support) = support_fixture("cushion_composter_raycast");
        let world = &fixture.world;
        let composter = vanilla_blocks::COMPOSTER
            .default_state()
            .set_value(&BlockStateProperties::LEVEL_COMPOSTER, 7);
        assert!(world.set_block(support, composter, UpdateFlags::UPDATE_ALL));
        player.base().set_position_local(DVec3::new(0.5, 66.0, 0.5));
        let mut context = place_context(world, &player, support);
        // The synthetic endpoint extends through the open centre. The collision
        // shape is always the empty composter's 2/16-high floor, not the
        // visible compost surface at level seven.
        context.hit_result.location.y = 64.0;
        let effective = CushionItem::effective_hit(&context);
        assert_eq!(effective.direction, Direction::Up);
        assert!((effective.location.y - 64.125).abs() < 1.0e-6);
    }

    #[test]
    fn bottom_slab_uses_the_actual_hit_height_and_remains_supported() {
        let (fixture, player, support) = support_fixture("cushion_place_bottom_slab");
        let world = &fixture.world;
        let slab = vanilla_blocks::OAK_SLAB
            .default_state()
            .set_value(&BlockStateProperties::SLAB_TYPE, SlabType::Bottom);
        assert!(world.set_block(support, slab, UpdateFlags::UPDATE_ALL));
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::new(&vanilla_items::WHITE_CUSHION));

        let mut context = place_context(world, &player, support);
        context.hit_result.location.y = 64.5;
        assert_eq!(CushionItem.use_on(&mut context), InteractionResult::Success);
        let cushions = cushions_in_world(world);
        assert_eq!(cushions.len(), 1);
        let cushion = as_cushion(&cushions[0]);
        assert_eq!(cushion.position().y, 64.5);
        assert!(CushionEntity::would_survive_at(
            world,
            cushion.bounding_box()
        ));
    }

    #[test]
    fn a_buried_anchor_is_refused_even_if_the_cover_is_not_suffocating() {
        let (fixture, player, support) = support_fixture("cushion_place_buried");
        let world = &fixture.world;
        assert!(world.set_block(
            support.offset(0, 1, 0),
            vanilla_blocks::GLASS.default_state(),
            UpdateFlags::UPDATE_ALL
        ));
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::new(&vanilla_items::WHITE_CUSHION));
        let mut context = place_context(world, &player, support);
        assert_eq!(CushionItem.use_on(&mut context), InteractionResult::Fail);
        assert!(cushions_in_world(world).is_empty());
    }

    #[test]
    fn placement_rejects_non_upward_faces() {
        let (fixture, player, support) = support_fixture("cushion_place_side");
        let world = &fixture.world;
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::with_count(&vanilla_items::WHITE_CUSHION, 2));

        for direction in [
            Direction::Down,
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ] {
            let mut context = place_context(world, &player, support);
            context.hit_result.direction = direction;
            assert_eq!(CushionItem.use_on(&mut context), InteractionResult::Fail);
        }
        assert!(cushions_in_world(world).is_empty());
        assert_eq!(held_stack(&player).count(), 2);
    }

    #[test]
    fn creative_placement_does_not_consume_the_item() {
        let (fixture, player, support) = support_fixture("cushion_place_creative");
        let world = &fixture.world;
        player.set_game_mode(GameType::Creative);
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::with_count(&vanilla_items::WHITE_CUSHION, 2));

        let mut context = place_context(world, &player, support);
        assert_eq!(CushionItem.use_on(&mut context), InteractionResult::Success);
        assert_eq!(cushions_in_world(world).len(), 1);
        assert_eq!(held_stack(&player).count(), 2);
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
