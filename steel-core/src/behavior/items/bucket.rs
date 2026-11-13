//! Bucket item behavior implementation.
use crate::behavior::context::InteractionResult;
use crate::behavior::item_utils::create_filled_result;
use crate::behavior::{
    BLOCK_BEHAVIORS, BlockStateBehaviorExt, FLUID_BEHAVIORS, ItemBehavior, UseItemContext,
};
use crate::entity::Entity;
use crate::fluid::{fluid_state_to_block, is_lava_fluid, is_water_fluid};
use crate::world::RaytraceAction;
use steel_macros::item_behavior;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::Direction;
use steel_registry::fluid::{FluidRef, FluidState};
use steel_registry::item_stack::ItemStack;
use steel_registry::level_events;
use steel_registry::sound_events;
use steel_registry::stat::vanilla_stat_types;
use steel_registry::vanilla_blocks;
use steel_registry::vanilla_fluids;
use steel_registry::vanilla_game_events;
use steel_registry::vanilla_items;
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId};

use crate::world::game_event::GameEventContext;

/// A bucket holding `content`, or the empty bucket when `content` is the empty fluid.
#[item_behavior]
pub struct BucketItem {
    #[json_arg(vanilla_fluids, ref, json = "content")]
    content: FluidRef,
}

impl BucketItem {
    /// Creates a bucket behavior for `content`.
    #[must_use]
    pub const fn new(content: FluidRef) -> Self {
        Self { content }
    }
}

impl ItemBehavior for BucketItem {
    fn use_item(&self, context: &mut UseItemContext) -> InteractionResult {
        if self.content.is_empty {
            return use_empty_bucket(context);
        }
        use_filled_bucket(self.content, context)
    }
}

fn filled_bucket_success_stack(context: &UseItemContext) -> ItemStack {
    if context.player.has_infinite_materials() {
        context
            .inv
            .with_item(|item| item.copy_with_count(item.count()))
    } else {
        ItemStack::new(&*vanilla_items::BUCKET)
    }
}

fn use_empty_bucket(context: &mut UseItemContext) -> InteractionResult {
    let (start, end) = context.player.get_ray_endpoints();

    // Vanilla `BucketItem.tryBucket`: an empty bucket first tries to scoop up a
    // `Bucketable` mob along the player's view ray.
    if let Some(result) = try_catch_bucketable_entity(context, start, end) {
        return result;
    }

    // Raytrace: stop on source fluids
    let (hit_block, hit_direction) = context.world.raytrace(start, end, |pos, world| {
        let state = world.get_block_state(pos);
        let block = state.get_block();

        if block == &vanilla_blocks::AIR {
            return RaytraceAction::Pass;
        }

        let fluid_state = state.get_fluid_state();
        if fluid_state.is_source() {
            return RaytraceAction::ImmediateHit;
        }
        // Flowing fluid is transparent to empty buckets.
        if !fluid_state.is_empty() {
            return RaytraceAction::Pass;
        }

        RaytraceAction::CheckShape
    });

    // A miss passes, so other handlers can try.
    let (Some(hit_pos), Some(direction)) = (hit_block, hit_direction) else {
        return InteractionResult::Pass;
    };

    if !may_use_bucket_at(context, hit_pos, direction) {
        return InteractionResult::Fail;
    }

    let hit_state = context.world.get_block_state(hit_pos);
    let block_behavior = BLOCK_BEHAVIORS.get_behavior(hit_state.get_block());

    // `BlockBehavior::pickup_block`'s default already drains waterloggable
    // blocks, so blocks with custom pickup logic only override it.
    let picked =
        block_behavior.pickup_block(context.world, hit_pos, hit_state, Some(context.player));
    // Fail, not pass, so the client knows no item changed.
    let Some(result) = picked else {
        return InteractionResult::Fail;
    };

    context.inv.with_item(|item| {
        context
            .player
            .award_stat(&vanilla_stat_types::ITEM_USED, item.item());
    });
    if let Some(sound) = result.sound {
        context.player.play_sound(sound, 1.0, 1.0);
    }

    create_filled_result(context, result.filled_bucket, true);
    context.world.game_event(
        &vanilla_game_events::FLUID_PICKUP,
        hit_pos,
        &GameEventContext::new(Some(context.player.as_ref()), None),
    );

    InteractionResult::Success
}

