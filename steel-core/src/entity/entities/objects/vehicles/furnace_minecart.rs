//! Vanilla furnace minecart.
//!
//! A `MinecartFurnace` is pushed by burning fuel: every ignition pushes the cart
//! along its facing for up to `320` ticks, and the synced `fuel` flag tells
//! clients to draw the smoke plume and the lit furnace texture.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_entity_data::MinecartFurnaceEntityData;
use steel_registry::vanilla_item_tags::ItemTag;
use steel_registry::{REGISTRY, TaggedRegistryExt};
use steel_utils::locks::SyncMutex;
use steel_utils::nbt::NbtValueInput as _;
use steel_utils::types::InteractionHand;
use steel_utils::{BlockPos, DowncastType, DowncastTypeKey};

use super::abstract_minecart::{AbstractMinecart, AbstractMinecartBase};
use crate::behavior::InteractionResult;
use crate::entity::{
    Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, EntitySyncedData, SharedEntity,
};
use crate::player::Player;
use crate::world::World;

/// Vanilla `MinecartFurnace` push time while burning (`320` ticks).
const FURNACE_PUSH_TIME: i32 = 320;
/// Vanilla `MinecartFurnace` push acceleration.
const FURNACE_PUSH_ACCELERATION: f64 = 0.0075;
/// Vanilla `MinecartFurnace` push drag while the cart is burning.
const FURNACE_PUSH_DRAG: f64 = 0.997;
/// Vanilla `MinecartFurnace` fuel per coal (`3600` ticks).
const FURNACE_COAL_TICKS: i32 = 3600;

#[entity_behavior(class = "MinecartFurnace")]
/// Vanilla furnace minecart with a synced burning flag.
pub struct FurnaceMinecartEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    entity_data: SyncMutex<MinecartFurnaceEntityData>,
    minecart_base: AbstractMinecartBase,
    state: SyncMutex<FurnaceMinecartState>,
}

#[derive(Debug, Default)]
struct FurnaceMinecartState {
    /// Ticks the cart keeps pushing after the last ignition.
    push_time: i32,
    /// Remaining burn ticks contributed by the current fuel item.
    fuel_ticks: i32,
    /// Direction the cart is pushed along, in the XZ plane.
    push_direction: DVec3,
}

// SAFETY: This key is owned by Steel and uniquely identifies `FurnaceMinecartEntity`.
unsafe impl DowncastType for FurnaceMinecartEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/furnace_minecart");
}

impl FurnaceMinecartEntity {
    /// Creates a new furnace minecart entity.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            entity_data: SyncMutex::new(MinecartFurnaceEntityData::new()),
            minecart_base: AbstractMinecartBase::new(),
            state: SyncMutex::new(FurnaceMinecartState::default()),
        }
    }

    /// Creates a furnace minecart entity from saved data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            entity_data: SyncMutex::new(MinecartFurnaceEntityData::new()),
            minecart_base: AbstractMinecartBase::new(),
            state: SyncMutex::new(FurnaceMinecartState::default()),
        }
    }

    /// Whether the furnace is currently burning (the synced `fuel` flag).
    #[must_use]
    pub fn is_lit(&self) -> bool {
        *self.entity_data.lock().fuel.get()
    }

    /// Sets the synced burning flag.
    pub fn set_lit(&self, lit: bool) {
        self.entity_data.lock().fuel.set(lit);
    }

    /// Remaining push ticks after the last ignition.
    #[must_use]
    pub fn push_time(&self) -> i32 {
        self.state.lock().push_time
    }

    /// Returns whether the stack can be used as furnace-minecart fuel.
    #[must_use]
    pub fn is_fuel(item_stack: &ItemStack) -> bool {
        REGISTRY
            .items
            .is_in_tag(item_stack.item(), &ItemTag::FURNACE_MINECART_FUEL)
    }

    /// Vanilla `MinecartFurnace.activateMinecart` / `push`: start pushing
    /// towards `direction` and remember the push time.
    pub fn push(&self, direction: DVec3) {
        {
            let mut state = self.state.lock();
            state.push_time = FURNACE_PUSH_TIME;
            let horizontal = DVec3::new(direction.x, 0.0, direction.z);
            if horizontal.length_squared() > 1.0e-7 {
                state.push_direction = horizontal.normalize();
            }
        }
        if self.is_lit() {
            return;
        }
        self.set_lit(true);
    }

    /// Vanilla `MinecartFurnace.tick`: burn fuel and push the cart along.
    fn tick_furnace(&self) {
        let (push_time, direction) = {
            let mut state = self.state.lock();
            if state.push_time > 0 {
                state.push_time -= 1;
            }
            (state.push_time, state.push_direction)
        };

        if push_time <= 0 {
            if self.is_lit() {
                self.set_lit(false);
            }
            return;
        }

        // Vanilla multiplies the current velocity by the push drag, then adds
        // the acceleration along the remembered direction.
        let velocity = self.velocity() * FURNACE_PUSH_DRAG + direction * FURNACE_PUSH_ACCELERATION;
        self.set_velocity(velocity);
    }
}

