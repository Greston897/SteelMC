//! `BadOmenMobEffect` behavior.

use super::MobEffectBehavior;
use crate::entity::{Entity as _, LivingEntity};
use crate::world::{World, raid};

/// Bad omen mob effect behavior.
///
/// Vanilla `BadOmenMobEffect.applyEffectTick`: a non-spectator player standing in
/// a village with no raid at the maximum omen level starts (or extends) a raid and
/// then loses the effect.
pub struct BadOmenBehavior;

impl MobEffectBehavior for BadOmenBehavior {
    fn should_apply_effect_tick_this_tick(&self, _tick_count: i32, _amplifier: i32) -> bool {
        true
    }

    fn apply_effect_tick(&self, world: &World, user: &dyn LivingEntity, amplifier: i32) -> bool {
        let Some(player) = user.as_player() else {
            return true;
        };
        if player.is_spectator() {
            return true;
        }

        let pos = player.block_position();
        let Some(center) = raid::village_center_world(world, pos) else {
            return true;
        };

        // Vanilla refuses to extend a raid that already reached the omen cap.
        if let Some(existing) = world.raids.raid_at(center) {
            if existing.omen_level >= raid::MAX_OMEN_LEVEL {
                return true;
            }
        }

        world.raids.raise_omen(center, amplifier);
        // Vanilla `BadOmenMobEffect.applyEffectTick` fires `CriteriaTriggers.
        // VOLUNTARY_EXILE` on the player that caused the raid, which is exactly
        // this player walking bad omen into a village.
        player.trigger_advancement_with_context(
            &steel_registry::vanilla_criteria_triggers::VOLUNTARY_EXILE,
            &crate::player::advancement::predicate::TriggerContext::at(
                &player.get_world(),
                player.block_position(),
            ),
        );
        true
    }
}
