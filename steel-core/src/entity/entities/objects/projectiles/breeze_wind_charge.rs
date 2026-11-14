//! Wind charge projectile entity (`BreezeWindCharge`).
//!
//! Vanilla `net.minecraft.world.entity.projectile.hurtingprojectile.windcharge.WindCharge`
//! flies in a straight line (no gravity) and, on the first block or entity it
//! touches, detonates a non-block-breaking explosion (`ExplosionInteraction.TRIGGER`,
//! radius 1.2) with `minecraft:wind_charge` as its damage type.
//!
//! Differences from vanilla, all deliberate:
//! * Steel's projectiles below `HurtingProjectile` use the
//!   `Projectile → ThrowableProjectile` stack, so this entity reuses the shared
//!   throwable tick (drag, hit raycast) with gravity disabled instead of
//!   reimplementing vanilla's hurting-projectile loop.
//! * The burst uses [`ExplosionInteraction::Trigger`], which leaves blocks in
//!   place and only dispatches `BLOCK_ACTIVATE` so interactables react.
//! * Vanilla applies an extra wind-charge-specific knockback multiplier; Steel
//!   uses the standard explosion knockback formula with radius 1.2.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::particle_type::ParticleData;
use steel_registry::sound_event::SoundEventHolder;
use steel_registry::{sound_events, vanilla_damage_types, vanilla_particle_types};
use steel_utils::{DowncastType, DowncastTypeKey};

use crate::entity::damage::DamageSource;
use crate::entity::{
    Entity, EntityBase, EntityBaseLoad, Projectile, ProjectileBase, ProjectileHit, RemovalReason,
    SharedEntity, ThrowableProjectile,
};
use crate::world::{ExplosionInteraction, ExplosionOptions, World};

/// Vanilla `WindCharge.BREEZE_WIND_CHARGE_POWER`.
pub const BREEZE_WIND_CHARGE_POWER: f32 = 1.2;

/// A breeze wind charge.
#[entity_behavior(class = "BreezeWindCharge")]
pub struct BreezeWindChargeEntity {
    /// Common entity fields (id, uuid, position, etc.).
    base: EntityBase,
    /// Vanilla entity type registered for this implementation.
    entity_type: EntityTypeRef,
    /// Shared `Projectile` state (owner / left-owner / has-been-shot).
    projectile_base: ProjectileBase,
}

// SAFETY: This key is owned by Steel and uniquely identifies `BreezeWindChargeEntity`.
unsafe impl DowncastType for BreezeWindChargeEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/breeze_wind_charge");
}

impl BreezeWindChargeEntity {
    /// Creates a breeze wind charge with no owner.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            projectile_base: ProjectileBase::new(),
        }
    }

    /// Creates a breeze wind charge from saved base data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            projectile_base: ProjectileBase::new(),
        }
    }

    /// Vanilla `WindCharge.damageSources().windCharge(this, owner)`.
    fn breeze_damage_source(self: &Arc<Self>) -> DamageSource {
        let mut source = DamageSource::environment(&vanilla_damage_types::WIND_CHARGE)
            .with_direct_entity(Arc::clone(self) as SharedEntity);
        if let Some(owner) = self.get_owner() {
            source = source.with_causing_entity(owner);
        }
        source
    }

    /// Vanilla `WindCharge.explode`: a trigger-only explosion plus the gust
    /// particles and burst sound.
    fn explode(self: &Arc<Self>, world: &Arc<World>) {
        let mut options = ExplosionOptions::new(
            self.position(),
            BREEZE_WIND_CHARGE_POWER,
            ExplosionInteraction::Trigger,
        );
        options.source = Some(Arc::clone(self) as SharedEntity);
        options.damage_source = Some(self.breeze_damage_source());
        options.small_explosion_particle =
            ParticleData::simple(&vanilla_particle_types::GUST_EMITTER_SMALL);
        options.large_explosion_particle =
            ParticleData::simple(&vanilla_particle_types::GUST_EMITTER_LARGE);
        options.explosion_sound =
            SoundEventHolder::registry(&sound_events::ENTITY_WIND_CHARGE_WIND_BURST);
        world.explode(options);
    }
}

impl Entity for BreezeWindChargeEntity {
    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn tick(self: Arc<Self>) {
        self.throwable_projectile_tick();
    }

    /// Vanilla wind charges are not affected by gravity; they fly straight.
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

impl Projectile for BreezeWindChargeEntity {
    fn projectile_base(&self) -> &ProjectileBase {
        &self.projectile_base
    }

    fn on_hit(self: Arc<Self>, hit: &ProjectileHit) {
        // Vanilla `AbstractWindCharge.onHit`: resolve the shared projectile hit
        // bookkeeping first (deflection, game event, block-side effects), then
        // detonate on the server before discarding the charge.
        Arc::clone(&self).projectile_on_hit(hit);

        if let Some(world) = self.level() {
            self.explode(&world);
        }
        self.set_removed(RemovalReason::Discarded);
    }
}

impl ThrowableProjectile for BreezeWindChargeEntity {}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_entities};
    use steel_utils::{BlockPos, Direction};

    use super::*;
    use crate::world::ClipHitResult;

    fn charge() -> Arc<BreezeWindChargeEntity> {
        init_vanilla_registry();
        Arc::new(BreezeWindChargeEntity::new(
            &vanilla_entities::BREEZE_WIND_CHARGE,
            1,
            DVec3::ZERO,
            Weak::<World>::new(),
        ))
    }

    #[test]
    fn explosion_power_matches_vanilla() {
        assert_eq!(BREEZE_WIND_CHARGE_POWER, 1.2);
    }

    #[test]
    fn wind_charges_fly_without_gravity() {
        assert_eq!(Entity::get_default_gravity(charge().as_ref()), 0.0);
    }

    #[test]
    fn breeze_charge_hitting_a_block_discards_it() {
        init_vanilla_registry();
        let charge = charge();
        let hit = ProjectileHit::Block {
            location: DVec3::ZERO,
            hit: ClipHitResult {
                location: DVec3::ZERO,
                direction: Direction::Up,
                block_pos: BlockPos::new(0, 0, 0),
                miss: false,
                inside: false,
                world_border_hit: false,
            },
        };

        Arc::clone(&charge).on_hit(&hit);

        assert!(charge.is_removed());
    }
}
