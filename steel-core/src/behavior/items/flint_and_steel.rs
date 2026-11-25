//! Flint and steel and fire charge item behavior.
//!
//! Vanilla `FireChargeItem` ignites blocks on use; only dispensers launch
//! fireballs. Using the item in the air has no special effect.

use crate::behavior::blocks::FireBlock;
use crate::behavior::context::{InteractionResult, UseOnContext};
use crate::behavior::item::ItemBehavior;
use crate::entity::Entity as _;
use steel_macros::item_behavior;
use steel_registry::item_stack::ItemStack;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_block_tags::BlockTag;
use steel_registry::{
    blocks::{block_state_ext::BlockStateExt, properties::BlockStateProperties},
    sound_events, vanilla_game_events,
};
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId};

use crate::world::game_event::GameEventContext;

/// Behavior for flint and steel items.
#[item_behavior]
pub struct FlintAndSteelItem;

impl FlintAndSteelItem {
    /// Shared ignition predicate for player and dispenser paths.
    pub(crate) fn can_light(state: BlockStateId) -> bool {
        can_light(state)
    }
}

impl ItemBehavior for FlintAndSteelItem {
    fn use_on(&self, context: &mut UseOnContext) -> InteractionResult {
        let click_pos = context.hit_result.block_pos;
        let clicked_state = context.world.get_block_state(click_pos);
        if try_light_block(
            context,
            click_pos,
            clicked_state,
            &sound_events::ITEM_FLINTANDSTEEL_USE,
            flint_and_steel_pitch(),
            UpdateFlags::UPDATE_ALL_IMMEDIATE,
        ) {
            let has_infinite_materials = context.player.has_infinite_materials();
            context
                .inv
                .with_item(|item| item.hurt_and_break(1, has_infinite_materials));
            return InteractionResult::Success;
        }

        let fire_pos = click_pos.relative(context.hit_result.direction);
        let forward_dir = context.player.direction_yaw();

        if !FireBlock::can_be_placed_at(context.world, fire_pos, forward_dir) {
            return InteractionResult::Fail;
        }

        context.world.play_block_sound(
            &sound_events::ITEM_FLINTANDSTEEL_USE,
            fire_pos,
            1.0,
            flint_and_steel_pitch(),
            Some(context.player.id()),
        );

        context.world.set_block(
            fire_pos,
            FireBlock::get_state(context.world.as_ref(), fire_pos),
            UpdateFlags::UPDATE_ALL_IMMEDIATE,
        );
        context.world.game_event(
            &vanilla_game_events::BLOCK_PLACE,
            click_pos,
            &GameEventContext::new(Some(context.player.as_ref()), None),
        );

        let has_infinite_materials = context.player.has_infinite_materials();
        context
            .inv
            .with_item(|item| item.hurt_and_break(1, has_infinite_materials));

        InteractionResult::Success
    }
}

/// Behavior for fire charge items.
#[item_behavior]
pub struct FireChargeItem;

impl ItemBehavior for FireChargeItem {
    fn use_on(&self, context: &mut UseOnContext) -> InteractionResult {
        let click_pos = context.hit_result.block_pos;
        let clicked_state = context.world.get_block_state(click_pos);
        if try_light_block(
            context,
            click_pos,
            clicked_state,
            &sound_events::ITEM_FIRECHARGE_USE,
            fire_charge_pitch(),
            UpdateFlags::UPDATE_ALL,
        ) {
            context.inv.with_item(ItemStack::shrink_one);
            return InteractionResult::Success;
        }

        let fire_pos = click_pos.relative(context.hit_result.direction);
        let forward_dir = context.player.direction_yaw();

        if !FireBlock::can_be_placed_at(context.world, fire_pos, forward_dir) {
            return InteractionResult::Fail;
        }

        context.world.play_block_sound(
            &sound_events::ITEM_FIRECHARGE_USE,
            fire_pos,
            1.0,
            fire_charge_pitch(),
            Some(context.player.id()),
        );

        context.world.set_block(
            fire_pos,
            FireBlock::get_state(context.world.as_ref(), fire_pos),
            UpdateFlags::UPDATE_ALL,
        );
        context.world.game_event(
            &vanilla_game_events::BLOCK_PLACE,
            fire_pos,
            &GameEventContext::new(Some(context.player.as_ref()), None),
        );

        context.inv.with_item(ItemStack::shrink_one);

        InteractionResult::Success
    }
}

fn try_light_block(
    context: &UseOnContext<'_>,
    pos: BlockPos,
    state: BlockStateId,
    sound: SoundEventRef,
    pitch: f32,
    flags: UpdateFlags,
) -> bool {
    if !can_light(state) {
        return false;
    }

    context
        .world
        .play_block_sound(sound, pos, 1.0, pitch, Some(context.player.id()));
    context.world.set_block(
        pos,
        state.set_value(&BlockStateProperties::LIT, true),
        flags,
    );
    context.world.game_event(
        &vanilla_game_events::BLOCK_CHANGE,
        pos,
        &GameEventContext::new(Some(context.player.as_ref()), None),
    );

    true
}

fn can_light(state: BlockStateId) -> bool {
    let Some(lit) = state.try_get_value(&BlockStateProperties::LIT) else {
        return false;
    };
    if lit {
        return false;
    }

    let block = state.get_block();
    if block.has_tag(&BlockTag::CAMPFIRES) {
        return state.try_get_value(&BlockStateProperties::WATERLOGGED) == Some(false);
    }

    if block.has_tag(&BlockTag::CANDLES) {
        return state.try_get_value(&BlockStateProperties::WATERLOGGED) == Some(false);
    }

    block.has_tag(&BlockTag::CANDLE_CAKES)
}

