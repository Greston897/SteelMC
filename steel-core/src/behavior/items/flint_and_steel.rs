//! Flint and steel and fire charge item behavior.
//!
//! Vanilla `FireChargeItem` only implements `useOn`: right-clicking a block
//! ignites it and only a dispenser launches the fireball. Steel intentionally
//! deviates from vanilla by also allowing the fire charge to be used in the
//! air: right-clicking with a fire charge in the air launches one owned small
//! fireball along the player's look direction, uses the held fire charge stack
//! as the projectile item and consumes exactly one charge.

use std::sync::Arc;

use crate::behavior::blocks::FireBlock;
use crate::behavior::context::{InteractionResult, UseItemContext, UseOnContext};
use crate::behavior::item::ItemBehavior;
use steel_macros::item_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::item_stack::ItemStack;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::stat::vanilla_stat_types;
use steel_registry::vanilla_block_tags::BlockTag;
use steel_registry::{
    blocks::{block_state_ext::BlockStateExt, properties::BlockStateProperties},
    sound_events, vanilla_entities, vanilla_game_events,
};
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId};

use crate::entity::entities::SmallFireballEntity;
use crate::entity::{Entity, next_entity_id, spawn_throwable_item_projectile};
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

/// Vertical offset (in degrees) applied to the player's pitch when the fire
/// charge is launched. Vanilla `Projectile.spawnProjectileFromRotation` uses
/// the same zero offset for hand-thrown projectiles.
const FIRE_CHARGE_SHOOT_Y_OFFSET: f32 = 0.0;

/// Speed of the launched small fireball.
const FIRE_CHARGE_SHOOT_POWER: f32 = 1.5;

/// Random spread applied to the launched small fireball.
const FIRE_CHARGE_SHOOT_UNCERTAINTY: f32 = 1.0;

/// Behavior for fire charge items.
#[item_behavior]
pub struct FireChargeItem;

impl ItemBehavior for FireChargeItem {
    /// Steel deviation from vanilla: a fire charge can also be used in the air.
    ///
    /// Vanilla `FireChargeItem` does not override `Item.use`, so
    /// right-clicking with a fire charge in the air does nothing; only a
    /// dispenser launches the fireball. Steel launches one owned small
    /// fireball along the player's look direction, renders the held fire
    /// charge stack on it and consumes exactly one charge. Like every other
    /// thrown item the charge is kept in creative mode.
    fn use_item(&self, context: &mut UseItemContext) -> InteractionResult {
        let world = context.world;
        let player = context.player;

        let mut held_stack = context.inv.with_item(|item| item.clone());

        let fireball = spawn_throwable_item_projectile(
            world,
            player,
            &mut held_stack,
            FIRE_CHARGE_SHOOT_Y_OFFSET,
            FIRE_CHARGE_SHOOT_POWER,
            FIRE_CHARGE_SHOOT_UNCERTAINTY,
            |spawn_pos| {
                SmallFireballEntity::new(
                    &vanilla_entities::SMALL_FIREBALL,
                    next_entity_id(),
                    spawn_pos,
                    Arc::downgrade(world),
                )
            },
        );
        if fireball.is_none() {
            log::debug!("world rejected the thrown fireball; still consuming the stack");
        }

        world.play_sound_at(
            &sound_events::ITEM_FIRECHARGE_USE,
            SoundSource::Blocks,
            player.position(),
            1.0,
            fire_charge_pitch(),
            None,
        );

        player.award_stat(&vanilla_stat_types::ITEM_USED, held_stack.item);
        let has_infinite_materials = player.has_infinite_materials();
        context
            .inv
            .with_item(|item| item.consume_one(has_infinite_materials));

        InteractionResult::Success
    }

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

