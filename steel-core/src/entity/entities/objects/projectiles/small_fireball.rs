//! Small fireball projectile (`SmallFireball`).
//!
//! Vanilla `SmallFireball` flies straight, sets the entity it hits on fire and
//! deals `5.0` fireball damage. It never explodes.

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
use crate::world::World;

/// Vanilla `SmallFireball` damage (`5.0`).
pub const SMALL_FIREBALL_DAMAGE: f32 = 5.0;
/// Vanilla fire time applied to the entity that is hit (`5` seconds).
pub const SMALL_FIREBALL_FIRE_SECONDS: i32 = 5;

/// A small fireball.
#[entity_behavior(class = "SmallFireball")]
pub struct SmallFireballEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    projectile_base: ProjectileBase,
}

// SAFETY: This key is owned by Steel and uniquely identifies `SmallFireballEntity`.
unsafe impl DowncastType for SmallFireballEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/small_fireball");
}

impl SmallFireballEntity {
    /// Creates a new small fireball with no owner.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            projectile_base: ProjectileBase::new(),
        }
    }

    /// Creates a small fireball from saved base data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            projectile_base: ProjectileBase::new(),
        }
    }

    /// Vanilla `SmallFireball.damageSources().fireball(this, owner)`.
    fn fireball_damage_source(self: &Arc<Self>) -> DamageSource {
        let mut source = DamageSource::environment(&vanilla_damage_types::ON_FIRE)
            .with_direct_entity(Arc::clone(self) as SharedEntity);
        if let Some(owner) = self.get_owner() {
            source = source.with_causing_entity(owner);
        }
        source
    }
}

impl Entity for SmallFireballEntity {
    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn tick(self: Arc<Self>) {
        self.throwable_projectile_tick();
    }

    /// Small fireballs fly straight.
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

impl Projectile for SmallFireballEntity {
    fn projectile_base(&self) -> &ProjectileBase {
        &self.projectile_base
    }

    fn on_hit(self: Arc<Self>, hit: &ProjectileHit) {
        Arc::clone(&self).projectile_on_hit(hit);

        if let ProjectileHit::Entity(hit) = hit
            && let Some(world) = hit.entity.level()
        {
            let source = self.fireball_damage_source();
            hit.entity.hurt(&world, &source, SMALL_FIREBALL_DAMAGE);
            hit.entity.ignite_for_ticks(SMALL_FIREBALL_FIRE_SECONDS * 20);
        }
        self.set_removed(RemovalReason::Discarded);
    }
}

impl ThrowableProjectile for SmallFireballEntity {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_fireball_constants_match_vanilla() {
        assert_eq!(SMALL_FIREBALL_DAMAGE, 5.0);
        assert_eq!(SMALL_FIREBALL_FIRE_SECONDS, 5);
    }
}
