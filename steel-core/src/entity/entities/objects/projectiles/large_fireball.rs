//! Large fireball projectile (`LargeFireball`).
//!
//! Vanilla `LargeFireball` flies straight (no gravity) and detonates on the
//! first block or entity it touches with a radius-1 explosion that can break
//! blocks, dealing `6.0` fireball damage in the blast.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::vanilla_damage_types;
use steel_utils::{DowncastType, DowncastTypeKey};

use crate::entity::damage::DamageSource;
use crate::entity::{
    Entity, EntityBase, EntityBaseLoad, Projectile, ProjectileBase, ProjectileHit, RemovalReason,
    SharedEntity, ThrowableProjectile,
};
use crate::world::{ExplosionInteraction, ExplosionOptions, World};

/// Vanilla `LargeFireball` explosion radius.
pub const LARGEFIREBALL_EXPLOSION_POWER: f32 = 1.0;

/// A large fireball.
#[entity_behavior(class = "LargeFireball")]
pub struct LargeFireballEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    projectile_base: ProjectileBase,
}

// SAFETY: This key is owned by Steel and uniquely identifies `LargeFireballEntity`.
unsafe impl DowncastType for LargeFireballEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/large_fireball");
}

impl LargeFireballEntity {
    /// Creates a new large fireball with no owner.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            projectile_base: ProjectileBase::new(),
        }
    }

    /// Creates a large fireball from saved base data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            projectile_base: ProjectileBase::new(),
        }
    }

    /// Vanilla `LargeFireball.damageSources().fireball(this, owner)`.
    fn fireball_damage_source(self: &Arc<Self>) -> DamageSource {
        let mut source = DamageSource::environment(&vanilla_damage_types::EXPLOSION)
            .with_direct_entity(Arc::clone(self) as SharedEntity);
        if let Some(owner) = self.get_owner() {
            source = source.with_causing_entity(owner);
        }
        source
    }

    /// Vanilla `LargeFireball.onHit` explosion.
    fn explode(self: &Arc<Self>, world: &Arc<World>) {
        let mut options = ExplosionOptions::new(
            self.position(),
            LARGEFIREBALL_EXPLOSION_POWER,
            ExplosionInteraction::Tnt,
        );
        options.source = Some(Arc::clone(self) as SharedEntity);
        options.damage_source = Some(self.fireball_damage_source());
        world.explode(options);
    }
}

impl Entity for LargeFireballEntity {
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

impl Projectile for LargeFireballEntity {
    fn projectile_base(&self) -> &ProjectileBase {
        &self.projectile_base
    }

    fn on_hit(self: Arc<Self>, hit: &ProjectileHit) {
        Arc::clone(&self).projectile_on_hit(hit);
        if let Some(world) = self.level() {
            self.explode(&world);
        }
        self.set_removed(RemovalReason::Discarded);
    }
}

impl ThrowableProjectile for LargeFireballEntity {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn large_fireball_power_matches_vanilla() {
        assert_eq!(LARGEFIREBALL_EXPLOSION_POWER, 1.0);
    }
}