    /// Steel deviation from vanilla: using a fire charge in the air launches
    /// one owned small fireball, renders the held fire charge stack on it and
    /// consumes exactly one charge.
    #[test]
    fn using_a_fire_charge_in_the_air_launches_an_owned_small_fireball() {
        use std::sync::Arc;

        use steel_registry::item_stack::ItemStack;
        use steel_registry::{init_vanilla_registry, vanilla_items};
        use steel_utils::types::InteractionHand;
        use steel_utils::{ChunkPos, Downcast as _, WorldAabb};

        use crate::behavior::{ITEM_BEHAVIORS, InteractionResult, UseItemContext, init_behaviors};
        use crate::entity::entities::SmallFireballEntity;
        use crate::entity::{Entity as _, Projectile as _, ThrowableItemProjectile as _};
        use crate::test_support::{TestPlayerBuilder, fresh_test_world, insert_ready_full_chunk};

        init_vanilla_registry();
        init_behaviors();
        let fixture = fresh_test_world("fire_charge_use_air");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let player = TestPlayerBuilder::new(Arc::clone(world), "Thrower", 1).build();
        player.set_client_loaded(true);
        player.set_rotation((90.0, 10.0));

        {
            let mut inventory = player.inventory.lock();
            inventory.set_item_in_hand(
                InteractionHand::MainHand,
                ItemStack::with_count(&vanilla_items::FIRE_CHARGE, 2),
            );
        }

        let behavior = ITEM_BEHAVIORS.get_behavior(&vanilla_items::FIRE_CHARGE);
        let inventory = Arc::clone(&player.inventory);
        let mut context = UseItemContext::new(&player, InteractionHand::MainHand, world, inventory);
        let result = behavior.use_item(&mut context);

        assert_eq!(result, InteractionResult::Success);

        let remaining = {
            let inventory = player.inventory.lock();
            inventory
                .get_item_in_hand(InteractionHand::MainHand)
                .count()
        };
        assert_eq!(remaining, 1, "exactly one fire charge is consumed");

        let origin = player.position();
        let search = WorldAabb::new(
            origin.x - 4.0,
            origin.y - 4.0,
            origin.z - 4.0,
            origin.x + 4.0,
            origin.y + 4.0,
            origin.z + 4.0,
        );
        let entities = world.get_entities_in_aabb(&search);
        let mut fireballs = Vec::new();
        for entity in &entities {
            if let Some(fireball) = entity.as_ref().downcast_ref::<SmallFireballEntity>() {
                fireballs.push(fireball);
            }
        }

        assert_eq!(fireballs.len(), 1, "one small fireball is launched");
        let fireball = fireballs[0];

        let owner = fireball.get_owner().expect("the fireball is owned");
        assert_eq!(owner.uuid(), player.uuid());

        let item = fireball.get_item();
        assert_eq!(item.item(), &*vanilla_items::FIRE_CHARGE);
        assert_eq!(item.count(), 1);

        let look = player.look_angle().normalize();
        let velocity = fireball.velocity().normalize();
        assert!(
            (look - velocity).length() < 0.05,
            "the fireball flies along the look direction: {look:?} vs {velocity:?}"
        );
    }

    /// The right-click-air path consumes the action and leaves exactly one
    /// fire charge in the player's hand.
    #[test]
    fn right_clicking_the_air_with_a_fire_charge_consumes_one_charge() {
        use std::sync::Arc;

        use steel_registry::item_stack::ItemStack;
        use steel_registry::{init_vanilla_registry, vanilla_items};
        use steel_utils::types::InteractionHand;
        use steel_utils::ChunkPos;

        use crate::behavior::init_behaviors;
        use crate::player::game_mode;
        use crate::test_support::{TestPlayerBuilder, fresh_test_world, insert_ready_full_chunk};

        init_vanilla_registry();
        init_behaviors();
        let fixture = fresh_test_world("fire_charge_use_air_dispatch");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let player = TestPlayerBuilder::new(Arc::clone(world), "Thrower", 1).build();
        player.set_client_loaded(true);

        {
            let mut inventory = player.inventory.lock();
            inventory.set_item_in_hand(
                InteractionHand::MainHand,
                ItemStack::with_count(&vanilla_items::FIRE_CHARGE, 3),
            );
        }

        let result = game_mode::use_item(&player, world, InteractionHand::MainHand);

        assert!(result.consumes_action());

        let remaining = {
            let inventory = player.inventory.lock();
            inventory
                .get_item_in_hand(InteractionHand::MainHand)
                .count()
        };
        assert_eq!(remaining, 2, "exactly one fire charge is consumed");
    }
}
