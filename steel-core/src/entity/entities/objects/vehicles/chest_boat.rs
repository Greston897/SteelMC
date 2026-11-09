//! Chest boat entities (`ChestBoat` and `ChestRaft`).
//!
//! Port of 26.3 `AbstractChestBoatEntity`: a boat that seats a single passenger
//! and stores a 27-slot inventory. Breaking it drops the boat item and scatters
//! the contents; sneaking (or an occupied boat) opens the chest instead of
//! boarding.
//!
//! Known divergences, all caused by missing subsystems:
//! - loot tables (`LootTable`/`LootableInventory`) are not ported, so a chest
//!   boat never generates loot;
//! - vanilla scatters the inventory on *every* destroying removal
//!   (`Entity#remove`); Steel only does it when the boat is destroyed by damage
//!   (see [`AbstractBoat::scatter_boat_contents`]);
//! - `PiglinBrain.onGuardedBlockInteracted` has no equivalent;
//! - `getPassengerHorizontalOffset` (0.15) is not applied, so the rider sits on
//!   the dimension attachment point instead of slightly ahead of it.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::{NbtCompound, NbtList, NbtTag};
use steel_macros::entity_behavior;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_entity_data::AbstractBoatEntityData;
use steel_registry::vanilla_game_events;
use steel_utils::locks::{IntoShared as _, SyncMutex};
use steel_utils::types::InteractionHand;
use steel_utils::{DowncastType, DowncastTypeKey};

use super::boat::{
    AbstractBoat, BoatState, GRAVITY_IN_WATER, boat_entity_impl, boat_tick, ride_boat_interact,
};
use super::vehicle_entity::{
    VEHICLE_DIMENSION_CHANGING_DELAY, VehicleEntity, vehicle_ignores_explosion,
};
use crate::behavior::InteractionResult;
use crate::entity::{
    DamageSource, Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, EntitySyncedData,
    Player, SharedEntity,
};
use crate::inventory::container::{Container as _, SimpleContainer, load_all_items};
use crate::inventory::menu::kinds::chest_boat;
use crate::world::World;

/// Vanilla `AbstractChestBoatEntity.INVENTORY_SIZE`.
pub const CHEST_BOAT_SLOTS: usize = 27;

/// Vanilla `AbstractChestBoatEntity.getMaxPassengers`.
pub const CHEST_BOAT_MAX_PASSENGERS: usize = 1;

fn new_inventory() -> steel_utils::locks::Shared<SimpleContainer> {
    SimpleContainer::new(CHEST_BOAT_SLOTS).into_shared()
}

/// Scatters every stored stack at the boat's position.
fn scatter(entity: &dyn Entity, inventory: &steel_utils::locks::Shared<SimpleContainer>) {
    let mut contents = Vec::new();
    {
        let mut container = inventory.lock();
        for slot in container.items_mut() {
            if !slot.is_empty() {
                contents.push(std::mem::replace(slot, ItemStack::empty()));
            }
        }
    }
    for stack in contents {
        let _ = entity.spawn_at_location(stack, 0.0);
    }
}

/// Persists the chest inventory under vanilla's `Items` key.
fn save_items(nbt: &mut NbtCompound, inventory: &steel_utils::locks::Shared<SimpleContainer>) {
    let container = inventory.lock();
    let mut entries = Vec::new();
    for (slot, stack) in container.items().iter().enumerate() {
        if stack.is_empty() {
            continue;
        }
        if let NbtTag::Compound(mut item_nbt) = stack.to_nbt_tag_ref() {
            item_nbt.insert("Slot", slot as i8);
            entries.push(item_nbt);
        }
    }
    nbt.insert("Items", NbtList::Compound(entries));
}

fn load_items(
    nbt: BorrowedNbtCompoundView<'_, '_>,
    inventory: &steel_utils::locks::Shared<SimpleContainer>,
) {
    let mut container = inventory.lock();
    load_all_items(nbt, container.items_mut());
}

/// Opens the 3×9 chest menu for `player`, mirroring vanilla `openInventory`.
fn open_chest(
    entity: &dyn Entity,
    player: &Player,
    inventory: &steel_utils::locks::Shared<SimpleContainer>,
) -> InteractionResult {
    let Some(world) = entity.level() else {
        return InteractionResult::Pass;
    };
    let Some(shared) = world.get_entity_by_id(entity.id()) else {
        return InteractionResult::Pass;
    };

    // The menu stays valid only while the boat lives and the player is close,
    // so it holds a weak owner reference.
    let owner: Weak<dyn Entity> = Arc::downgrade(&shared);
    drop(shared);

    let player_inventory = Arc::clone(&player.inventory);
    let container = Arc::clone(inventory);
    player.open_menu(entity.display_name(), move |context| {
        chest_boat(player_inventory, context.container_id, container, owner)
    });

    entity.game_event_with_source_entity(
        &vanilla_game_events::CONTAINER_OPEN,
        Some(player as &dyn Entity),
    );
    InteractionResult::Success
}

