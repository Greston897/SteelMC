//! Bow (`BowItem`).
//!
//! Vanilla bow behaviour: a `72000`-tick draw that reports the `bow` use
//! animation, and a release that converts the draw time into the vanilla
//! `(f² + 2f) / 3` power curve, shoots the arrow at `power * 3`, marks full draws
//! critical, plays `entity.arrow.shoot`, consumes one arrow (except in creative)
//! and adds one point of durability damage to the bow.

use std::sync::Arc;

use glam::DVec3;
use steel_macros::item_behavior;
use steel_protocol::packets::game::SoundSource;

use steel_registry::item_stack::ItemStack;
use steel_registry::{sound_events, vanilla_items, vanilla_stat_types};
use steel_utils::types::GameType;

use crate::behavior::{ItemBehavior, ItemUseAnimation};
use crate::behavior::items::{ArrowItem, SpectralArrowItem};
use crate::entity::entities::{AbstractArrow, ArrowPickup};
use crate::entity::{Entity, Player, Projectile as _, SharedEntity};
use crate::entity::LivingEntity;
use crate::world::World;

/// Vanilla `BowItem` maximum draw duration.
pub const MAX_DRAW_TICKS: i32 = 72000;

/// Vanilla minimum charge fraction before an arrow is released.
pub const MIN_POWER: f32 = 0.1;

/// Vanilla arrow launch speed multiplier (`power * 3.0`).
pub const ARROW_SPEED_SCALE: f32 = 3.0;

/// Vanilla `ItemStack#getMaxUseTime` for bows.
#[item_behavior(class = "BowItem")]
pub struct BowItem;

impl BowItem {
    /// Vanilla `BowItem#getPullProgress`: `(f² + 2f) / 3`, clamped to `[0, 1]`.
    #[must_use]
    pub fn power_for_ticks(ticks: i32) -> f32 {
        let f = (ticks as f32 / 20.0).clamp(0.0, 1.0);
        ((f * f + f * 2.0) / 3.0).clamp(0.0, 1.0)
    }

    /// Vanilla `BowItem#getWeaponStack` equivalent: the stack currently drawn.
    #[must_use]
    pub fn arrow_pickup(creative: bool) -> ArrowPickup {
        if creative {
            ArrowPickup::CreativeOnly
        } else {
            ArrowPickup::Allowed
        }
    }

    /// Finds the first arrow in the inventory (hotbar first, then main slots).
    #[must_use]
    pub fn find_arrow_slot(player: &Player) -> Option<u8> {
        let inventory = player.inventory.lock();
        inventory
            .get_items()
            .iter()
            .position(|stack| ArrowItem::is_arrow(stack))
            .map(|index| index as u8)
    }
}

/// Vanilla `Projectile#shootFromRotation` plus the bow-specific critical flag.
fn launch_arrow<A: AbstractArrow + 'static>(
    arrow: Arc<A>,
    world: &Arc<World>,
    shooter: &SharedEntity,
    power: f32,
    weapon: &ItemStack,
) {
    arrow.set_arrow_damage(2.0);
    arrow.set_critical(power >= 1.0);
    let (yaw, pitch) = shooter.rotation();
    arrow.shoot_from_rotation(
        shooter.as_ref(),
        pitch,
        yaw,
        0.0,
        power * ARROW_SPEED_SCALE,
        1.0,
    );
    arrow.set_has_been_shot(true);
    arrow.set_weapon(weapon.copy_with_count(1));

    let shared: SharedEntity = arrow;
    if let Err(error) = world.try_add_entity(shared) {
        log::debug!("failed to spawn bow arrow: {error}");
    }
}

impl ItemBehavior for BowItem {
    fn get_use_duration(&self, _stack: &ItemStack, _user: &dyn LivingEntity) -> i32 {
        MAX_DRAW_TICKS
    }

    fn get_use_animation(&self, _stack: &ItemStack) -> ItemUseAnimation {
        ItemUseAnimation::Bow
    }

