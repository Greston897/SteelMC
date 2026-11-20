//! Wind charge item behavior (`WindChargeItem`).
//!
//! Using a wind charge spawns a [`WindChargeEntity`] from the player's eye,
//! shot along their look direction, plays the throw sound, starts vanilla's
//! 10-tick cooldown and consumes one charge unless the player has infinite
//! materials. Mirrors vanilla `WindChargeItem.use`.

use std::sync::Arc;

use steel_macros::item_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::stat::vanilla_stat_types;
use steel_registry::{sound_events, vanilla_entities};

use crate::behavior::context::{InteractionResult, UseItemContext};
use crate::behavior::item::ItemBehavior;
use crate::entity::entities::WindChargeEntity;
use crate::entity::{Entity as _, SharedEntity, next_entity_id, spawn_projectile_from_rotation};
use crate::player::Player;

/// Vanilla `WindChargeItem` shoot power.
const SHOOT_POWER: f32 = 1.5;

/// Vanilla `WindChargeItem` shoot uncertainty.
const SHOOT_UNCERTAINTY: f32 = 1.0;

/// Vanilla `WindChargeItem` `getCooldowns().addCooldown(stack, 10)`.
pub const COOLDOWN_TICKS: i32 = 10;

/// Behaviour for the wind charge item.
#[item_behavior(class = "WindChargeItem")]
pub struct WindChargeItem;

impl ItemBehavior for WindChargeItem {
    fn use_item(&self, context: &mut UseItemContext) -> InteractionResult {
        let world = context.world;
        let player = context.player;

        let shooter: SharedEntity = Arc::clone(player) as SharedEntity;
        let charge: Option<SharedEntity> = spawn_projectile_from_rotation(
            world,
            &shooter,
            0.0,
            SHOOT_POWER,
            SHOOT_UNCERTAINTY,
            |spawn_pos| {
                WindChargeEntity::new(
                    &vanilla_entities::WIND_CHARGE,
                    next_entity_id(),
                    spawn_pos,
                    Arc::downgrade(world),
                )
            },
            |_| {},
        );

        if charge.is_some() {
            play_throw_sound(player);
            let stack = context.inv.with_item(|stack| stack.clone());
            player.set_item_cooldown(stack.item.key.clone(), COOLDOWN_TICKS);
        }

        player.award_stat(&vanilla_stat_types::ITEM_USED, {
            context.inv.with_item(|stack| stack.item)
        });
        let has_infinite_materials = player.has_infinite_materials();
        context
            .inv
            .with_item(|stack| stack.consume_one(has_infinite_materials));

        InteractionResult::Success
    }
}

/// Vanilla `WindChargeItem` throw sound with the `0.4 / (random * 0.4 + 0.8)` pitch.
fn play_throw_sound(player: &Arc<Player>) {
    let pitch = 0.4 / (rand::random_range(0.0_f32..1.0) * 0.4 + 0.8);
    let world = player.get_world();
    world.play_sound_at(
        &sound_events::ENTITY_WIND_CHARGE_THROW,
        SoundSource::Neutral,
        player.position(),
        0.5,
        pitch,
        None,
    );
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use steel_registry::item_stack::ItemStack;
    use steel_registry::{init_vanilla_registry, vanilla_items};
    use steel_utils::types::InteractionHand;
    use steel_utils::{ChunkPos, Downcast as _, WorldAabb};

    use crate::behavior::{ITEM_BEHAVIORS, InteractionResult, UseItemContext, init_behaviors};
    use crate::entity::Entity as _;
    use crate::entity::entities::WindChargeEntity;
    use crate::test_support::{TestPlayerBuilder, fresh_test_world, insert_ready_full_chunk};

    /// Vanilla `ExperienceBottleItem.use` throws one bottle and consumes it.
    #[test]
    fn using_an_wind_charge_throws_it_and_consumes_one() {
        init_vanilla_registry();
        init_behaviors();
        let fixture = fresh_test_world("wind_charge_use");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let player = TestPlayerBuilder::new(Arc::clone(world), "Thrower", 1).build();
        player.set_client_loaded(true);

        {
            let mut inventory = player.inventory.lock();
            inventory.set_item_in_hand(
                InteractionHand::MainHand,
                ItemStack::with_count(&vanilla_items::WIND_CHARGE, 2),
            );
        }

        let behavior = ITEM_BEHAVIORS.get_behavior(&vanilla_items::WIND_CHARGE);
        let inventory = Arc::clone(&player.inventory);
        let mut context = UseItemContext::new(&player, InteractionHand::MainHand, world, inventory);
        let result = behavior.use_item(&mut context);

        assert_eq!(result, InteractionResult::Success);
        assert!(player.is_item_on_cooldown(&ItemStack::new(&vanilla_items::WIND_CHARGE)));
        let remaining = {
            let inventory = player.inventory.lock();
            inventory
                .get_item_in_hand(InteractionHand::MainHand)
                .count()
        };
        assert_eq!(remaining, 1);

        let eye = player.position();
        let search = WorldAabb::new(
            eye.x - 4.0,
            eye.y - 4.0,
            eye.z - 4.0,
            eye.x + 4.0,
            eye.y + 4.0,
            eye.z + 4.0,
        );
        let charges = world
            .get_entities_in_aabb(&search)
            .iter()
            .filter(|entity| entity.as_ref().downcast_ref::<WindChargeEntity>().is_some())
            .count();
        assert_eq!(charges, 1);
    }
}
