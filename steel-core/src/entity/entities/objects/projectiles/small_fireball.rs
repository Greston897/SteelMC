//! Small fireball projectile (`SmallFireball`).
//!
//! Vanilla `SmallFireball` flies straight, sets the entity it hits on fire and
//! deals `5.0` fireball damage. It never explodes.
//!
//! The vanilla behaviour is split across three overrides:
//!
//! * `onHitEntity` saves the victim's remaining fire ticks, ignites it for
//!   five seconds, applies `5.0` fireball damage and restores the saved fire
//!   ticks when the damage did not land. On a successful hit it runs the
//!   post-attack enchantment effects.
//! * `onHitBlock` lights the block next to the impact, unless the owner is a
//!   mob and `mobGriefing` is disabled.
//! * `onHit` discards the projectile on the server.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::item_stack::ItemStack;
use steel_registry::items::ItemRef;
use steel_registry::vanilla_damage_types;
use steel_registry::vanilla_entity_data::SmallFireballEntityData;
use steel_registry::vanilla_game_rules;
use steel_registry::vanilla_items;
use steel_utils::locks::SyncMutex;
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, Direction, DowncastType, DowncastTypeKey};

use crate::behavior::blocks::FireBlock;
use crate::entity::damage::DamageSource;
use crate::entity::{
    Entity, EntityBase, EntityBaseLoad, EntitySyncedData, Projectile, ProjectileBase, ProjectileHit,
    RemovalReason, SharedEntity, ThrowableItemProjectile, ThrowableProjectile,
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
    /// Synced data carrying the vanilla `Fireball.DATA_ITEM_STACK` item.
    entity_data: SyncMutex<SmallFireballEntityData>,
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
            entity_data: SyncMutex::new(SmallFireballEntityData::new()),
            projectile_base: ProjectileBase::new(),
        }
    }

    /// Creates a small fireball from saved base data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            entity_data: SyncMutex::new(SmallFireballEntityData::new()),
            projectile_base: ProjectileBase::new(),
        }
    }

    /// Vanilla `SmallFireball.damageSources().fireball(this, owner)`.
    ///
    /// Vanilla uses `DamageTypes.FIREBALL` when the projectile has an owner
    /// and `DamageTypes.UNATTRIBUTED_FIREBALL` when it does not.
    fn fireball_damage_source(self: &Arc<Self>) -> DamageSource {
        let fireball = Arc::clone(self) as SharedEntity;
        match self.get_owner() {
            Some(owner) => DamageSource::direct(&vanilla_damage_types::FIREBALL, fireball)
                .with_causing_entity(owner),
            None => DamageSource::direct(&vanilla_damage_types::UNATTRIBUTED_FIREBALL, fireball),
        }
    }

    /// Vanilla `SmallFireball.onHitBlock`: light the block next to the impact.
    ///
    /// Mob-owned fireballs skip the placement when `mobGriefing` is disabled.
    fn light_block_on_hit(&self, world: &Arc<World>, block_pos: BlockPos, direction: Direction) {
        let owner = self.get_owner();
        let owner_is_mob = owner
            .as_ref()
            .is_some_and(|owner| owner.as_ref().is_mob());
        let mob_griefing = world.get_game_rule(&vanilla_game_rules::MOB_GRIEFING);
        if owner_is_mob && !mob_griefing {
            return;
        }
        let target = block_pos.relative(direction);
        if !world.get_block_state(target).is_air() {
            return;
        }
        world.set_block(
            target,
            FireBlock::get_state(world.as_ref(), target),
            UpdateFlags::UPDATE_ALL,
        );
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
        self.save_throwable_item(nbt);
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_projectile(nbt);
        self.load_throwable_item(nbt);
    }

    fn synced_data(&self) -> Option<&dyn EntitySyncedData> {
        Some(&self.entity_data)
    }
}

impl Projectile for SmallFireballEntity {
    fn projectile_base(&self) -> &ProjectileBase {
        &self.projectile_base
    }

    fn on_hit(self: Arc<Self>, hit: &ProjectileHit) {
        Arc::clone(&self).projectile_on_hit(hit);

        match hit {
            ProjectileHit::Entity(hit) => {
                let Some(world) = hit.entity.level() else {
                    return;
                };
                let source = self.fireball_damage_source();
                // Vanilla saves the fire ticks before igniting and restores them
                // when the damage does not land.
                let remaining_fire_ticks = hit.entity.remaining_fire_ticks();
                hit.entity
                    .ignite_for_ticks(SMALL_FIREBALL_FIRE_SECONDS * 20);
                if !hit.entity.hurt(&world, &source, SMALL_FIREBALL_DAMAGE) {
                    hit.entity.set_remaining_fire_ticks(remaining_fire_ticks);
                } else {
                    let context = crate::enchantment_helper::EnchantmentPostAttackContext::new(
                        &hit.entity,
                        &source,
                    );
                    crate::enchantment_helper::do_post_attack_effects_with_item_source(
                        &world,
                        hit.entity.as_ref(),
                        &steel_registry::item_stack::ItemStack::empty(),
                        &context,
                    );
                }
            }
            ProjectileHit::Block { hit, .. } => {
                if let Some(world) = self.level() {
                    self.light_block_on_hit(&world, hit.block_pos, hit.direction);
                }
            }
        }

        self.set_removed(RemovalReason::Discarded);
    }
}

impl ThrowableProjectile for SmallFireballEntity {}

impl ThrowableItemProjectile for SmallFireballEntity {
    /// Vanilla `Fireball.getDefaultItem`.
    fn get_default_item(&self) -> ItemRef {
        &vanilla_items::FIRE_CHARGE
    }

    /// Vanilla `Fireball.setItem`.
    fn set_item(&self, item: ItemStack) {
        self.entity_data.lock().fireball_mut().item_stack.set(item);
    }

    /// Vanilla `Fireball.getItem`.
    fn get_item(&self) -> ItemStack {
        self.entity_data.lock().fireball().item_stack.get().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_fireball_constants_match_vanilla() {
        assert_eq!(SMALL_FIREBALL_DAMAGE, 5.0);
        assert_eq!(SMALL_FIREBALL_FIRE_SECONDS, 5);
    }
}
