//! Llama spit projectile (`LlamaSpit`).

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

/// Vanilla `LlamaSpit` damage (`1.0`).
pub const LLAMA_SPIT_DAMAGE: f32 = 1.0;

/// A llama spit projectile.
#[entity_behavior(class = "LlamaSpit")]
pub struct LlamaSpitEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    projectile_base: ProjectileBase,
}

// SAFETY: This key is owned by Steel and uniquely identifies `LlamaSpitEntity`.
unsafe impl DowncastType for LlamaSpitEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/llama_spit");
}

impl LlamaSpitEntity {
    /// Creates a new llama spit with no owner.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            projectile_base: ProjectileBase::new(),
        }
    }

    /// Creates a llama spit from saved base data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            projectile_base: ProjectileBase::new(),
        }
    }

    /// Vanilla `LlamaSpit.damageSources().spit(this, owner)`.
    fn spit_damage_source(self: &Arc<Self>) -> DamageSource {
        let mut source = DamageSource::environment(&vanilla_damage_types::THROWN)
            .with_direct_entity(Arc::clone(self) as SharedEntity);
        if let Some(owner) = self.get_owner() {
            source = source.with_causing_entity(owner);
        }
        source
    }
}

impl Entity for LlamaSpitEntity {
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

impl Projectile for LlamaSpitEntity {
    fn projectile_base(&self) -> &ProjectileBase {
        &self.projectile_base
    }

    fn on_hit(self: Arc<Self>, hit: &ProjectileHit) {
        Arc::clone(&self).projectile_on_hit(hit);

        if let ProjectileHit::Entity(hit) = hit {
            let entity = hit.entity.as_ref();
            if let Some(world) = entity.level() {
                let source = self.spit_damage_source();
                entity.hurt(&world, &source, LLAMA_SPIT_DAMAGE);
            }
        }
        self.set_removed(RemovalReason::Discarded);
    }
}

impl ThrowableProjectile for LlamaSpitEntity {}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_entities};

    use super::*;

    #[test]
    fn llama_spit_damage_matches_vanilla() {
        assert_eq!(LLAMA_SPIT_DAMAGE, 1.0);
    }

    #[test]
    fn new_llama_spit_has_no_owner() {
        init_vanilla_registry();
        let spit = LlamaSpitEntity::new(
            &vanilla_entities::LLAMA_SPIT,
            1,
            DVec3::ZERO,
            Weak::<World>::new(),
        );
        assert!(spit.get_owner().is_none());
        assert!(!spit.is_removed());
    }
}
