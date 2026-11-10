//! Armor stand placement (`ArmorStandItem`).
//!
//! Mirrors vanilla `ArmorStandItem#useOn` for 26.3: clicking the bottom face of a
//! block fails, the clicked block's neighbour must be replaceable, the spawned
//! stand's yaw is snapped to one of the sixteen vanilla directions, the stand
//! copies the stack's custom name, vanilla emits `entity.armor_stand.place` and
//! `ENTITY_PLACE` at the stand's final position, the `ITEM_USED` statistic is
//! awarded and exactly one item is consumed outside creative mode.
//!
//! Not yet applied from the stack: the `minecraft:entity_data` component (Steel
//! has no generic "apply item stack config to a spawned entity" helper yet, see
//! `PARITY-26.3-RU.md`).

use std::sync::Arc;

use glam::DVec3;
use steel_macros::item_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::item_stack::ItemStack;
use steel_registry::stat::vanilla_stat_types;
use steel_registry::{sound_events, vanilla_entities, vanilla_game_events};
use steel_utils::Direction;

use crate::behavior::context::{InteractionResult, UseOnContext};
use crate::behavior::item::ItemBehavior;
use crate::entity::entities::ArmorStandEntity;
use crate::entity::{Entity, LivingEntity, SharedEntity, next_entity_id};
use crate::world::game_event::GameEventContext;

/// Number of yaw steps vanilla snaps a placed armor stand to.
const YAW_STEPS: f32 = 16.0;
/// Degrees per snapped armor stand yaw step.
const YAW_STEP_DEGREES: f32 = 360.0 / YAW_STEPS;

/// Vanilla `entity.armor_stand.place` volume.
const PLACE_SOUND_VOLUME: f32 = 0.75;
/// Vanilla `entity.armor_stand.place` pitch.
const PLACE_SOUND_PITCH: f32 = 0.8;

/// Placement behavior of the armor stand item.
#[item_behavior(class = "ArmorStandItem")]
pub struct ArmorStandItem;

impl ArmorStandItem {
    /// Vanilla `ArmorStandItem#snapTo`: the stand faces the player, snapped to
    /// the nearest of sixteen directions.
    #[must_use]
    fn snapped_yaw(player_yaw: f32) -> f32 {
        let step = ((player_yaw + 180.0) * YAW_STEPS / 360.0 + 0.5).floor() as i32 & 0xF;
        step as f32 * YAW_STEP_DEGREES
    }
}

