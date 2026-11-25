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
use steel_registry::vanilla_entity_data::MinecartFurnaceEntityData;
use steel_utils::BlockStateId;
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

/// Vanilla `MinecartFurnace` fuel added per ignition.
pub const FURNACE_FUEL_PER_IGNITION: i32 = 3600;
/// Vanilla `MinecartFurnace` ceiling for accumulated fuel.
pub const FURNACE_MAX_FUEL: i32 = 32000;
/// Vanilla `MinecartFurnace.applyNaturalSlowdown`: burning carts damp the
/// horizontal axes by `0.8` and re-add the stored push vector.
pub const FURNACE_BURNING_SLOWDOWN: f64 = 0.8;

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
    /// Vanilla `MinecartFurnace.fuel`: remaining burn ticks. Each fuel item adds
    /// `3600` and the counter can hold up to `32000`.
    fuel: i32,
    /// Vanilla `MinecartFurnace.push`, kept in the XZ plane.
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
        *self.entity_data.lock().minecart_furnace().id_fuel.get()
    }

    /// Sets the synced burning flag.
    pub fn set_lit(&self, lit: bool) {
        self.entity_data
            .lock()
            .minecart_furnace_mut()
            .id_fuel
            .set(lit);
    }

    /// Vanilla `MinecartFurnace.fuel`: burn ticks left before the cart stops.
    #[must_use]
    pub fn fuel_ticks(&self) -> i32 {
        self.state.lock().fuel
    }

    /// Vanilla `MinecartFurnace.push`, the stored push vector.
    #[must_use]
    pub fn push_direction(&self) -> DVec3 {
        self.state.lock().push_direction
    }

    /// Vanilla `MinecartFurnace.addFuel`.
    ///
    /// Fuel only accumulates when the fuel-tag item is accepted and the tank has
    /// room for another `3600`; a successful ignition then pushes the cart along
    /// the horizontal vector *away from the interacting position*.
    pub fn add_fuel(&self, interacting_pos: DVec3, accepted: bool) -> bool {
        if !accepted {
            return false;
        }
        let mut state = self.state.lock();
        if state.fuel + FURNACE_FUEL_PER_IGNITION > FURNACE_MAX_FUEL {
            return false;
        }
        state.fuel += FURNACE_FUEL_PER_IGNITION;
        let away = self.position() - interacting_pos;
        state.push_direction = DVec3::new(away.x, 0.0, away.z);
        true
    }

    /// Vanilla `MinecartFurnace.tick`: burn the fuel down and keep the lit flag
    /// in step with the remaining fuel.
    fn tick_furnace(&self) {
        let lit = {
            let mut state = self.state.lock();
            if state.fuel > 0 {
                state.fuel -= 1;
            }
            if state.fuel <= 0 {
                state.push_direction = DVec3::ZERO;
            }
            state.fuel > 0
        };
        if self.is_lit() != lit {
            self.set_lit(lit);
        }
    }

    /// Vanilla `MinecartFurnace.calculateNewPushAlong`.
    ///
    /// While the cart is already moving, the stored push is re-aimed along that
    /// movement so a climbing cart keeps climbing instead of pushing into a wall.
    #[must_use]
    pub fn calculate_new_push_along(push: DVec3, movement: DVec3) -> DVec3 {
        // Vanilla `Vec3.horizontal()` drops the Y component.
        let push_horizontal = DVec3::new(push.x, 0.0, push.z);
        let movement_horizontal = DVec3::new(movement.x, 0.0, movement.z);
        if push_horizontal.length_squared() > 1.0e-4 && movement_horizontal.length_squared() > 0.001
        {
            let dot = push_horizontal.dot(movement_horizontal);
            let projected = movement_horizontal * (dot / movement_horizontal.length_squared());
            projected.normalize_or_zero() * push_horizontal.length()
        } else {
            push
        }
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
        player: &Player,
        hand: InteractionHand,
        location: DVec3,
    ) -> InteractionResult {
        // Vanilla `MinecartFurnace.interact`: feeding a `furnace_minecart_fuel`
        // item is what actually pushes the cart, and the stack shrinks by one.
        // Vanilla also opens the furnace screen, but a minecart has no screen in
        // Steel, and the fuel path is the gameplay-relevant half.
        let accepted = {
            let inventory = player.inventory.lock();
            let item = inventory.get_item_in_hand(hand);
            !item.is_empty()
                && item
                    .item()
                    .has_tag(&steel_utils::Identifier::vanilla_static(
                        "furnace_minecart_fuel",
                    ))
        };
        if !accepted {
            return InteractionResult::Pass;
        }
        if !self.add_fuel(location, true) {
            return InteractionResult::Pass;
        }
        // Vanilla `itemStack.consume(1, player)`.
        player.inventory.lock().shrink_item_in_hand(hand, 1);
        InteractionResult::Success
    }

    fn tick(self: Arc<Self>) {
        if let Some(minecart) = self.as_abstract_minecart() {
            minecart.tick_minecart(&(Arc::clone(&self) as SharedEntity));
        }
        self.tick_furnace();
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        let state = self.state.lock();
        // Vanilla writes `PushX` / `PushZ` / `Fuel` unconditionally.
        nbt.insert("PushX", state.push_direction.x);
        nbt.insert("PushZ", state.push_direction.z);
        nbt.insert("Fuel", i16::try_from(state.fuel).unwrap_or(i16::MAX));
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        let push_x = nbt.double_value("PushX").unwrap_or(0.0);
        let push_z = nbt.double_value("PushZ").unwrap_or(0.0);
        // Vanilla stores the fuel as a short (`getShortOr("Fuel", 0)`).
        let fuel = nbt.int_value("Fuel").unwrap_or(0);
        let mut state = self.state.lock();
        state.fuel = fuel;
        state.push_direction = DVec3::new(push_x, 0.0, push_z);
    }
}

