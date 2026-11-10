//! Trident item behavior (`TridentItem`).
//!
//! Vanilla `TridentItem` is a 72000-tick charged item: holding right-click for
//! at least 10 ticks and releasing throws the trident as a `ThrownTrident`
//! projectile at `2.5` blocks/tick with `0.5` uncertainty, costs one durability
//! point and removes the stack (creative keeps it), plays
//! `item.trident.throw` and awards `ITEM_USED`.
//!
//! Differences from vanilla, all deliberate:
//! * The `riptide` spin attack needs charge-tick movement and damage while
//!   swimming; Steel has no riptide implementation yet, so a fully charged
//!   throw always throws the trident instead of starting a spin attack.
//! * `Loyalty` is read from the stack's enchantments and handed to the
//!   projectile, which implements the return trip.

use std::sync::Arc;

use steel_macros::item_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::item_stack::ItemStack;
use steel_registry::stat::vanilla_stat_types;
use steel_registry::{sound_events, vanilla_entities, vanilla_enchantments};
use steel_utils::types::GameType;

use crate::behavior::context::{InteractionResult, UseItemContext};
use crate::behavior::item::ItemBehavior;
use crate::behavior::ItemUseAnimation;
use crate::entity::entities::{AbstractArrow as _, ArrowPickup, ThrownTridentEntity};
use crate::entity::{Entity, LivingEntity, next_entity_id, spawn_projectile_from_rotation};
use crate::world::World;

/// Vanilla `TridentItem.MAX_USE_DURATION`.
pub const MAX_USE_TICKS: i32 = 72000;

/// Vanilla `TridentItem.THROW_THRESHOLD_TIME`: minimum charge before a throw.
pub const THROW_THRESHOLD_TICKS: i32 = 10;

/// Vanilla `TridentItem.SHOOT_POWER`.
pub const SHOOT_POWER: f32 = 2.5;

/// Vanilla `Projectile.spawnProjectileFromRotation` uncertainty used by tridents.
pub const SHOOT_UNCERTAINTY: f32 = 0.5;

/// Behavior for the trident item.
#[item_behavior(class = "TridentItem")]
pub struct TridentItem;

impl TridentItem {
    /// Ticks the player must hold the item before `release_using` throws it.
    #[must_use]
    pub const fn charge_ticks(time_left: i32) -> i32 {
        MAX_USE_TICKS - time_left
    }

    /// Vanilla `AbstractArrow.Pickup` for a freshly thrown trident.
    #[must_use]
    pub const fn pickup_for(creative: bool) -> ArrowPickup {
        if creative {
            ArrowPickup::CreativeOnly
        } else {
            ArrowPickup::Allowed
        }
    }
}

impl ItemBehavior for TridentItem {
    fn get_use_duration(&self, _stack: &ItemStack, _user: &dyn LivingEntity) -> i32 {
        MAX_USE_TICKS
    }

    fn get_use_animation(&self, _stack: &ItemStack) -> ItemUseAnimation {
        ItemUseAnimation::Spear
    }

    fn use_on_release(&self, _stack: &ItemStack) -> bool {
        true
    }

    fn use_item(&self, context: &mut UseItemContext) -> InteractionResult {
        // Vanilla `TridentItem.use`: a trident that would break on the next
        // durability point refuses to start charging.
        if context.inv.with_item(ItemStack::next_damage_will_break) {
            return InteractionResult::Fail;
        }
        context.player.start_using_item(context.hand);
        InteractionResult::Consume
    }

    fn release_using(
        &self,
        stack: &mut ItemStack,
        world: &Arc<World>,
        user: &dyn LivingEntity,
        time_left: i32,
    ) -> bool {
        let Some(player) = user.as_player() else {
            return false;
        };
        if Self::charge_ticks(time_left) < THROW_THRESHOLD_TICKS {
            return false;
        }
        if stack.next_damage_will_break() {
            return false;
        }

        let creative = player.game_mode() == GameType::Creative;
        let loyalty = stack.get_enchantment_level(&vanilla_enchantments::LOYALTY.key);
        let thrown_stack = stack.copy_with_count(1);

        let trident = spawn_projectile_from_rotation(
            world,
            player,
            0.0,
            SHOOT_POWER,
            SHOOT_UNCERTAINTY,
            |spawn_pos| {
                ThrownTridentEntity::new(
                    &vanilla_entities::TRIDENT,
                    next_entity_id(),
                    spawn_pos,
                    Arc::downgrade(world),
                    thrown_stack,
                    loyalty,
                )
            },
            |trident| trident.set_pickup(Self::pickup_for(creative)),
        );

        if trident.is_some() {
            world.play_sound(
                &sound_events::ITEM_TRIDENT_THROW,
                SoundSource::Players,
                player.block_position(),
                1.0,
                1.0,
                None,
            );
        }

        // Vanilla costs a durability point, then shrinks the stack in survival.
        stack.hurt_and_break(1, creative);
        if !creative {
            stack.consume_one(false);
        }
        player.award_stat(&vanilla_stat_types::ITEM_USED, stack.item());
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_USE_TICKS, SHOOT_POWER, THROW_THRESHOLD_TICKS, TridentItem};
    use crate::entity::entities::ArrowPickup;

    #[test]
    fn a_throw_needs_ten_ticks_of_charge() {
        assert_eq!(THROW_THRESHOLD_TICKS, 10);
        assert_eq!(TridentItem::charge_ticks(MAX_USE_TICKS), 0);
        assert_eq!(TridentItem::charge_ticks(MAX_USE_TICKS - 9), 9);
        assert_eq!(TridentItem::charge_ticks(MAX_USE_TICKS - 10), 10);
    }

    #[test]
    fn thrown_tridents_use_the_vanilla_shoot_power() {
        assert!((SHOOT_POWER - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn creative_throws_are_creative_only_pickups() {
        assert_eq!(TridentItem::pickup_for(true), ArrowPickup::CreativeOnly);
        assert_eq!(TridentItem::pickup_for(false), ArrowPickup::Allowed);
    }
}
