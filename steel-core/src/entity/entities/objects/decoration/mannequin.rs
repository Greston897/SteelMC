//! Vanilla `net.minecraft.world.entity.decoration.Mannequin`.
//!
//! A mannequin is the humanoid decoration entity introduced in 26.3: it shares
//! the `Avatar` synced layer with `Player` (main hand, skin customisation mask)
//! and adds its own layer with the player profile, the `immovable` flag and an
//! optional description component rendered in the UI.
//!
//! Documented deviations from vanilla 26.3 (see `PARITY-26.3-RU.md`):
//! * mannequins do not render and cannot be equipped through interaction yet;
//!   the equipment slots live in `LivingEntityBase` and are saved with the
//!   entity, but the interaction path is not implemented.
//! * a mannequin always keeps the default `HumanoidArm::Right` and the default
//!   profile; NBT round-trips of `profile` are not implemented because
//!   `ResolvableProfile` has no NBT codec in Steel yet.
//! * mannequins never take damage in vanilla; Steel keeps the base
//!   `LivingEntity` damage path, so they are destroyable like other decorations.

use std::sync::{Arc, Weak};

use glam::DVec3;
use steel_macros::entity_behavior;
use steel_registry::entity_data::HumanoidArm;
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::vanilla_entity_data::MannequinEntityData;
use steel_utils::locks::SyncMutex;
use steel_utils::{DowncastType, DowncastTypeKey};

use crate::entity::{
    Entity, EntityBase, EntityBaseLoad, EntityPose, EntitySyncedData, LivingEntity,
    LivingEntityBase, SharedEntity,
};
use crate::world::World;

/// The vanilla mannequin decoration entity.
#[entity_behavior(class = "Mannequin")]
pub struct MannequinEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    entity_data: SyncMutex<MannequinEntityData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `MannequinEntity`.
unsafe impl DowncastType for MannequinEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/mannequin");
}

impl MannequinEntity {
    /// Creates a fresh mannequin from the generic entity factory path.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Creates a mannequin from persistent entity data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self::new_with_base(
            EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
        )
    }

    fn new_with_base(base: EntityBase, entity_type: EntityTypeRef) -> Self {
        let living_base = LivingEntityBase::new(entity_type);
        let mut entity_data = MannequinEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        Self {
            base,
            entity_type,
            living_base,
            entity_data: SyncMutex::new(entity_data),
        }
    }

    /// Vanilla `Avatar.getMainHand` (`DATA_PLAYER_MAIN_HAND`).
    #[must_use]
    pub fn main_hand(&self) -> HumanoidArm {
        *self.entity_data.lock().avatar().player_main_hand.get()
    }

    /// Sets the synced main hand.
    pub fn set_main_hand(&self, arm: HumanoidArm) {
        self.entity_data.lock().avatar_mut().player_main_hand.set(arm);
    }

    /// Vanilla `Avatar.getPlayerModeCustomisation` skin mask byte.
    #[must_use]
    pub fn skin_customisation(&self) -> i8 {
        *self.entity_data.lock().avatar().player_mode_customization.get()
    }

    /// Sets the raw skin customisation mask.
    pub fn set_skin_customisation(&self, mask: i8) {
        self.entity_data
            .lock()
            .avatar_mut()
            .player_mode_customization
            .set(mask);
    }

    /// Vanilla `Mannequin.isImmovable`.
    #[must_use]
    pub fn is_immovable(&self) -> bool {
        *self.entity_data.lock().immovable.get()
    }

    /// Sets the vanilla `immovable` flag.
    pub fn set_immovable(&self, immovable: bool) {
        self.entity_data.lock().immovable.set(immovable);
    }
}

impl Entity for MannequinEntity {
    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn synced_data(&self) -> Option<&dyn EntitySyncedData> {
        Some(&self.entity_data)
    }

    fn is_pushable(&self) -> bool {
        !self.is_immovable()
    }

    fn is_hard_collision_relevant(&self) -> bool {
        false
    }

    fn tick(self: std::sync::Arc<Self>) {
        let entity: SharedEntity = Arc::<Self>::clone(&self);
        self.tick_living_entity(&entity);
    }

    fn base_tick(&self) {
        LivingEntity::base_tick_living_entity(self);
    }

    fn dimensions_for_pose(&self, _pose: EntityPose) -> EntityDimensions {
        self.entity_type.dimensions
    }
}

impl LivingEntity for MannequinEntity {
    fn living_base(&self) -> &LivingEntityBase {
        &self.living_base
    }

    fn get_health(&self) -> f32 {
        *self.entity_data.lock().living_entity().health.get()
    }

    fn set_health(&self, health: f32) {
        let max_health = self.get_max_health();
        let clamped = health.clamp(0.0, max_health);
        self.entity_data
            .lock()
            .living_entity_mut()
            .health
            .set(clamped);
    }

    fn push_entities(&self) {}
}
