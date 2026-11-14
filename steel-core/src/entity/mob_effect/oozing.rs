//! `OozingMobEffect` behavior.

use std::sync::Arc;

use steel_registry::vanilla_entities;
use steel_utils::Downcast as _;

use super::MobEffectBehavior;
use crate::entity::entities::mobs::hostile::slime::SlimeEntity;
use crate::entity::{
    EntitySpawnPlacement, EntitySpawnReason, EntitySpawnRequest, LivingEntity, spawn_entity,
};
use crate::world::World;

/// Oozing mob effect behavior.
pub struct OozingBehavior;

impl MobEffectBehavior for OozingBehavior {
    /// Vanilla `OozingMobEffect.onMobRemoved`: on death the mob leaves two to
    /// four size-2 slimes behind.
    fn on_mob_removed(&self, world: &Arc<World>, mob: &dyn LivingEntity) {
        let pos = mob.block_position();
        let count = rand::random_range(2..=4);
        for _ in 0..count {
            let request = EntitySpawnRequest {
                entity_type: &vanilla_entities::SLIME,
                placement: EntitySpawnPlacement::Block {
                    pos,
                    try_move_down: false,
                    moved_up: false,
                },
                reason: EntitySpawnReason::Conversion,
                finalize_spawn: true,
                play_ambient_sound: false,
                item_stack: None,
                user_is_operator: false,
            };
            if let Ok(entity) = spawn_entity(world, request)
                && let Some(slime) = entity.downcast_ref::<SlimeEntity>()
            {
                slime.set_size(2, true);
            }
        }
    }
}
