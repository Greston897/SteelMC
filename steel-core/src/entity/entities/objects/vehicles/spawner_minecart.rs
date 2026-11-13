//! Vanilla spawner minecart.
//!
//! A `MinecartSpawner` reuses the exact same `BaseSpawner` state machine as the
//! spawner block: it remembers a `SpawnData` plus a delay, requires a player
//! nearby, and spawns up to four mobs per cycle. Only the host position differs
//! (the cart's own block position instead of a fixed block).

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::{
    BaseNbtCompound as BorrowedNbtCompound, NbtCompound as BorrowedNbtCompoundView,
};
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::vanilla_entity_data::MinecartEntityData;
use steel_utils::locks::SyncMutex;
use steel_utils::BlockStateId;
use steel_utils::types::InteractionHand;
use steel_utils::{BlockPos, DowncastType, DowncastTypeKey};

use super::abstract_minecart::{AbstractMinecart, AbstractMinecartBase};
use crate::behavior::InteractionResult;
use crate::block_entity::entities::BaseSpawner;
use crate::entity::{Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, SharedEntity};
use crate::player::Player;
use crate::world::World;

#[entity_behavior(class = "MinecartSpawner")]
/// Vanilla spawner minecart.
///
/// The `BaseSpawner` state lives behind a `SyncMutex` because `Entity::tick`
/// takes `&self`.
pub struct SpawnerMinecartEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    entity_data: SyncMutex<MinecartEntityData>,
    minecart_base: AbstractMinecartBase,
    spawner: SyncMutex<BaseSpawner>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `SpawnerMinecartEntity`.
unsafe impl DowncastType for SpawnerMinecartEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/spawner_minecart");
}

impl SpawnerMinecartEntity {
    /// Creates a new spawner minecart entity.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            entity_data: SyncMutex::new(MinecartEntityData::new()),
            minecart_base: AbstractMinecartBase::new(),
            spawner: SyncMutex::new(BaseSpawner::new()),
        }
    }

    /// Creates a spawner minecart entity from saved data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            entity_data: SyncMutex::new(MinecartEntityData::new()),
            minecart_base: AbstractMinecartBase::new(),
            spawner: SyncMutex::new(BaseSpawner::new()),
        }
    }

    /// Applies a spawner event (`set_entity_id`, `set_spawn_data`, ...).
    ///
    /// Mirrors `BlockEntity` events so pistons/commands can retarget the cart.
    pub fn apply_event(&self, event: i32) -> bool {
        self.spawner.lock().on_event_triggered(event)
    }

    /// The host block position, matching vanilla `Entity.blockPosition()`.
    fn block_pos(&self) -> BlockPos {
        BlockPos::from(self.position())
    }

    /// Vanilla `MinecartSpawner` has no client-side ticking.
    fn server_tick(self: &Arc<Self>) {
        let Some(world) = self.level() else {
            return;
        };
        let _ = self
            .spawner
            .lock()
            .server_tick(&world, self.block_pos());
    }
}

impl Entity for SpawnerMinecartEntity {
    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn movement_emission(&self) -> EntityMovementEmission {
        EntityMovementEmission::Events
    }

    fn interact(
        &self,
        _player: &Player,
        _hand: InteractionHand,
        _location: DVec3,
    ) -> InteractionResult {
        InteractionResult::Pass
    }

    fn tick(self: Arc<Self>) {
        if let Some(minecart) = self.as_abstract_minecart() {
            minecart.tick_minecart(&(Arc::clone(&self) as SharedEntity));
        }
        self.server_tick();
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.spawner.lock().save(nbt);
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        // `BaseSpawner::load` takes the trait object and converts to the view
        // itself, so the compound is handed over unchanged.
        let view: &dyn BorrowedNbtCompound<'_> = &nbt;
        self.spawner.lock().load(view);
    }
}

impl AbstractMinecart for SpawnerMinecartEntity {
    fn minecart_base(&self) -> &AbstractMinecartBase {
        &self.minecart_base
    }
    fn custom_display_block(&self) -> Option<BlockStateId> {
        *self
            .entity_data
            .lock()
            .abstract_minecart()
            .id_custom_display_block
            .get()
    }

    fn set_custom_display_block(&self, value: Option<BlockStateId>) {
        self.entity_data
            .lock()
            .abstract_minecart_mut()
            .id_custom_display_block
            .set(value);
    }

    fn display_offset(&self) -> i32 {
        *self
            .entity_data
            .lock()
            .abstract_minecart()
            .id_display_offset
            .get()
    }

    fn set_display_offset(&self, value: i32) {
        self.entity_data
            .lock()
            .abstract_minecart_mut()
            .id_display_offset
            .set(value);
    }

}

#[cfg(test)]
mod tests {
    use steel_registry::init_vanilla_registry;
    use steel_registry::vanilla_entities;

    use super::*;

    fn cart() -> SpawnerMinecartEntity {
        SpawnerMinecartEntity::new(
            &vanilla_entities::SPAWNER_MINECART,
            1,
            DVec3::new(0.5, 4.0, 0.5),
            Weak::new(),
        )
    }

    #[test]
    fn new_spawner_minecarts_have_a_live_spawner() {
        init_vanilla_registry();
        let cart = cart();
        assert!(cart.spawner.lock().next_spawn_data().is_none());
        assert!(cart.spawner.lock().spawn_delay() >= -1);
    }

    #[test]
    fn spawner_event_returns_whether_it_was_applied() {
        init_vanilla_registry();
        let cart = cart();
        // Vanilla `BaseSpawner.onEventTriggered` accepts every event.
        assert!(cart.apply_event(1));
    }

    #[test]
    fn saved_spawners_round_trip_their_delay() {
        init_vanilla_registry();
        let cart = cart();
        cart.spawner.lock().set_spawn_delay(42);
        let mut nbt = NbtCompound::new();
        cart.spawner.lock().save(&mut nbt);
        assert!(nbt.get("Delay").is_some());
    }
}
