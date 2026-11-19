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
        if !raid::is_village_world(world, pos) {
            return true;
        }

        // Vanilla refuses to extend a raid that already reached the omen cap.
        if let Some(existing) = world.raids.raid_at(pos.0)
            && existing.omen_level >= raid::MAX_OMEN_LEVEL
        {
            return true;
        }

        if world.difficulty() == steel_utils::types::Difficulty::Peaceful {
            return true;
        }
        player.add_mob_effect(crate::entity::MobEffectInstance::with_duration(
            steel_registry::vanilla_mob_effects::RAID_OMEN,
            600,
            amplifier,
        ));
        *player.raid_omen_position.lock() = Some(pos);
        false
    }
}
