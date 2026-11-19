//! `RaidOmenMobEffect` behavior.

use super::MobEffectBehavior;
use crate::entity::{Entity as _, LivingEntity};
use crate::world::World;

/// Raid omen mob effect behavior.
///
/// Vanilla applies it on the final tick: the stored omen position becomes the
/// raid center, then the effect is consumed.
pub struct RaidOmenBehavior;

impl MobEffectBehavior for RaidOmenBehavior {
    fn should_apply_effect_tick_this_tick(&self, tick_count: i32, _amplifier: i32) -> bool {
        tick_count == 1
    }

    fn apply_effect_tick(&self, world: &World, user: &dyn LivingEntity, amplifier: i32) -> bool {
        let Some(player) = user.as_player() else {
            return true;
        };
        if player.is_spectator() {
            return true;
        }

        let Some(pos) = player.raid_omen_position.lock().take() else { return true; };
        if !world.get_game_rule(&steel_registry::vanilla_game_rules::RAIDS) {
            return false;
        }
        let center = crate::world::raid::village_center_world(world, pos).unwrap_or(pos.0);
        world.raids.raise_omen(center, amplifier);
        player.trigger_advancement_with_context(
            &steel_registry::vanilla_criteria_triggers::VOLUNTARY_EXILE,
            &crate::player::advancement::predicate::TriggerContext::at(
                &player.get_world(), player.position(),
            ),
        );
        false
    }
}