/// Vanilla `AbstractChestBoatEntity#interact` tail: open the container.
fn chest_boat_interact(
    entity: &dyn Entity,
    player: &Player,
    inventory: &steel_utils::locks::Shared<SimpleContainer>,
) -> InteractionResult {
    let result = ride_boat_interact(entity, player);
    if result != InteractionResult::Pass {
        return result;
    }

    if entity.can_add_passenger(player) && !player.is_secondary_use_active() {
        return InteractionResult::Pass;
    }

    open_chest(entity, player, inventory)
}

/// Macro body shared by `ChestBoatEntity` and `ChestRaftEntity`.
/// A chest boat (any wood type).
#[entity_behavior(class = "ChestBoat")]
pub struct ChestBoatEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    entity_data: SyncMutex<AbstractBoatEntityData>,
    boat_state: SyncMutex<BoatState>,
    inventory: steel_utils::locks::Shared<SimpleContainer>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `ChestBoatEntity`.
unsafe impl DowncastType for ChestBoatEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/chest_boat");
}

impl ChestBoatEntity {
    /// Creates a chest boat entity.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            entity_data: SyncMutex::new(AbstractBoatEntityData::new()),
            boat_state: SyncMutex::new(BoatState::default()),
            inventory: new_inventory(),
        }
    }

    /// Creates a chest boat entity from saved data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            entity_data: SyncMutex::new(AbstractBoatEntityData::new()),
            boat_state: SyncMutex::new(BoatState::default()),
            inventory: new_inventory(),
        }
    }
}

boat_entity_impl!(
    ChestBoatEntity,
    CHEST_BOAT_MAX_PASSENGERS,
    |this: &ChestBoatEntity, player: &Player| chest_boat_interact(this, player, &this.inventory);
    fn max_passengers(&self) -> usize {
        CHEST_BOAT_MAX_PASSENGERS
    }

    fn scatter_boat_contents(&self, world: &World) {
        let _ = world;
        scatter(self, &self.inventory);
    }

    fn save_boat_extra(&self, nbt: &mut NbtCompound) {
        save_items(nbt, &self.inventory);
    }

    fn load_boat_extra(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        load_items(nbt, &self.inventory);
    }
);

/// A bamboo chest raft.
#[entity_behavior(class = "ChestRaft")]
pub struct ChestRaftEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    entity_data: SyncMutex<AbstractBoatEntityData>,
    boat_state: SyncMutex<BoatState>,
    inventory: steel_utils::locks::Shared<SimpleContainer>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `ChestRaftEntity`.
unsafe impl DowncastType for ChestRaftEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/chest_raft");
}

impl ChestRaftEntity {
    /// Creates a chest raft entity.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            entity_data: SyncMutex::new(AbstractBoatEntityData::new()),
            boat_state: SyncMutex::new(BoatState::default()),
            inventory: new_inventory(),
        }
    }

    /// Creates a chest raft entity from saved data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            entity_data: SyncMutex::new(AbstractBoatEntityData::new()),
            boat_state: SyncMutex::new(BoatState::default()),
            inventory: new_inventory(),
        }
    }
}

boat_entity_impl!(
    ChestRaftEntity,
    CHEST_BOAT_MAX_PASSENGERS,
    |this: &ChestRaftEntity, player: &Player| chest_boat_interact(this, player, &this.inventory);
    fn max_passengers(&self) -> usize {
        CHEST_BOAT_MAX_PASSENGERS
    }

    fn scatter_boat_contents(&self, world: &World) {
        let _ = world;
        scatter(self, &self.inventory);
    }

    fn save_boat_extra(&self, nbt: &mut NbtCompound) {
        save_items(nbt, &self.inventory);
    }

    fn load_boat_extra(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        load_items(nbt, &self.inventory);
    }
);

#[cfg(test)]
mod tests {
    use std::sync::Weak;

    use glam::DVec3;
    use steel_registry::{init_vanilla_registry, vanilla_entities};

    use super::{CHEST_BOAT_SLOTS, ChestBoatEntity};
    use crate::entity::entities::objects::vehicles::boat::AbstractBoat as _;
    use crate::inventory::container::Container as _;
    use crate::world::World;

    #[test]
    fn a_chest_boat_has_a_full_size_inventory() {
        init_vanilla_registry();
        let boat = ChestBoatEntity::new(
            &vanilla_entities::OAK_CHEST_BOAT,
            1,
            DVec3::ZERO,
            Weak::<World>::new(),
        );
        assert_eq!(boat.inventory.lock().items().len(), CHEST_BOAT_SLOTS);
        assert!(boat.inventory.lock().items().iter().all(|s| s.is_empty()));
    }

    #[test]
    fn a_chest_boat_seats_a_single_passenger() {
        assert_eq!(super::CHEST_BOAT_MAX_PASSENGERS, 1);
    }

    #[test]
    fn a_chest_boat_drops_the_matching_item() {
        init_vanilla_registry();
        let boat = ChestBoatEntity::new(
            &vanilla_entities::OAK_CHEST_BOAT,
            1,
            DVec3::ZERO,
            Weak::<World>::new(),
        );
        assert_eq!(
            boat.boat_drop_item().key,
            steel_registry::vanilla_items::OAK_CHEST_BOAT.key
        );
        assert!(!boat.boat_state().lock().location.is_water());
    }
}