    fn use_on_release(&self, _stack: &ItemStack) -> bool {
        true
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

        let power = Self::power_for_ticks(MAX_DRAW_TICKS - time_left);
        if power < MIN_POWER {
            return false;
        }

        let creative = player.game_mode() == GameType::Creative;
        let pickup = Self::arrow_pickup(creative);

        // Vanilla shoots only with a matching arrow in the inventory.
        let Some(slot) = Self::find_arrow_slot(player) else {
            return false;
        };
        let arrow_stack = player
            .inventory
            .lock()
            .get_items()
            .get(usize::from(slot))
            .cloned()
            .unwrap_or_else(ItemStack::empty);
        if arrow_stack.is_empty() {
            return false;
        }

        let Some(shooter) = world.get_entity_by_id(player.id()) else {
            return false;
        };

        // Vanilla spawns the projectile slightly off the shooter's eye height.
        let position = player.position() + DVec3::new(0.0, 1.5, 0.0);
        let weapon = stack.copy_with_count(1);
        if arrow_stack.is(&vanilla_items::SPECTRAL_ARROW) {
            let arrow = SpectralArrowItem::create_arrow(
                &arrow_stack,
                position,
                world,
                Some(&shooter),
                pickup,
            );
            launch_arrow(arrow, world, &shooter, power, &weapon);
        } else {
            let arrow =
                ArrowItem::create_arrow(&arrow_stack, position, world, Some(&shooter), pickup);
            launch_arrow(arrow, world, &shooter, power, &weapon);
        }

        if !creative {
            let inventory = Arc::clone(&player.inventory);
            let mut inventory = inventory.lock();
            inventory.with_item_mut(usize::from(slot), |stack| stack.consume_one(false));
        }

        // Vanilla costs one durability point and awards `ITEM_USED`.
        stack.hurt_and_break(1, creative);
        player.award_stat(&vanilla_stat_types::ITEM_USED, stack.item());
        world.play_sound(
            &sound_events::ENTITY_ARROW_SHOOT,
            SoundSource::Players,
            player.block_position(),
            1.0,
            1.0 / (rand::random::<f32>() * 0.4 + 1.2),
            None,
        );

        true
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::item_stack::ItemStack;
    use steel_registry::{init_vanilla_registry, vanilla_items};

    use super::BowItem;
    use crate::behavior::items::ArrowItem;
    use crate::entity::entities::ArrowPickup;

    #[test]
    fn the_power_curve_matches_vanilla_pull_progress() {
        assert!((BowItem::power_for_ticks(0) - 0.0).abs() < 1.0e-6);
        // 10 ticks = f 0.5 -> (0.25 + 1.0) / 3
        assert!((BowItem::power_for_ticks(10) - (1.25 / 3.0)).abs() < 1.0e-6);
        // 20 ticks = full draw
        assert!((BowItem::power_for_ticks(20) - 1.0).abs() < 1.0e-6);
        // Longer draws stay clamped at full power.
        assert!((BowItem::power_for_ticks(200) - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn creative_players_get_creative_only_pickup() {
        assert_eq!(BowItem::arrow_pickup(true), ArrowPickup::CreativeOnly);
        assert_eq!(BowItem::arrow_pickup(false), ArrowPickup::Allowed);
    }

    #[test]
    fn all_three_arrow_items_are_accepted() {
        init_vanilla_registry();
        assert!(ArrowItem::is_arrow(&ItemStack::new(&vanilla_items::ARROW)));
        assert!(ArrowItem::is_arrow(&ItemStack::new(
            &vanilla_items::TIPPED_ARROW
        )));
        assert!(ArrowItem::is_arrow(&ItemStack::new(
            &vanilla_items::SPECTRAL_ARROW
        )));
        assert!(!ArrowItem::is_arrow(&ItemStack::new(&vanilla_items::STICK)));
    }
}
