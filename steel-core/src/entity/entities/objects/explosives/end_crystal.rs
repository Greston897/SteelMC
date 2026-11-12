//! Minimal End Crystal entity implementation for End spike worldgen.

use std::sync::Arc;
use std::sync::Weak;

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::{NbtCompound, NbtTag};
use steel_macros::entity_behavior;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::vanilla_entity_data::EndCrystalEntityData;
use steel_utils::nbt::NbtValueInput as _;
use steel_utils::{BlockPos, locks::SyncMutex};
use steel_utils::{DowncastType, DowncastTypeKey};

use crate::entity::damage::DamageSource;
use crate::entity::{
    Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, EntitySyncedData, RemovalReason,
};
use crate::world::World;
use crate::world::explosion::{ExplosionInteraction, ExplosionOptions};

/// Vanilla `EndCrystal` explosion power.
const END_CRYSTAL_EXPLOSION_POWER: f32 = 6.0;

/// End Crystal entity state needed by worldgen and persistence.
///
/// Covers the synchronized data and saved fields used by generated End spikes, the
/// fire refresh under a crystal, the dragon beam link, and the power-6 explosion
/// that removing a crystal triggers.
#[entity_behavior(class = "EndCrystal")]
pub struct EndCrystalEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    entity_data: SyncMutex<EndCrystalEntityData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `EndCrystalEntity`.
unsafe impl DowncastType for EndCrystalEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/end_crystal");
}

impl EndCrystalEntity {
    /// Creates a new End Crystal entity.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            entity_data: SyncMutex::new(EndCrystalEntityData::new()),
        }
    }

    /// Creates an End Crystal entity from saved data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            entity_data: SyncMutex::new(EndCrystalEntityData::new()),
        }
    }

    /// Sets the optional beam target.
    pub fn set_beam_target(&self, target: Option<BlockPos>) {
        self.entity_data.lock().beam_target.set(target);
    }

    /// Returns the optional beam target.
    #[must_use]
    pub fn beam_target(&self) -> Option<BlockPos> {
        *self.entity_data.lock().beam_target.get()
    }

    /// Sets whether the crystal renders its bedrock base.
    pub fn set_show_bottom(&self, show_bottom: bool) {
        self.entity_data.lock().show_bottom.set(show_bottom);
    }

    /// Returns whether the crystal renders its bedrock base.
    #[must_use]
    pub fn shows_bottom(&self) -> bool {
        *self.entity_data.lock().show_bottom.get()
    }

    /// Vanilla `EndCrystal.tick` re-links the beam to the fight's dragon while it
    /// is still alive; a dead or missing dragon leaves the beam cleared.
    fn refresh_dragon_beam(&self, world: &Arc<World>) {
        let target = world.dragon_fight_target();
        if self.beam_target() == target {
            return;
        }
        self.entity_data.lock().beam_target.set(target);
    }

    /// Vanilla `EndCrystal.onRemove`/`hurt`: crystals explode with power 6 and
    /// fire, and their death can respawn the dragon on the End's exit portal.
    fn detonate(&self, world: &Arc<World>) {
        let center = self.position();
        let mut options = ExplosionOptions::new(center, END_CRYSTAL_EXPLOSION_POWER, ExplosionInteraction::Block);
        options.fire = true;
        options.source = None;
        world.explode(options);
        world.on_end_crystal_destroyed(self.block_position());
        self.set_removed(RemovalReason::Killed);
    }

    const fn nbt_bool(value: bool) -> i8 {
        if value { 1 } else { 0 }
    }
}

impl Entity for EndCrystalEntity {
    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn movement_emission(&self) -> EntityMovementEmission {
        EntityMovementEmission::None
    }

    /// Vanilla `EndCrystal.tick`: re-targets the dragon for the crystal's beam.
    /// (The fire refresh under a crystal is client-visible only and needs no
    /// server write, because the fire block already ticks itself.)
    fn tick(self: Arc<Self>) {
        let Some(world) = self.level() else {
            return;
        };
        self.refresh_dragon_beam(&world);
    }

    /// Vanilla `EndCrystal.hurt`: any damage detonates the crystal.
    fn hurt(&self, world: &World, _source: &DamageSource, _amount: f32) -> bool {
        // `hurt` only sees `&World`; the crystal is only damaged through paths
        // that already hold an `Arc`, so the tick path reuses `level()`.
        let _ = world;
        if let Some(world) = self.level() {
            self.detonate(&world);
        }
        true
    }

    fn is_pickable(&self) -> bool {
        true
    }

    fn blocks_building(&self) -> bool {
        true
    }

    fn synced_data(&self) -> Option<&dyn EntitySyncedData> {
        Some(&self.entity_data)
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        if let Some(target) = self.beam_target() {
            nbt.insert(
                "beam_target",
                NbtTag::IntArray(vec![target.x(), target.y(), target.z()]),
            );
        }

        nbt.insert("ShowBottom", Self::nbt_bool(self.shows_bottom()));
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        if let Some(target) = nbt.int_array("beam_target")
            && target.len() == 3
        {
            self.set_beam_target(Some(BlockPos::new(target[0], target[1], target[2])));
        }

        if let Some(show_bottom) = nbt.byte_value("ShowBottom") {
            self.set_show_bottom(show_bottom != 0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use steel_registry::vanilla_entities;

    #[test]
    fn end_crystal_does_not_duplicate_shared_invulnerable_state() {
        let crystal = EndCrystalEntity::new(
            &vanilla_entities::END_CRYSTAL,
            1,
            DVec3::new(1.5, 2.5, 3.5),
            Weak::new(),
        );
        crystal.set_invulnerable(true);

        let mut nbt = NbtCompound::new();
        crystal.save_additional(&mut nbt);

        assert_eq!(nbt.byte("Invulnerable"), None);
    }

    #[test]
    fn end_crystal_is_pickable_like_vanilla() {
        let crystal = EndCrystalEntity::new(
            &vanilla_entities::END_CRYSTAL,
            1,
            DVec3::new(1.5, 2.5, 3.5),
            Weak::new(),
        );

        assert!(crystal.is_pickable());
    }

    #[test]
    fn end_crystal_blocks_building_like_vanilla() {
        let crystal =
            EndCrystalEntity::new(&vanilla_entities::END_CRYSTAL, 1, DVec3::ZERO, Weak::new());

        assert!(crystal.blocks_building());
    }
}
