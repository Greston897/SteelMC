//! `RaidOmenMobEffect` behavior.

use glam::IVec3;

use super::MobEffectBehavior;
use crate::entity::{Entity as _, LivingEntity};
use crate::world::{World, raid};

/// Raid omen mob effect behavior.
///
/// Vanilla applies it on the first tick: the stored omen position becomes the
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

        // The raid that raid omen starts is centered on the omen source; vanilla
        // stores it on the effect. Steel only records the player's position, which
        // serves the same purpose for a pending omen.
        let pos = player.block_position();
        world
            .raids
            .raise_omen(IVec3::new(pos.x(), pos.y(), pos.z()), amplifier);
        true
    }
}
