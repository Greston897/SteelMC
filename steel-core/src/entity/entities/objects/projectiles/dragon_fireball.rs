//! Dragon fireball projectile (`DragonFireball`).
//!
//! Vanilla `DragonFireball` flies straight and, instead of exploding, spawns an
//! `AreaEffectCloud` with the dragon-breath effect where it lands
//! (`DragonFireball.onHit` → `spawnAreaCloud`). The cloud is anchored to the
//! block position below the impact, has radius `3`, lasts `100` (wait time `0`)
//! ticks and applies `Instant Damage II` for `1` tick once per entity.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::data_components::components::potion_contents::PotionContents;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::mob_effect::instance::MobEffectInstance;
use steel_registry::particle_type::{ParticleData, PowerParticleOption};
use steel_registry::{vanilla_entities, vanilla_mob_effects, vanilla_particle_types};
use steel_utils::BlockPos;
use steel_utils::{DowncastType, DowncastTypeKey};

use crate::entity::entities::objects::effects::AreaEffectCloudEntity;
use crate::entity::next_entity_id;
use crate::entity::{
    Entity, EntityBase, EntityBaseLoad, Projectile, ProjectileBase, ProjectileHit, RemovalReason,
    SharedEntity, ThrowableProjectile,
};
use crate::world::World;

/// Vanilla `DragonFireball` breath cloud radius (`3.0`).
pub const DRAGON_FIREBALL_RADIUS: f32 = 3.0;

/// Ticks the breath cloud stays alive (vanilla `spawnAreaCloud` duration).
pub const DRAGON_BREATH_CLOUD_DURATION: i32 = 100;

/// `Instant Damage` duration applied by the cloud, in ticks.
pub const DRAGON_BREATH_EFFECT_DURATION: i32 = 1;

/// `Instant Damage` amplifier (`1` = level II, like vanilla).
pub const DRAGON_BREATH_EFFECT_AMPLIFIER: i32 = 1;

/// Vanilla `DragonFireball` breath particle power (`0.4`).
pub const DRAGON_BREATH_PARTICLE_POWER: f32 = 0.4;

/// A dragon fireball.
#[entity_behavior(class = "DragonFireball")]
pub struct DragonFireballEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    projectile_base: ProjectileBase,
}

// SAFETY: This key is owned by Steel and uniquely identifies `DragonFireballEntity`.
unsafe impl DowncastType for DragonFireballEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/dragon_fireball");
}

impl DragonFireballEntity {
    /// Creates a new dragon fireball with no owner.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            projectile_base: ProjectileBase::new(),
        }
    }

    /// Creates a dragon fireball from saved base data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            projectile_base: ProjectileBase::new(),
        }
    }

    /// Vanilla `DragonFireball.spawnAreaCloud`: spawns the breath cloud and
    /// discards the fireball. The block below the impact becomes the cloud
    /// anchor when it is a solid surface.
    fn spawn_breath_cloud(self: &Arc<Self>, world: &Arc<World>) {
        let position = self.position();
        let below = BlockPos::from(position - DVec3::new(0.0, 1.0, 0.0));
        let anchor = if world.get_block_state(below).is_solid() {
            position - DVec3::new(0.0, 1.0, 0.0)
        } else {
            position
        };

        let cloud = Arc::new(AreaEffectCloudEntity::new(
            &vanilla_entities::AREA_EFFECT_CLOUD,
            next_entity_id(),
            anchor,
            Arc::downgrade(world),
        ));
        cloud.set_owner(self.get_owner().as_ref());
        cloud.set_radius(DRAGON_FIREBALL_RADIUS);
        cloud.set_duration(DRAGON_BREATH_CLOUD_DURATION);
        cloud.set_wait_time(0);
        cloud.set_radius_per_tick(-DRAGON_FIREBALL_RADIUS / DRAGON_BREATH_CLOUD_DURATION as f32);
        cloud.set_particle(ParticleData::new(
            &vanilla_particle_types::DRAGON_BREATH,
            PowerParticleOption::new(DRAGON_BREATH_PARTICLE_POWER),
        ));
        cloud.set_potion_contents(PotionContents::new(
            None,
            None,
            vec![MobEffectInstance::new(
                &vanilla_mob_effects::INSTANT_DAMAGE,
                DRAGON_BREATH_EFFECT_DURATION,
                DRAGON_BREATH_EFFECT_AMPLIFIER,
                false,
                true,
                true,
            )],
            None,
        ));

        let cloud: SharedEntity = cloud;
        if let Err(error) = world.try_add_entity(cloud) {
            log::debug!("world rejected a dragon breath cloud: {error}");
        }
    }
}

impl Entity for DragonFireballEntity {
    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn tick(self: Arc<Self>) {
        self.throwable_projectile_tick();
    }

    /// Large fireballs fly straight.
    fn get_default_gravity(&self) -> f64 {
        0.0
    }

    fn sound_source(&self) -> SoundSource {
        SoundSource::Neutral
    }

    fn spawn_data(&self) -> i32 {
        self.get_owner().map_or(0, |owner| owner.id())
    }

    fn restore_owner_reference(&self, owner: &SharedEntity) {
        self.cache_owner_entity(owner);
    }

    fn projectile_owner_uuid(&self) -> Option<uuid::Uuid> {
        self.owner_uuid()
    }

    fn projectile_owner(&self) -> Option<SharedEntity> {
        self.get_owner()
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_projectile(nbt);
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_projectile(nbt);
    }
}

impl Projectile for DragonFireballEntity {
    fn projectile_base(&self) -> &ProjectileBase {
        &self.projectile_base
    }

    fn on_hit(self: Arc<Self>, hit: &ProjectileHit) {
        Arc::clone(&self).projectile_on_hit(hit);
        if let Some(world) = self.level() {
            self.spawn_breath_cloud(&world);
        }
        self.set_removed(RemovalReason::Discarded);
    }
}

impl ThrowableProjectile for DragonFireballEntity {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dragon_fireball_radius_matches_vanilla() {
        assert_eq!(DRAGON_FIREBALL_RADIUS, 3.0);
    }

    #[test]
    fn dragon_breath_cloud_matches_vanilla() {
        assert_eq!(DRAGON_BREATH_CLOUD_DURATION, 100);
        assert_eq!(DRAGON_BREATH_EFFECT_DURATION, 1);
        assert_eq!(DRAGON_BREATH_EFFECT_AMPLIFIER, 1);
        assert_eq!(DRAGON_BREATH_PARTICLE_POWER, 0.4);
    }
}