fn use_filled_bucket(fluid: FluidRef, context: &mut UseItemContext) -> InteractionResult {
    let (start, end) = context.player.get_ray_endpoints();
    let (ray_block, ray_dir) = context.world.raytrace(start, end, |pos, world| {
        let state = world.get_block_state(pos);
        let block = state.get_block();
        // Filled buckets use ClipContext.Fluid.NONE: ignore fluid shapes, but
        // still test the block shape of waterlogged/container blocks.
        if block == &vanilla_blocks::AIR {
            return RaytraceAction::Pass;
        }
        RaytraceAction::CheckShape
    });

    let (Some(clicked_pos), Some(direction)) = (ray_block, ray_dir) else {
        return InteractionResult::Pass;
    };

    if !context.world.is_in_valid_bounds(clicked_pos) {
        return InteractionResult::Fail;
    }

    if !may_use_bucket_at(context, clicked_pos, direction) {
        return InteractionResult::Fail;
    }

    let clicked_state = context.world.get_block_state(clicked_pos);
    let is_sneaking = context.player.is_crouching();
    // Only water itself enters liquid containers, not water-tagged fluids.
    let is_water_bucket = fluid == &vanilla_fluids::WATER;

    // `check_sneak` is false for the secondary attempt, which ignores sneaking.
    let try_place_fluid = |pos: BlockPos, check_sneak: bool| -> bool {
        if !context.world.is_in_valid_bounds(pos) {
            return false;
        }

        let state = context.world.get_block_state(pos);
        let fluid_state = state.get_fluid_state();

        // When sneaking, only air accepts the fluid here; other blocks redirect to the
        // neighbor through the secondary attempt.
        if check_sneak && is_sneaking && !state.get_block().config.is_air {
            return false;
        }

        let behavior = BLOCK_BEHAVIORS.get_behavior(state.get_block());
        let is_liquid_container = state.is_liquid_container();
        let can_place_liquid = is_liquid_container
            && behavior.can_place_liquid_with_player(state, fluid, Some(context.player));
        let can_replace = state.can_be_replaced_by_fluid(fluid);

        if !can_replace && !can_place_liquid {
            return false;
        }

        // Water-tagged fluids fizz out where water evaporates (e.g. the Nether).
        // Vanilla reads `gameplay/water_evaporates` from the position's
        // environment attributes; the dimension type supplies the same value
        // until a datapack overrides it per biome.
        let water_evaporates = context
            .world
            .dimension_type
            .gameplay_attribute_bool("minecraft:gameplay/water_evaporates")
            .unwrap_or(false);
        if is_water_fluid(fluid) && water_evaporates {
            context
                .world
                .level_event(level_events::PARTICLES_WATER_EVAPORATING, pos, 0, None);
            return true;
        }

        if is_water_bucket && is_liquid_container {
            let source_water = FluidState::source(&vanilla_fluids::WATER);
            behavior.place_liquid(context.world, pos, state, source_water);
            play_empty_sound_and_event(context, pos, fluid);
            return true;
        }

        // Containers that accept a fluid other than water get overwritten here too.
        // An existing source of the same fluid just consumes the bucket.
        let is_same_fluid = FLUID_BEHAVIORS
            .get_behavior(fluid)
            .is_same(fluid_state.fluid_id);

        if is_same_fluid && fluid_state.is_source() {
            play_empty_sound_and_event(context, pos, fluid);
            return true;
        }

        // Destroy non-liquid replaceable blocks first so they drop their items.
        if can_replace && !state.get_block().config.liquid && !state.get_block().config.is_air {
            context.player.get_world().destroy_block(pos, true);
        }

        let fluid_state_to_place = fluid_state_to_block(FluidState::source(fluid));
        if context
            .world
            .set_block(pos, fluid_state_to_place, UpdateFlags::UPDATE_ALL_IMMEDIATE)
        {
            let tick_delay = FLUID_BEHAVIORS
                .get_behavior(fluid)
                .tick_delay(context.world);
            context
                .world
                .schedule_fluid_tick_default(pos, fluid, tick_delay);

            play_empty_sound_and_event(context, pos, fluid);

            return true;
        }
        false
    };

    // If the primary attempt fails, the secondary one retries at the offset position,
    // whatever the bucket.
    let primary_pos =
        filled_bucket_primary_pos(clicked_state, clicked_pos, direction, is_water_bucket);

    if !try_place_fluid(primary_pos, true)
        && !try_place_fluid(direction.relative(clicked_pos), false)
    {
        return InteractionResult::Fail;
    }

    let result_stack = filled_bucket_success_stack(context);
    create_filled_result(context, result_stack, true);
    InteractionResult::Success
}

/// Whether the player may use the held bucket against the `direction` face of `pos`.
fn may_use_bucket_at(context: &UseItemContext, pos: BlockPos, direction: Direction) -> bool {
    let item = context.inv.with_item(|item| item.clone());
    context
        .player
        .may_use_item_at(direction.relative(pos), direction, &item)
}

fn play_empty_sound_and_event(context: &UseItemContext, pos: BlockPos, fluid: FluidRef) {
    let sound_event = if is_lava_fluid(fluid) {
        &sound_events::ITEM_BUCKET_EMPTY_LAVA
    } else {
        &sound_events::ITEM_BUCKET_EMPTY
    };
    context.world.play_block_sound(
        sound_event,
        pos,
        1.0,
        1.0,
        // Excludes the placing player.
        Some(context.player.id()),
    );
    context.world.game_event(
        &vanilla_game_events::FLUID_PLACE,
        pos,
        &GameEventContext::new(Some(context.player.as_ref()), None),
    );
}

