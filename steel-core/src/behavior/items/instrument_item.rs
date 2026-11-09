//! Goat horn behavior implementation.
//!
//! Vanilla class: `GoatHornItem` (the 26.3 behavior dump names the class
//! `InstrumentItem`). Using the item plays the sound of the stack's
//! `minecraft:instrument` component at `range / 16` volume on the `RECORDS`
//! sound category, emits `GameEvent.INSTRUMENT_PLAY` and puts the item on that
//! instrument's cooldown. The item is then "used" for
//! `floor(useDuration * 20)` ticks with the `TOOT_HORN` animation.

use steel_macros::item_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::data_components::vanilla_components::INSTRUMENT;
use steel_registry::item_stack::ItemStack;
use steel_registry::stat::vanilla_stat_types;
use steel_registry::vanilla_game_events;

use crate::behavior::{InteractionResult, ItemBehavior, ItemUseAnimation, UseItemContext};
use crate::entity::{Entity, LivingEntity};

/// Java's `TICKS_PER_SECOND`, kept as `f32` because instrument use durations
/// are vanilla floats multiplied before flooring.
const TICKS_PER_SECOND: f32 = 20.0;

/// Goat horn behavior.
#[item_behavior]
pub struct InstrumentItem;

impl InstrumentItem {
    /// `Floor(useDuration * 20)` for the stack's instrument, or `0` when the
    /// stack has no instrument component.
    fn use_ticks(stack: &ItemStack) -> i32 {
        let Some(instrument) = stack.get(INSTRUMENT) else {
            return 0;
        };
        (instrument.value().use_duration() * TICKS_PER_SECOND).floor() as i32
    }
}

impl ItemBehavior for InstrumentItem {
    /// Mirrors vanilla `GoatHornItem.use`: without an instrument component the
    /// use fails, otherwise the horn is played and the item goes on cooldown.
    fn use_item(&self, context: &mut UseItemContext) -> InteractionResult {
        let component = context.inv.with_item(|stack| stack.get(INSTRUMENT).cloned());
        let Some(component) = component else {
            return InteractionResult::Fail;
        };
        let instrument = component.value();
        let use_ticks = (instrument.use_duration() * TICKS_PER_SECOND).floor() as i32;
        let item = context.inv.with_item(ItemStack::item);

        context.player.start_using_item(context.hand);
        context.world.play_sound_holder_at(
            instrument.sound_event(),
            SoundSource::Records,
            context.player.position(),
            instrument.range() / 16.0,
            1.0,
            None,
        );
        context.player.game_event(&vanilla_game_events::INSTRUMENT_PLAY);
        context.player.set_item_cooldown(item.key.clone(), use_ticks);
        context.player.award_stat(&vanilla_stat_types::ITEM_USED, item);
        InteractionResult::Consume
    }

    fn get_use_animation(&self, _stack: &ItemStack) -> ItemUseAnimation {
        ItemUseAnimation::TootHorn
    }

    fn get_use_duration(&self, stack: &ItemStack, _user: &dyn LivingEntity) -> i32 {
        Self::use_ticks(stack)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use steel_registry::data_components::vanilla_components::INSTRUMENT;
    use steel_registry::item_stack::ItemStack;
    use steel_registry::stat::vanilla_stat_types;
    use steel_registry::{init_vanilla_registry, vanilla_items};
    use steel_utils::ChunkPos;
    use steel_utils::types::InteractionHand;

    use crate::behavior::{
        ITEM_BEHAVIORS, InteractionResult, ItemUseAnimation, UseItemContext, init_behaviors,
    };
    use crate::inventory::container::Container as _;
    use crate::test_support::{TestPlayerBuilder, fresh_test_world, insert_ready_full_chunk};

    /// Vanilla `GoatHornItem.use`: the horn starts a `TOOT_HORN` use whose
    /// length comes from the instrument, and the item is put on that
    /// instrument's cooldown so it cannot be spammed.
    #[test]
    fn using_a_goat_horn_starts_its_instrument_cooldown() {
        init_vanilla_registry();
        init_behaviors();
        let fixture = fresh_test_world("goat_horn_use");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let player = TestPlayerBuilder::new(Arc::clone(world), "Horn", 1).build();
        player.set_client_loaded(true);

        let stack = ItemStack::new(&vanilla_items::GOAT_HORN);
        let instrument = stack.get(INSTRUMENT).expect("goat horn instrument");
        let use_duration = instrument.value().use_duration();
        let expected_ticks = (use_duration * 20.0).floor() as i32;

        let horn = ItemStack::new(stack.item());
        {
            let mut inventory = player.inventory.lock();
            inventory.set_item_in_hand(InteractionHand::MainHand, horn);
        }

        let behavior = ITEM_BEHAVIORS.get_behavior(&vanilla_items::GOAT_HORN);
        let animation = behavior.get_use_animation(&stack);
        assert_eq!(animation, ItemUseAnimation::TootHorn);
        let duration = behavior.get_use_duration(&stack, player.as_ref());
        assert_eq!(duration, expected_ticks);

        let mut context = UseItemContext::new(
            &player,
            InteractionHand::MainHand,
            world,
            Arc::clone(&player.inventory),
        );
        let result = behavior.use_item(&mut context);
        assert_eq!(result, InteractionResult::Consume);
        assert!(player.is_item_on_cooldown(&stack));

        let used_stat = vanilla_stat_types::ITEM_USED.get(&vanilla_items::GOAT_HORN);
        let used = player
            .stats()
            .into_iter()
            .find(|(stat, _)| *stat == used_stat);
        assert_eq!(used.map(|(_, count)| count), Some(1));
    }
}
