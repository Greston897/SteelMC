//! `InfestedMobEffect` behavior.

use std::sync::Arc;

use steel_registry::vanilla_entities;

use super::MobEffectBehavior;
use crate::entity::{
    EntitySpawnPlacement, EntitySpawnReason, EntitySpawnRequest, LivingEntity, spawn_entity,
};
use crate::world::World;

/// Chance per hit, mirroring vanilla `InfestedMobEffect.CHANCE_TO_SPAWN`.
const CHANCE_TO_SPAWN: f32 = 0.1;

/// Infested mob effect behavior.
pub struct InfestedBehavior;

impl MobEffectBehavior for InfestedBehavior {
    /// Vanilla `InfestedMobEffect.onMobHurt`: each hit has a 10% chance to spill
    /// one to two silverfish around the mob.
    fn on_mob_hurt(&self, world: &Arc<World>, mob: &dyn LivingEntity, amplifier: i32) {
        // Vanilla rolls once per hit regardless of amplifier; the level scales
        // how many silverfish can appear (`level + 1`).
        if rand::random::<f32>() >= CHANCE_TO_SPAWN {
            return;
        }
        let pos = mob.block_position();
        let max = (amplifier + 1).min(3);
        let count = rand::random_range(1..=max.max(1));
        for _ in 0..count {
            let request = EntitySpawnRequest {
                entity_type: &vanilla_entities::SILVERFISH,
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
            let _ = spawn_entity(world, request);
        }
    }
}
