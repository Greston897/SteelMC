//! Wither skull projectile (`WitherSkull`).
//!
//! Vanilla `WitherSkull`: `8.0` impact damage plus wither II on an entity hit,
//! and a blast of power `1.0` when it hits a block, which is the same power as a
//! ghast fireball and enough to break any block below blast resistance 4.
//!
//! Charged skulls synchronise their blue variant and cap destructible block
//! resistance; both block and entity impacts explode.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::vanilla_damage_types;
use steel_registry::vanilla_mob_effects;
use steel_registry::vanilla_entity_data::WitherSkullEntityData;
use steel_utils::locks::SyncMutex;
use steel_utils::nbt::NbtValueInput as _;
use steel_utils::{DowncastType, DowncastTypeKey};

use crate::entity::damage::DamageSource;
use crate::entity::{
    Entity, EntityBase, EntityBaseLoad, EntitySyncedData, MobEffectInstance, Projectile, ProjectileBase,
    ProjectileHit, RemovalReason, SharedEntity, ThrowableProjectile,
};
use crate::world::World;
use crate::world::explosion::{ExplosionInteraction, ExplosionOptions};

/// Vanilla `WitherSkull` impact damage.
pub const WITHER_SKULL_DAMAGE: f32 = 8.0;

/// Vanilla black wither skull blast power, the same as a ghast fireball.
pub const WITHER_SKULL_EXPLOSION_POWER: f32 = 1.0;

/// Vanilla `WitherSkull` wither duration on Normal, in ticks (10 seconds).
pub const WITHER_SKULL_WITHER_TICKS: i32 = 200;

/// Vanilla `WitherSkull` wither amplifier: wither II.
pub const WITHER_SKULL_WITHER_AMPLIFIER: i32 = 1;

/// A `WitherSkull` projectile.
#[entity_behavior(class = "WitherSkull")]
pub struct WitherSkullEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    projectile_base: ProjectileBase,
    entity_data: SyncMutex<WitherSkullEntityData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `WitherSkullEntity`.
unsafe impl DowncastType for WitherSkullEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/wither_skull");
}

impl WitherSkullEntity {
    /// Creates a new `wither_skull` with no owner.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            projectile_base: ProjectileBase::new(),
            entity_data: SyncMutex::new(WitherSkullEntityData::new()),
        }
    }

    /// Creates a `wither_skull` from saved base data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            projectile_base: ProjectileBase::new(),
            entity_data: SyncMutex::new(WitherSkullEntityData::new()),
        }
    }

    /// Blue skull variant, carried by vanilla DATA_DANGEROUS and NBT.
    pub fn set_dangerous(&self, dangerous: bool) {
        self.entity_data.lock().dangerous.set(dangerous);
    }

    /// Whether this skull caps destructible block resistance at 0.8.
    pub fn is_dangerous(&self) -> bool {
        *self.entity_data.lock().dangerous.get()
    }

    /// Vanilla `WitherSkull` damage source.
    fn impact_damage_source(self: &Arc<Self>) -> DamageSource {
        let mut source = DamageSource::environment(&vanilla_damage_types::WITHER_SKULL)
            .with_direct_entity(Arc::clone(self) as SharedEntity);
        if let Some(owner) = self.get_owner() {
            source = source.with_causing_entity(owner);
        }
        source
    }
    /// Vanilla `WitherSkull.onHit` / `onHitBlock` explosion.
    ///
    /// `ExplosionInteraction::Mob` is what vanilla passes for a mob-owned blast,
    /// so `mobGriefing` and the mob drop decay apply.
    fn explode(self: &Arc<Self>, world: &Arc<World>) {
        let mut options = ExplosionOptions::new(
            self.position(),
            WITHER_SKULL_EXPLOSION_POWER,
            ExplosionInteraction::Mob,
        );
        options.source = Some(Arc::clone(self) as SharedEntity);
        options.damage_source = Some(self.impact_damage_source());
        if self.is_dangerous() { options.damage_calculator = Some(&CHARGED_SKULL_CALCULATOR); }
        world.explode(options);
    }
}

impl Entity for WitherSkullEntity {
    fn synced_data(&self) -> Option<&dyn EntitySyncedData> { Some(&self.entity_data) }

