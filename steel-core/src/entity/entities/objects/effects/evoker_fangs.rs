//! Vanilla `EvokerFangs`.
//!
//! A short-lived spike summoned by evokers. It lives for `22` ticks, deals
//! damage once at tick `20` (vanilla `ATTACK_TICK`) to every living entity in a
//! small box, and is then discarded.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::vanilla_entity_data::BaseEntityData;
use steel_registry::{sound_events, vanilla_damage_types};
use steel_utils::locks::SyncMutex;
use steel_utils::{DowncastType, DowncastTypeKey, WorldAabb};

use crate::entity::damage::DamageSource;
use crate::entity::{Entity, EntityBase, EntityBaseLoad, EntitySyncedData, RemovalReason};
use crate::world::World;

/// Vanilla `EvokerFangs.LIFE_TIME`.
const LIFE_TIME: i32 = 22;
/// Vanilla `EvokerFangs.ATTACK_TICK`.
const ATTACK_TICK: i32 = 20;
/// Vanilla fangs damage.
const DAMAGE: f32 = 6.0;
/// Half-width of the damage box.
const DAMAGE_BOX: f64 = 1.0;
/// Half-height of the damage box.
const DAMAGE_HEIGHT: f64 = 0.4;

/// Vanilla `net.minecraft.world.entity.projectile.EvokerFangs`.
#[entity_behavior(class = "EvokerFangs")]
pub struct EvokerFangsEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    entity_data: SyncMutex<BaseEntityData>,
}

impl EvokerFangsEntity {
    /// Vanilla fangs dimensions are fixed regardless of pose.
    /// Creates a new evoker fangs entity.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            entity_data: SyncMutex::new(BaseEntityData::new()),
        }
    }

    /// Creates an evoker fangs entity from saved data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            entity_data: SyncMutex::new(BaseEntityData::new()),
        }
    }

    /// Vanilla `EvokerFangs.dealDamageTo`: hurt everything living inside the box.
    fn deal_damage(self: &Arc<Self>) {
        let Some(world) = self.level() else {
            return;
        };
        let center = self.position() + DVec3::new(0.0, DAMAGE_HEIGHT, 0.0);
        let origin = center - DVec3::new(DAMAGE_BOX, DAMAGE_HEIGHT, DAMAGE_BOX);
        let box_ = WorldAabb::new(
            origin.x,
            origin.y,
            origin.z,
            origin.x + DAMAGE_BOX * 2.0,
            origin.y + DAMAGE_HEIGHT * 2.0,
            origin.z + DAMAGE_BOX * 2.0,
        );

        let source = DamageSource::environment(&vanilla_damage_types::MAGIC);
        for entity in world.get_entities_in_aabb(&box_) {
            if entity.id() == self.id() || !entity.is_alive() {
                continue;
            }
            entity.hurt(&world, &source, DAMAGE);
        }
        self.play_sound(&sound_events::ENTITY_EVOKER_FANGS_ATTACK, 1.0, 1.0);
    }
}

// SAFETY: This key is owned by Steel and uniquely identifies `EvokerFangsEntity`.
unsafe impl DowncastType for EvokerFangsEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/evoker_fangs");
}

impl Entity for EvokerFangsEntity {
    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn synced_data(&self) -> Option<&dyn EntitySyncedData> {
        Some(&self.entity_data)
    }

    fn tick(self: Arc<Self>) {
        let age = self.tick_count();
        if age >= LIFE_TIME + 1 {
            self.set_removed(RemovalReason::Discarded);
            return;
        }
        if age == ATTACK_TICK {
            self.deal_damage();
        }
    }

    fn save_additional(&self, _nbt: &mut NbtCompound) {}

    fn load_additional(&self, _nbt: BorrowedNbtCompoundView<'_, '_>) {}
}

#[cfg(test)]
mod tests {
    use steel_registry::init_vanilla_registry;
    use steel_registry::vanilla_entities;

    use super::*;

    fn fangs() -> EvokerFangsEntity {
        EvokerFangsEntity::new(
            &vanilla_entities::EVOKER_FANGS,
            1,
            DVec3::new(0.5, 4.0, 0.5),
            Weak::new(),
        )
    }

    #[test]
    fn evoker_fangs_use_fixed_dimensions() {
        init_vanilla_registry();
        let fangs = fangs();
        let box_ = fangs.bounding_box();
        assert!(box_.max.x > box_.min.x);
    }

    #[test]
    fn evoker_fangs_have_no_synced_payload_but_keep_base_data() {
        init_vanilla_registry();
        let fangs = fangs();
        assert!(fangs.synced_data().is_some());
    }

    #[test]
    fn vanilla_fangs_timings_match() {
        assert_eq!(LIFE_TIME, 22);
        assert_eq!(ATTACK_TICK, 20);
        assert_eq!(DAMAGE, 6.0);
    }
}