impl ItemBehavior for ArmorStandItem {
    fn use_on(&self, context: &mut UseOnContext) -> InteractionResult {
        let side = context.hit_result.direction;
        // Vanilla refuses stands placed against the bottom face of a block.
        if side == Direction::Down {
            return InteractionResult::Fail;
        }

        let (dx, dy, dz) = side.offset();
        let target = context.hit_result.block_pos.offset(dx, dy, dz);
        let target_state = context.world.get_block_state(target);
        if !target_state.is_replaceable() || target.y() >= context.world.max_build_height() {
            return InteractionResult::Fail;
        }

        let stack = context.inv.with_item(|stack: &mut ItemStack| stack.clone());
        let player_yaw = context.player.rotation().0;
        let yaw = Self::snapped_yaw(player_yaw);

        let stand = ArmorStandEntity::new(
            &vanilla_entities::ARMOR_STAND,
            next_entity_id(),
            DVec3::new(
                f64::from(target.x()) + 0.5,
                f64::from(target.y()),
                f64::from(target.z()) + 0.5,
            ),
            Arc::downgrade(context.world),
        );
        stand.set_rotation((yaw, 0.0));
        stand.set_y_head_rot(yaw);
        stand.set_y_body_rot(yaw);
        if let Some(custom_name) = stack.custom_name() {
            stand.set_custom_name(Some(custom_name.into_owned()));
        }

        let entity: SharedEntity = Arc::new(stand);
        if context.world.try_add_entity(entity).is_err() {
            return InteractionResult::Fail;
        }

        context.world.play_sound(
            &sound_events::ENTITY_ARMOR_STAND_PLACE,
            SoundSource::Blocks,
            target,
            PLACE_SOUND_VOLUME,
            PLACE_SOUND_PITCH,
            None,
        );
        context.world.game_event(
            &vanilla_game_events::ENTITY_PLACE,
            target,
            &GameEventContext::new(Some(&**context.player as &dyn Entity), None),
        );

        let item = context.inv.with_item(|stack| stack.item);
        context.player.award_stat(&vanilla_stat_types::ITEM_USED, item);
        let has_infinite_materials = context.player.has_infinite_materials();
        context
            .inv
            .with_item(|stack| stack.consume_one(has_infinite_materials));

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
    use steel_utils::Downcast as _;
    use steel_utils::types::{InteractionHand, UpdateFlags};
    use steel_utils::{BlockPos, ChunkPos, Direction, WorldAabb};
    use text_components::TextComponent;

    use super::ArmorStandItem;
    use crate::behavior::ItemBehavior as _;
    use crate::behavior::context::{InteractionResult, UseOnContext};
    use crate::behavior::init_behaviors;
    use crate::entity::entities::ArmorStandEntity;
    use crate::entity::{Entity as _, SharedEntity};
    use crate::player::Player;
    use crate::test_support::{
        TestPlayerBuilder, TestWorld, fresh_test_world, insert_ready_full_chunk,
    };
    use crate::world::World;

    fn stand_area() -> WorldAabb {
        WorldAabb::new(-4.0, 60.0, -4.0, 4.0, 72.0, 4.0)
    }

    /// Builds a world with a stone block at `(0, 64, 0)` to click on.
    fn floor_fixture(name: &str) -> (TestWorld, Arc<Player>, BlockPos) {
        init_vanilla_registry();
        init_behaviors();

        let world_fixture = fresh_test_world(name);
        let world = Arc::clone(&world_fixture.world);
        let floor = BlockPos::new(0, 64, 0);
        insert_ready_full_chunk(&world, ChunkPos::from_block_pos(floor));
        assert!(world.set_block(
            floor,
            vanilla_blocks::STONE.default_state(),
            UpdateFlags::UPDATE_ALL
        ));

        let player = TestPlayerBuilder::new(Arc::clone(&world), "StandPlacer", 1).build();
        player.set_client_loaded(true);

        (world_fixture, player, floor)
    }

    fn place_context<'a>(
        world: &'a Arc<World>,
        player: &'a Arc<Player>,
        floor: BlockPos,
        side: Direction,
    ) -> UseOnContext<'a> {
        UseOnContext::new(
            player,
            InteractionHand::MainHand,
            BlockHitResult {
                location: DVec3::new(0.5, 65.0, 0.5),
                direction: side,
                block_pos: floor,
                miss: false,
                inside: false,
                world_border_hit: false,
            },
            world,
            Arc::clone(&player.inventory),
        )
    }

    fn stands_in_world(world: &Arc<World>) -> Vec<SharedEntity> {
        world
            .get_entities_in_aabb(&stand_area())
            .into_iter()
            .filter(|entity| entity.downcast_ref::<ArmorStandEntity>().is_some())
            .collect()
    }

    fn as_stand(entity: &SharedEntity) -> &ArmorStandEntity {
        entity
            .downcast_ref::<ArmorStandEntity>()
            .expect("entity should be an armor stand")
    }

    #[test]
    fn placing_an_armor_stand_consumes_one_item() {
        let (fixture, player, floor) = floor_fixture("armor_stand_place_ok");
        let world = &fixture.world;
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::with_count(&vanilla_items::ARMOR_STAND, 2));

        let mut context = place_context(world, &player, floor, Direction::Up);
        assert_eq!(
            ArmorStandItem.use_on(&mut context),
            InteractionResult::Success
        );

        let stands = stands_in_world(world);
        assert_eq!(stands.len(), 1);
        let position = stands[0].position();
        assert!((position.x - 0.5).abs() < f64::EPSILON);
        assert!((position.y - 65.0).abs() < f64::EPSILON);
        assert!((position.z - 0.5).abs() < f64::EPSILON);
        assert_eq!(player.inventory.lock().get_selected_item().count(), 1);
    }

    #[test]
    fn placement_snaps_the_stand_to_the_player_facing() {
        let (fixture, player, floor) = floor_fixture("armor_stand_place_yaw");
        let world = &fixture.world;
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::new(&vanilla_items::ARMOR_STAND));
        player.set_rotation((10.0, 0.0));

        let mut context = place_context(world, &player, floor, Direction::Up);
        assert_eq!(
            ArmorStandItem.use_on(&mut context),
            InteractionResult::Success
        );

        // (10 + 180) / 22.5 = 8.44 -> step 8 -> 180 degrees.
        let stands = stands_in_world(world);
        let stand = stands.first().expect("a stand should have been placed");
        assert!((as_stand(stand).rotation().0 - 180.0).abs() < f32::EPSILON);
    }

    #[test]
    fn placement_is_refused_under_a_block() {
        let (fixture, player, floor) = floor_fixture("armor_stand_place_down");
        let world = &fixture.world;
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::new(&vanilla_items::ARMOR_STAND));

        let mut context = place_context(world, &player, floor, Direction::Down);
        assert_eq!(ArmorStandItem.use_on(&mut context), InteractionResult::Fail);
        assert!(stands_in_world(world).is_empty());
        assert_eq!(player.inventory.lock().get_selected_item().count(), 1);
    }

    #[test]
    fn placement_is_refused_into_a_solid_block() {
        let (fixture, player, floor) = floor_fixture("armor_stand_place_occupied");
        let world = &fixture.world;
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::new(&vanilla_items::ARMOR_STAND));
        assert!(world.set_block(
            floor.above(),
            vanilla_blocks::STONE.default_state(),
            UpdateFlags::UPDATE_ALL
        ));

        let mut context = place_context(world, &player, floor, Direction::Up);
        assert_eq!(ArmorStandItem.use_on(&mut context), InteractionResult::Fail);
        assert!(stands_in_world(world).is_empty());
    }

    #[test]
    fn placement_copies_the_stack_custom_name() {
        let (fixture, player, floor) = floor_fixture("armor_stand_place_name");
        let world = &fixture.world;

        let mut stack = ItemStack::new(&vanilla_items::ARMOR_STAND);
        stack.set(CUSTOM_NAME, TextComponent::plain("Sir Stands"));
        player.inventory.lock().set_selected_item(stack);

        let mut context = place_context(world, &player, floor, Direction::Up);
        assert_eq!(
            ArmorStandItem.use_on(&mut context),
            InteractionResult::Success
        );

        let stands = stands_in_world(world);
        let stand = stands.first().expect("a stand should have been placed");
        assert_eq!(
            stand.custom_name(),
            Some(TextComponent::plain("Sir Stands"))
        );
    }
}
