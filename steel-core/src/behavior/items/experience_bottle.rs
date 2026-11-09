//! Experience bottle item behavior (`ExperienceBottleItem`).
//!
//! Mirrors vanilla `ExperienceBottleItem.use`: throws an
//! [`ExperienceBottleEntity`] with a -20° pitch offset, 0.7 power and vanilla's
//! throw sound, then awards `ITEM_USED` and consumes one bottle.

use std::sync::Arc;

use steel_macros::item_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::{sound_events, vanilla_entities};

use super::throw_projectile::{ThrowParams, throw_item_projectile};
use crate::behavior::context::{InteractionResult, UseItemContext};
use crate::behavior::item::ItemBehavior;
use crate::entity::entities::ExperienceBottleEntity;
use crate::entity::next_entity_id;

/// Vanilla `ExperienceBottleItem` throw parameters
/// (`spawnWithVelocity(..., THROW_PITCH = -20, THROW_POWER = 0.7, 1.0)`).
const THROW: ThrowParams = ThrowParams {
    sound: &sound_events::ENTITY_EXPERIENCE_BOTTLE_THROW,
    sound_source: SoundSource::Neutral,
    sound_volume: 0.5,
    y_offset: -20.0,
    power: 0.7,
    uncertainty: 1.0,
};

/// Behavior for the experience bottle item.
#[item_behavior]
pub struct ExperienceBottleItem;

impl ItemBehavior for ExperienceBottleItem {
    fn use_item(&self, context: &mut UseItemContext) -> InteractionResult {
        let world = context.world;
        throw_item_projectile(context, &THROW, |spawn_pos| {
            ExperienceBottleEntity::new(
                &vanilla_entities::EXPERIENCE_BOTTLE,
                next_entity_id(),
                spawn_pos,
                Arc::downgrade(world),
            )
        });
        InteractionResult::Success
    }
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
    use crate::entity::entities::ExperienceBottleEntity;
    use crate::inventory::container::Container as _;
    use crate::test_support::{TestPlayerBuilder, fresh_test_world, insert_ready_full_chunk};

    /// Vanilla `ExperienceBottleItem.use` throws one bottle and consumes it.
    #[test]
    fn using_an_experience_bottle_throws_it_and_consumes_one() {
        init_vanilla_registry();
        init_behaviors();
        let fixture = fresh_test_world("experience_bottle_use");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let player = TestPlayerBuilder::new(Arc::clone(world), "Thrower", 1).build();
        player.set_client_loaded(true);

        {
            let mut inventory = player.inventory.lock();
            inventory.set_item_in_hand(
                InteractionHand::MainHand,
                ItemStack::with_count(&vanilla_items::EXPERIENCE_BOTTLE, 2),
            );
        }

        let behavior = ITEM_BEHAVIORS.get_behavior(&vanilla_items::EXPERIENCE_BOTTLE);
        let inventory = Arc::clone(&player.inventory);
        let mut context =
            UseItemContext::new(&player, InteractionHand::MainHand, world, inventory);
        let result = behavior.use_item(&mut context);

        assert_eq!(result, InteractionResult::Success);
        let remaining = {
            let inventory = player.inventory.lock();
            inventory.get_item_in_hand(InteractionHand::MainHand).count()
        };
        assert_eq!(remaining, 1);

        let bottles = world
            .get_entities_in_aabb(&WorldAabb::new(-4.0, -4.0, -4.0, 4.0, 8.0, 4.0))
            .iter()
            .filter(|entity| {
                entity
                    .as_ref()
                    .downcast_ref::<ExperienceBottleEntity>()
                    .is_some()
            })
            .count();
        assert_eq!(bottles, 1);
    }
}