fn flint_and_steel_pitch() -> f32 {
    rand::random_range(0.8..1.2)
}

fn fire_charge_pitch() -> f32 {
    (rand::random::<f32>() - rand::random::<f32>()) * 0.2 + 1.0
}

#[cfg(test)]
mod tests {
    use steel_registry::{
        blocks::{block_state_ext::BlockStateExt, properties::BlockStateProperties},
        init_vanilla_registry, vanilla_blocks,
    };

    use super::can_light;

    #[test]
    fn can_light_rejects_waterlogged_campfires_and_candles() {
        init_vanilla_registry();

        let waterlogged_campfire = vanilla_blocks::CAMPFIRE
            .default_state()
            .set_value(&BlockStateProperties::LIT, false)
            .set_value(&BlockStateProperties::WATERLOGGED, true);
        let dry_campfire =
            waterlogged_campfire.set_value(&BlockStateProperties::WATERLOGGED, false);

        let waterlogged_candle = vanilla_blocks::CANDLE
            .default_state()
            .set_value(&BlockStateProperties::LIT, false)
            .set_value(&BlockStateProperties::WATERLOGGED, true);
        let dry_candle = waterlogged_candle.set_value(&BlockStateProperties::WATERLOGGED, false);

        assert!(!can_light(waterlogged_campfire));
        assert!(can_light(dry_campfire));
        assert!(!can_light(waterlogged_candle));
        assert!(can_light(dry_candle));
    }

    #[test]
    fn can_light_accepts_unlit_candle_cakes() {
        init_vanilla_registry();

        let unlit_candle_cake = vanilla_blocks::CANDLE_CAKE
            .default_state()
            .set_value(&BlockStateProperties::LIT, false);
        let lit_candle_cake = unlit_candle_cake.set_value(&BlockStateProperties::LIT, true);

        assert!(can_light(unlit_candle_cake));
        assert!(!can_light(lit_candle_cake));
    }

    #[test]
    fn using_a_fire_charge_in_the_air_does_not_launch_or_consume_it() {
        use std::sync::Arc;

        use steel_registry::item_stack::ItemStack;
        use steel_registry::{init_vanilla_registry, vanilla_items};
        use steel_utils::types::InteractionHand;
        use steel_utils::{ChunkPos, Downcast as _, WorldAabb};

        use crate::behavior::{ITEM_BEHAVIORS, InteractionResult, UseItemContext, init_behaviors};
        use crate::entity::Entity as _;
        use crate::entity::entities::SmallFireballEntity;
        use crate::test_support::{TestPlayerBuilder, fresh_test_world, insert_ready_full_chunk};

        init_vanilla_registry();
        init_behaviors();
        let fixture = fresh_test_world("fire_charge_use_air");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let player = TestPlayerBuilder::new(Arc::clone(world), "User", 1).build();
        player.set_client_loaded(true);
        player.inventory.lock().set_item_in_hand(
            InteractionHand::MainHand,
            ItemStack::with_count(&vanilla_items::FIRE_CHARGE, 2),
        );
        let inventory = Arc::clone(&player.inventory);
        let mut context = UseItemContext::new(&player, InteractionHand::MainHand, world, inventory);
        let result = ITEM_BEHAVIORS
            .get_behavior(&vanilla_items::FIRE_CHARGE)
            .use_item(&mut context);
        assert_eq!(result, InteractionResult::Pass);
        assert_eq!(
            player
                .inventory
                .lock()
                .get_item_in_hand(InteractionHand::MainHand)
                .count(),
            2
        );
        let pos = player.position();
        let area = WorldAabb::new(
            pos.x - 4.0,
            pos.y - 4.0,
            pos.z - 4.0,
            pos.x + 4.0,
            pos.y + 4.0,
            pos.z + 4.0,
        );
        assert!(world.get_entities_in_aabb(&area).iter().all(|entity| {
            entity
                .as_ref()
                .downcast_ref::<SmallFireballEntity>()
                .is_none()
        }));
    }

    #[test]
    fn right_clicking_air_with_fire_charge_passes_without_consuming() {
        use crate::behavior::init_behaviors;
        use crate::player::game_mode;
        use crate::test_support::{TestPlayerBuilder, fresh_test_world, insert_ready_full_chunk};
        use std::sync::Arc;
        use steel_registry::item_stack::ItemStack;
        use steel_registry::{init_vanilla_registry, vanilla_items};
        use steel_utils::ChunkPos;
        use steel_utils::types::InteractionHand;

        init_vanilla_registry();
        init_behaviors();
        let fixture = fresh_test_world("fire_charge_use_air_dispatch");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let player = TestPlayerBuilder::new(Arc::clone(world), "User", 1).build();
        player.set_client_loaded(true);
        player.inventory.lock().set_item_in_hand(
            InteractionHand::MainHand,
            ItemStack::with_count(&vanilla_items::FIRE_CHARGE, 3),
        );
        let result = game_mode::use_item(&player, world, InteractionHand::MainHand);
        assert!(!result.consumes_action());
        assert_eq!(
            player
                .inventory
                .lock()
                .get_item_in_hand(InteractionHand::MainHand)
                .count(),
            3
        );
    }
}