fn filled_bucket_primary_pos(
    clicked_state: BlockStateId,
    clicked_pos: BlockPos,
    direction: Direction,
    is_water_bucket: bool,
) -> BlockPos {
    if is_water_bucket && clicked_state.is_liquid_container() {
        clicked_pos
    } else {
        direction.relative(clicked_pos)
    }
}

/// Vanilla `ProjectileUtil.getEntityHitResult` pass from `BucketItem.use`:
/// the nearest alive, not-yet-bucketed `Bucketable` mob within reach.
fn try_catch_bucketable_entity(
    context: &UseItemContext,
    start: glam::DVec3,
    end: glam::DVec3,
) -> Option<InteractionResult> {
    use crate::entity::bucketable::{is_bucketable, try_bucket_entity};
    use crate::entity::projectile::get_entity_hit_result;

    let look = end - start;
    let search_box = context
        .player
        .bounding_box()
        .expand_towards(look)
        .inflate(1.0);

    let hit = get_entity_hit_result(context.world, start, end, search_box, 0.0, |entity| {
        entity.is_alive() && is_bucketable(entity)
    })?;

    try_bucket_entity(context.player, context.hand, hit.entity.as_ref())
}

#[cfg(test)]
mod tests {
    use crate::{
        behavior::init_behaviors,
        player::game_mode::use_item,
        test_support::{TestPlayerBuilder, fresh_test_world, insert_ready_full_chunk},
    };
    use glam::DVec3;
    use std::sync::Arc;
    use steel_registry::{
        RegistryHolderSet,
        data_component_predicate::DataComponentMatchers,
        data_components::{
            AdventureModePredicate, BlockPredicate, vanilla_components::CAN_PLACE_ON,
        },
        init_vanilla_registry,
        item_stack::ItemStack,
        vanilla_blocks, vanilla_items,
    };
    use steel_utils::{
        BlockPos, ChunkPos,
        types::{GameType, InteractionHand, UpdateFlags},
    };

    use super::*;

    #[test]
    fn filled_water_bucket_targets_non_waterlogged_liquid_container_in_place() {
        init_vanilla_registry();
        init_behaviors();

        let kelp = vanilla_blocks::KELP.default_state();

        assert_eq!(
            filled_bucket_primary_pos(kelp, BlockPos::ZERO, Direction::North, true),
            BlockPos::ZERO
        );
        assert_eq!(
            filled_bucket_primary_pos(kelp, BlockPos::ZERO, Direction::North, false),
            Direction::North.relative(BlockPos::ZERO)
        );
    }

    #[test]
    fn adventure_buckets_require_can_place_on_and_fail_when_denied() {
        init_vanilla_registry();
        init_behaviors();
        let world_fixture = fresh_test_world("adventure_bucket_restriction");
        let world = &world_fixture.world;
        let pos = BlockPos::new(0, 65, 2);
        insert_ready_full_chunk(world, ChunkPos::from_block_pos(pos));
        assert!(world.set_block(
            pos,
            vanilla_blocks::WATER.default_state(),
            UpdateFlags::UPDATE_ALL,
        ));

        let player = TestPlayerBuilder::new(Arc::clone(world), "TestPlayer", 1).build();
        player.base().set_position_local(DVec3::new(0.5, 64.0, 0.5));
        player.set_rotation((0.0, 0.0));
        player.restore_game_modes(GameType::Adventure, None);
        player
            .abilities
            .lock()
            .update_for_game_mode(GameType::Adventure);

        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::new(&*vanilla_items::BUCKET));
        assert_eq!(
            use_item(&player, world, InteractionHand::MainHand),
            InteractionResult::Fail
        );
        assert_eq!(
            world.get_block_state(pos),
            vanilla_blocks::WATER.default_state()
        );

        let predicate = BlockPredicate::new(
            Some(RegistryHolderSet::Direct(vec![&vanilla_blocks::WATER])),
            None,
            None,
            DataComponentMatchers::ANY,
        );
        let mut allowed_bucket = ItemStack::new(&*vanilla_items::BUCKET);
        allowed_bucket.set(
            CAN_PLACE_ON,
            AdventureModePredicate::new(vec![predicate]).expect("one block predicate is valid"),
        );
        player.inventory.lock().set_selected_item(allowed_bucket);
        assert!(use_item(&player, world, InteractionHand::MainHand).consumes_action());
        assert_eq!(
            world.get_block_state(pos),
            vanilla_blocks::AIR.default_state()
        );

        assert!(world.set_block(
            pos,
            vanilla_blocks::STONE.default_state(),
            UpdateFlags::UPDATE_ALL,
        ));
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::new(&*vanilla_items::WATER_BUCKET));
        assert_eq!(
            use_item(&player, world, InteractionHand::MainHand),
            InteractionResult::Fail
        );
        assert_eq!(
            world.get_block_state(pos),
            vanilla_blocks::STONE.default_state()
        );
    }
}
