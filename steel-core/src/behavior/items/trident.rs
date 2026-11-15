//! Trident item behavior (`TridentItem`).
//!
//! Vanilla `TridentItem` is a 72000-tick charged item: holding right-click for
//! at least 10 ticks and releasing throws the trident as a `ThrownTrident`
//! projectile at `2.5` blocks/tick with `0.5` uncertainty, costs one durability
//! point and removes the stack (creative keeps it), plays
//! `item.trident.throw` and awards `ITEM_USED`.
//!
//! Riptide follows vanilla: in water or rain the trident starts a spin attack
//! instead of throwing, scaled by the enchantment level (see
//! `Player::start_auto_spin_attack`). Out of water and rain a charged release
//! throws the trident.
//!
//! `Loyalty` is read from the stack's enchantments and handed to the
//! projectile, which implements the return trip.

use std::sync::Arc;

use steel_macros::item_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::item_stack::ItemStack;
use steel_registry::stat::vanilla_stat_types;
use steel_registry::{sound_events, vanilla_enchantments, vanilla_entities};
use steel_utils::types::GameType;

use crate::behavior::ItemUseAnimation;
use crate::behavior::context::{InteractionResult, UseItemContext};
use crate::behavior::item::ItemBehavior;
use crate::entity::entities::{AbstractArrow as _, ArrowPickup, ThrownTridentEntity};
use glam::DVec3;
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

    /// Vanilla `TridentItem.releaseUsing` riptide branch.
    ///
    /// Computes the look direction from the *`f32`* rotation with the sine table
    /// (exactly like `Mth.sin`/`Mth.cos`), normalises it, pushes the player with
    /// `strength / length` and starts the spin attack. The upward hop only
    /// happens on the ground.
    fn launch_riptide(
        player: &crate::player::Player,
        user: &dyn LivingEntity,
        level: i32,
        strength: f32,
        world: &Arc<World>,
    ) {
        let (yaw, pitch) = Entity::rotation(user);
        let yaw_rad = yaw * steel_math::DEG_TO_RAD;
        let pitch_rad = pitch * steel_math::DEG_TO_RAD;
        let sin_yaw = steel_math::trig::sin(f64::from(yaw_rad));
        let cos_yaw = steel_math::trig::cos(f64::from(yaw_rad));
        let sin_pitch = steel_math::trig::sin(f64::from(pitch_rad));
        let cos_pitch = steel_math::trig::cos(f64::from(pitch_rad));

        let xd = -sin_yaw * cos_pitch;
        let yd = -sin_pitch;
        let zd = cos_yaw * cos_pitch;
        let length = (xd * xd + yd * yd + zd * zd).sqrt();
        if length > 0.0 {
            let scale = strength / length;
            let push = DVec3::new(
                f64::from(xd * scale),
                f64::from(yd * scale),
                f64::from(zd * scale),
            );
            player.set_velocity(player.velocity() + push);
            player.mark_velocity_sync();
        }

        player.start_auto_spin_attack(level);
        if player.on_ground() {
            // Vanilla hops the player 1.1999999284744263 blocks up on release.
            player.set_velocity(player.velocity() + DVec3::new(0.0, 1.199_999_928_474_426_3, 0.0));
            player.mark_velocity_sync();
        }
        // Vanilla picks the highest `trident_sound` enchantment effect, which for
        // riptide is `item.trident.riptide_{1,2,3}` by level, and falls back to
        // the throw sound.
        let sound = match level {
            1 => &sound_events::ITEM_TRIDENT_RIPTIDE_1,
            2 => &sound_events::ITEM_TRIDENT_RIPTIDE_2,
            _ => &sound_events::ITEM_TRIDENT_RIPTIDE_3,
        };
        world.play_sound(
            sound,
            SoundSource::Players,
            player.block_position(),
            1.0,
            1.0,
            None,
        );
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
        // Vanilla `TridentItem.getUseAnimation` is `ItemUseAnimation.TRIDENT`.
        ItemUseAnimation::Trident
    }

    fn use_on_release(&self, _stack: &ItemStack) -> bool {
        true
    }

    fn use_item(&self, context: &mut UseItemContext) -> InteractionResult {
        // Vanilla `TridentItem.use`: a trident that would break on the next
        // durability point refuses to start charging.
        if context
            .inv
            .with_item(|stack| stack.next_damage_will_break())
        {
            return InteractionResult::Fail;
        }

        // Vanilla `use` does *not* spin here: riptide only refuses to start
        // charging while the player is dry. The spin itself happens on release.
        let strength = context.inv.with_item(|stack| {
            let level = stack.get_enchantment_level(&vanilla_enchantments::RIPTIDE.key);
            context.player.trident_spin_attack_strength(level)
        });
        if strength > 0.0 && !Entity::is_in_water_or_rain(&**context.player) {
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
        let level = stack.get_enchantment_level(&vanilla_enchantments::RIPTIDE.key);
        let strength = player.trident_spin_attack_strength(level);
        // Vanilla: a riptide trident cannot be released while dry or while riding.
        if strength > 0.0 && (!Entity::is_in_water_or_rain(user) || user.is_passenger()) {
            return false;
        }
        if stack.next_damage_will_break() {
            return false;
        }

        // Vanilla `TridentItem.releaseUsing`: the stack loses one durability
        // point first, then riptide launches the player instead of throwing.
        player.award_stat(&vanilla_stat_types::ITEM_USED, stack.item());
        stack.hurt_and_break(1, player.game_mode() == GameType::Creative);
        if strength > 0.0 {
            Self::launch_riptide(player, user, level, strength, world);
            return true;
        }

        let creative = player.game_mode() == GameType::Creative;
        let loyalty = stack.get_enchantment_level(&vanilla_enchantments::LOYALTY.key);
        let thrown_stack = stack.copy_with_count(1);

        let Some(shooter) = world.get_entity_by_id(user.id()) else {
            return false;
        };

        let trident = spawn_projectile_from_rotation(
            world,
            &shooter,
            0.0,
            SHOOT_POWER,
            SHOOT_UNCERTAINTY,
            |spawn_pos| {
                ThrownTridentEntity::with_throw(
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

        // Vanilla `consumeAndReturn(1, player)`: survival throws the whole stack,
        // creative keeps it.
        if !creative {
            stack.consume_one(false);
        }
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
