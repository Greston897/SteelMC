//! Wither skull projectile (`WitherSkull`).
//!
//! Vanilla `WitherSkull` deals `8.0` wither damage on contact and can apply the
//! wither effect. The explosion on block impact is not implemented.

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

/// Vanilla `WitherSkull` impact damage.
pub const WITHER_SKULL_DAMAGE: f32 = 8.0;

/// A WitherSkull projectile.
#[entity_behavior(class = "WitherSkull")]
pub struct WitherSkullEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    projectile_base: ProjectileBase,
}

// SAFETY: This key is owned by Steel and uniquely identifies `WitherSkullEntity`.
unsafe impl DowncastType for WitherSkullEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/wither_skull");
}

impl WitherSkullEntity {
    /// Creates a new wither_skull with no owner.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            projectile_base: ProjectileBase::new(),
        }
    }

    /// Creates a wither_skull from saved base data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            projectile_base: ProjectileBase::new(),
        }
    }

    /// Vanilla `WitherSkull` damage source.
    fn impact_damage_source(self: &Arc<Self>) -> DamageSource {
        let mut source = DamageSource::environment(&vanilla_damage_types::WITHER)
            .with_direct_entity(Arc::clone(self) as SharedEntity);
        if let Some(owner) = self.get_owner() {
            source = source.with_causing_entity(owner);
        }
        source
    }
}

impl Entity for WitherSkullEntity {
    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn tick(self: Arc<Self>) {
        self.throwable_projectile_tick();
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

impl Projectile for WitherSkullEntity {
    fn projectile_base(&self) -> &ProjectileBase {
        &self.projectile_base
    }

    fn on_hit(self: Arc<Self>, hit: &ProjectileHit) {
        Arc::clone(&self).projectile_on_hit(hit);

        if let ProjectileHit::Entity(hit) = hit {
            let entity = hit.entity.as_ref();
            if let Some(world) = entity.level() {
                let source = self.impact_damage_source();
                entity.hurt(&world, &source, WITHER_SKULL_DAMAGE);
            }
        }
        self.set_removed(RemovalReason::Discarded);
    }
}

impl ThrowableProjectile for WitherSkullEntity {}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_entities};

    use super::*;

    #[test]
    fn wither_skull_damage_matches_vanilla() {
        assert_eq!(WITHER_SKULL_DAMAGE, 1.0);
    }

    #[test]
    fn new_wither_skull_has_no_owner() {
        init_vanilla_registry();
        let projectile = WitherSkullEntity::new(
            &vanilla_entities::WITHER_SKULL,
            1,
            DVec3::ZERO,
            Weak::<World>::new(),
        );
        assert!(projectile.get_owner().is_none());
        assert!(!projectile.is_removed());
    }
}