    fn get_default_gravity(&self) -> f64 { 0.0 }

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
        nbt.insert("dangerous", i8::from(self.is_dangerous()));
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_projectile(nbt);
        self.set_dangerous(nbt.byte_value("dangerous").is_some_and(|value| value != 0));
    }
}

impl Projectile for WitherSkullEntity {
    fn projectile_base(&self) -> &ProjectileBase {
        &self.projectile_base
    }

    fn on_hit(self: Arc<Self>, hit: &ProjectileHit) {
        Arc::clone(&self).projectile_on_hit(hit);

        match hit {
            ProjectileHit::Entity(hit) => {
                let entity = hit.entity.as_ref();
                if let Some(world) = entity.level() {
                    let owner = self.get_owner();
                    let living_owner = owner.as_ref().and_then(|owner| owner.as_living_entity());
                    let source = if living_owner.is_some() { self.impact_damage_source() }
                        else { DamageSource::environment(&vanilla_damage_types::MAGIC) };
                    let was_hurt = entity.hurt(&world, &source, if living_owner.is_some() { WITHER_SKULL_DAMAGE } else { 5.0 });
                    if was_hurt {
                        if !entity.is_alive() && let Some(owner) = living_owner { owner.heal(5.0); }
                        let ticks = match world.difficulty() {
                            steel_utils::types::Difficulty::Normal => WITHER_SKULL_WITHER_TICKS,
                            steel_utils::types::Difficulty::Hard => 800,
                            _ => 0,
                        };
                        if ticks > 0 && let Some(living) = entity.as_living_entity() {
                            living.add_mob_effect(MobEffectInstance::simple(vanilla_mob_effects::WITHER, ticks, WITHER_SKULL_WITHER_AMPLIFIER));
                        }
                    }
                }
            }
            // Vanilla `WitherSkull.onHitBlock` explodes, which is why a shot
            // that misses every entity still leaves a crater.
            ProjectileHit::Block { .. } => {}
        }
        if let Some(world) = self.level() { self.explode(&world); }
        self.set_removed(RemovalReason::Discarded);
    }
}

impl ThrowableProjectile for WitherSkullEntity {
    fn apply_inertia(&self) {
        let movement = self.velocity();
        let inertia = if self.is_in_water() { 0.8 } else if self.is_dangerous() { 0.73 } else { 0.95 };
        self.set_velocity((movement + movement.normalize_or_zero() * 0.1) * inertia);
    }
}

struct ChargedSkullCalculator;
static CHARGED_SKULL_CALCULATOR: ChargedSkullCalculator = ChargedSkullCalculator;

impl crate::world::explosion::ExplosionDamageCalculator for ChargedSkullCalculator {
    fn block_explosion_resistance(&self, explosion: &dyn crate::world::Explosion, world: &World,
        pos: steel_utils::BlockPos, state: steel_utils::BlockStateId, fluid: steel_registry::fluid::FluidState) -> Option<f32> {
        use steel_registry::{TaggedRegistryExt as _, blocks::block_state_ext::BlockStateExt as _};
        use crate::world::explosion::DefaultExplosionDamageCalculator;
        let resistance = DefaultExplosionDamageCalculator.block_explosion_resistance(explosion, world, pos, state, fluid);
        if !state.is_air() && !steel_registry::REGISTRY.blocks.is_in_tag(state.get_block(), &steel_registry::vanilla_block_tags::BlockTag::WITHER_IMMUNE) {
            resistance.map(|value| value.min(0.8))
        } else { resistance }
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_entities};

    use super::*;

    #[test]
    fn wither_skull_damage_matches_vanilla() {
        // 8 on Normal; the test used to compare the damage against the blast
        // power and so asserted 1.0 against a constant of 8.0.
        assert_eq!(WITHER_SKULL_DAMAGE, 8.0);
    }

    #[test]
    fn wither_skull_wither_effect_matches_vanilla() {
        // Wither II for 10 seconds on Normal.
        assert_eq!(WITHER_SKULL_WITHER_TICKS, 10 * 20);
        assert_eq!(WITHER_SKULL_WITHER_AMPLIFIER, 1);
    }

    #[test]
    fn wither_skull_blast_power_matches_vanilla() {
        // Same power as a ghast fireball, and what the entry docs describe.
        assert_eq!(WITHER_SKULL_EXPLOSION_POWER, 1.0);
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
