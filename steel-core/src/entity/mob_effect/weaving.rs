//! `WeavingMobEffect` behavior.

use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::vanilla_blocks;
use steel_utils::types::UpdateFlags;

use std::sync::Arc;

use super::MobEffectBehavior;
use crate::entity::LivingEntity;
use crate::world::World;

/// Weaving mob effect behavior.
pub struct WeavingBehavior;

impl MobEffectBehavior for WeavingBehavior {
    /// Vanilla `WeavingMobEffect.onMobRemoved`: on death the mob scatters
    /// cobwebs in a three-block radius, replacing only replaceable blocks and
    /// skipping a random half of the candidate positions.
    fn on_mob_removed(&self, world: &Arc<World>, mob: &dyn LivingEntity) {
        let origin = mob.block_position();
        let cobweb = vanilla_blocks::COBWEB.default_state();
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if rand::random::<bool>() {
                        continue;
                    }
                    let pos = origin.offset(dx, dy, dz);
                    if !world.get_block_state(pos).is_replaceable() {
                        continue;
                    }
                    world.set_block(pos, cobweb, UpdateFlags::UPDATE_ALL);
                }
            }
        }
    }
}