impl AbstractMinecart for FurnaceMinecartEntity {
    fn minecart_base(&self) -> &AbstractMinecartBase {
        &self.minecart_base
    }

    fn activate_minecart(&self, _world: &World, _pos: BlockPos, _powered: bool) {
        // Vanilla `MinecartFurnace` does not override `activateMinecart`: a
        // powered rail does not ignite it. Only feeding fuel pushes the cart.
    }

    /// Vanilla `MinecartFurnace.applyNaturalSlowdown`.
    ///
    /// While the stored push is non-trivial it is re-aimed along the current
    /// motion, the horizontal axes are damped by `0.8`, and the push is added
    /// back. A burning cart in water is additionally scaled by `0.1`. With no
    /// push, the cart falls back to the plain `0.98` minecart drag.
    fn apply_natural_slowdown(&self, movement: DVec3) -> DVec3 {
        let push = {
            let mut state = self.state.lock();
            if state.push_direction.length_squared() > 1.0e-7 {
                state.push_direction =
                    FurnaceMinecartEntity::calculate_new_push_along(state.push_direction, movement);
            }
            state.push_direction
        };

        let adjusted = if push.length_squared() > 1.0e-7 {
            let damped =
                movement * DVec3::new(FURNACE_BURNING_SLOWDOWN, 0.0, FURNACE_BURNING_SLOWDOWN);
            let mut pushed = damped + push;
            if self.is_in_water() {
                pushed *= 0.1;
            }
            pushed
        } else {
            movement * DVec3::new(0.98, 0.0, 0.98)
        };
        // Vanilla chains into `super.applyNaturalSlowdown(result)`.
        self.apply_base_natural_slowdown(adjusted)
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
        assert_eq!(cart.fuel_ticks(), 0);
    }

    #[test]
    fn one_fuel_item_adds_the_vanilla_fuel_budget() {
        init_vanilla_registry();
        let cart = cart();
        assert!(cart.add_fuel(DVec3::new(-1.0, 0.0, 0.0), true));
        assert_eq!(cart.fuel_ticks(), FURNACE_FUEL_PER_IGNITION);
        // Vanilla pushes along `position - interactingPos`, so feeding from the
        // negative X side pushes the cart towards positive X.
        assert!(cart.push_direction().x > 0.0);
    }

    #[test]
    fn fuel_stops_accumulating_at_the_vanilla_ceiling() {
        init_vanilla_registry();
        let cart = cart();
        for _ in 0..8 {
            assert!(cart.add_fuel(DVec3::ZERO, true));
        }
        // 9 * 3600 = 32400 > 32000, so the ninth ignition is refused.
        assert_eq!(cart.fuel_ticks(), 8 * FURNACE_FUEL_PER_IGNITION);
        assert!(!cart.add_fuel(DVec3::ZERO, true));
    }
}