impl Entity for FurnaceMinecartEntity {
    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn movement_emission(&self) -> EntityMovementEmission {
        EntityMovementEmission::Events
    }

    fn synced_data(&self) -> Option<&dyn EntitySyncedData> {
        Some(&self.entity_data)
    }

    fn interact(
        &self,
        _player: &Player,
        _hand: InteractionHand,
        _location: DVec3,
    ) -> InteractionResult {
        // Vanilla opens the furnace screen; Steel has no minecart screens yet.
        InteractionResult::Pass
    }

    fn tick(self: Arc<Self>) {
        if let Some(minecart) = self.as_abstract_minecart() {
            minecart.tick_minecart(&(Arc::clone(&self) as SharedEntity));
        }
        self.tick_furnace();
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        let (push_time, fuel_ticks, direction) = {
            let state = self.state.lock();
            (state.push_time, state.fuel_ticks, state.push_direction)
        };
        if push_time > 0 {
            nbt.insert("PushX", direction.x);
            nbt.insert("PushZ", direction.z);
            nbt.insert("Fuel", push_time);
        }
        if fuel_ticks > 0 {
            nbt.insert("FuelTicks", fuel_ticks);
        }
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        let push_x = nbt.double_value("PushX").unwrap_or(0.0);
        let push_z = nbt.double_value("PushZ").unwrap_or(0.0);
        let fuel = nbt.int_value("Fuel").unwrap_or(0);
        let fuel_ticks = nbt.int_value("FuelTicks").unwrap_or(0);
        let mut state = self.state.lock();
        state.push_time = fuel;
        state.fuel_ticks = fuel_ticks;
        let horizontal = DVec3::new(push_x, 0.0, push_z);
        if horizontal.length_squared() > 1.0e-7 {
            state.push_direction = horizontal.normalize();
        }
    }
}

impl AbstractMinecart for FurnaceMinecartEntity {
    fn minecart_base(&self) -> &AbstractMinecartBase {
        &self.minecart_base
    }

    fn activate_minecart(&self, _world: &World, _pos: BlockPos, powered: bool) {
        if !powered {
            return;
        }
        // Vanilla pushes the cart forward along its current facing.
        let (yaw, _) = self.rotation();
        let rotation = yaw.to_radians();
        let direction = DVec3::new(-rotation.sin(), 0.0, rotation.cos());
        self.push(direction);
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::init_vanilla_registry;
    use steel_registry::vanilla_entities;
    use steel_registry::vanilla_items;

    use super::*;

    fn cart() -> FurnaceMinecartEntity {
        FurnaceMinecartEntity::new(
            &vanilla_entities::FURNACE_MINECART,
            1,
            DVec3::new(0.5, 4.0, 0.5),
            Weak::new(),
        )
    }

    #[test]
    fn new_furnace_minecarts_are_unlit() {
        init_vanilla_registry();
        let cart = cart();
        assert!(!cart.is_lit());
        assert_eq!(cart.push_time(), 0);
    }

    #[test]
    fn pushing_lights_the_furnace_for_the_vanilla_duration() {
        init_vanilla_registry();
        let cart = cart();
        cart.push(DVec3::new(1.0, 0.0, 0.0));
        assert!(cart.is_lit());
        assert_eq!(cart.push_time(), FURNACE_PUSH_TIME);
    }

    #[test]
    fn coal_is_furnace_minecart_fuel() {
        init_vanilla_registry();
        assert!(FurnaceMinecartEntity::is_fuel(&ItemStack::new(
            &vanilla_items::COAL
        )));
        assert!(!FurnaceMinecartEntity::is_fuel(&ItemStack::new(
            &vanilla_items::STONE
        )));
        assert_eq!(FURNACE_COAL_TICKS, 3600);
    }
}
