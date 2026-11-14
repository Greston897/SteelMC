//! `WindChargedMobEffect` behavior.

use std::sync::Arc;

use steel_registry::{sound_events, vanilla_damage_types, vanilla_particle_types};
use steel_registry::particle_type::ParticleData;
use steel_registry::sound_event::SoundEventHolder;

use super::MobEffectBehavior;
use crate::entity::LivingEntity;
use crate::entity::damage::DamageSource;
use crate::world::{ExplosionInteraction, ExplosionOptions, World};

/// Vanilla `WindCharge.WIND_CHARGE_POWER`.
const WIND_CHARGE_POWER: f32 = 1.2;

/// Wind charged mob effect behavior.
pub struct WindChargedBehavior;

impl MobEffectBehavior for WindChargedBehavior {
    /// Vanilla `WindChargedMobEffect.onMobRemoved`: the dying mob bursts into a
    /// blockless wind-charge explosion at its position.
    fn on_mob_removed(&self, world: &Arc<World>, mob: &dyn LivingEntity) {
        let mut options = ExplosionOptions::new(
            mob.position(),
            WIND_CHARGE_POWER,
            ExplosionInteraction::Trigger,
        );
        options.damage_source = Some(DamageSource::environment(
            &vanilla_damage_types::WIND_CHARGE,
        ));
        options.small_explosion_particle =
            ParticleData::simple(&vanilla_particle_types::GUST_EMITTER_SMALL);
        options.large_explosion_particle =
            ParticleData::simple(&vanilla_particle_types::GUST_EMITTER_LARGE);
        options.explosion_sound =
            SoundEventHolder::registry(&sound_events::ENTITY_WIND_CHARGE_WIND_BURST);
        world.explode(options);
    }
}
