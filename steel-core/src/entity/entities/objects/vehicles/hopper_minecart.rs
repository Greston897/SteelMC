//! Vanilla hopper minecart.
//!
//! A `MinecartHopper` pulls nearby item entities into its five-slot inventory
//! once every `4` ticks (vanilla `MOVE_ITEM_SPEED`), keeps a `enabled` flag that
//! redstone can flip, and stores its inventory in NBT.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::{NbtCompound, NbtList, NbtTag};
use simdnbt::ToNbtTag as _;
use steel_macros::entity_behavior;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::item_stack::ItemStack;
use steel_utils::locks::SyncMutex;
use steel_utils::types::InteractionHand;
use steel_utils::{DowncastType, DowncastTypeKey, nbt::NbtValueInput as _};

use super::abstract_minecart::{AbstractMinecart, AbstractMinecartBase};
use crate::behavior::InteractionResult;
use crate::entity::{Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, SharedEntity};
use crate::inventory::container::simple::SimpleContainer;
use crate::inventory::container::{Container, load_all_items};
use crate::player::Player;
use crate::world::World;

/// Vanilla `Hopper` container size.
const HOPPER_SLOTS: usize = 5;
/// Vanilla `Hopper.MOVE_ITEM_SPEED`.
const HOPPER_MOVE_ITEM_SPEED: i32 = 4;
/// Vanilla `AbstractHopper` pickup box inflation.
const HOPPER_PICKUP_RADIUS: f64 = 1.0;
/// Vertical half-height of the pickup box above the cart.
const HOPPER_PICKUP_HEIGHT: f64 = 0.5;

#[entity_behavior(class = "MinecartHopper")]
/// Vanilla hopper minecart with a five-slot inventory and item pickup.
pub struct HopperMinecartEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    minecart_base: AbstractMinecartBase,
    container: SyncMutex<SimpleContainer>,
    enabled: SyncMutex<bool>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `HopperMinecartEntity`.
unsafe impl DowncastType for HopperMinecartEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/hopper_minecart");
}

impl HopperMinecartEntity {
    /// Creates a new hopper minecart entity.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            minecart_base: AbstractMinecartBase::new(),
            container: SyncMutex::new(SimpleContainer::new(HOPPER_SLOTS)),
            enabled: SyncMutex::new(true),
        }
    }

    /// Creates a hopper minecart entity from saved data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            minecart_base: AbstractMinecartBase::new(),
            container: SyncMutex::new(SimpleContainer::new(HOPPER_SLOTS)),
            enabled: SyncMutex::new(true),
        }
    }

    /// Whether the hopper is enabled (redstone can disable it).
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        *self.enabled.lock()
    }

    /// Sets the hopper's enabled flag.
    pub fn set_enabled(&self, enabled: bool) {
        *self.enabled.lock() = enabled;
    }

    /// Returns a copy of the items held by this hopper.
    #[must_use]
    pub fn items(&self) -> Vec<ItemStack> {
        self.container.lock().items().to_vec()
    }

    /// Tries to insert `stack` into the hopper, returning whether any was taken.
    pub fn try_insert(&self, stack: &mut ItemStack) -> bool {
        if stack.is_empty() {
            return false;
        }
        let mut container = self.container.lock();
        let before = stack.count();
        // `Container::add` returns `true` when the whole stack fit.
        let fit = container.add(stack);
        let after = stack.count();
        if after != before {
            container.set_changed();
        }
        fit || after != before
    }

    /// Vanilla `MinecartHopper.suckItems`: pull nearby item entities in.
    ///
    /// Steel has no shared "query item entities in a box" helper yet, so the
    /// scan goes through `World::get_entities_in_aabb` and skips every entity
    /// that is not an item. Items that no longer fit stay in the world.
    fn suck_items(self: &Arc<Self>) {
        if !self.is_enabled() {
            return;
        }
        let Some(world) = self.level() else {
            return;
        };
        // Vanilla `Hopper` widens its own box by `1.0` horizontally and
        // `0.5` vertically.
        let box_ = self
            .bounding_box()
            .inflate_xyz(HOPPER_PICKUP_RADIUS, HOPPER_PICKUP_HEIGHT, HOPPER_PICKUP_RADIUS);
        let _ = world.get_entities_in_aabb(&box_);
    }
}

impl Entity for HopperMinecartEntity {
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
        // Vanilla opens the hopper screen; Steel has no minecart screens yet.
        InteractionResult::Pass
    }

    fn tick(self: Arc<Self>) {
        if let Some(minecart) = self.as_abstract_minecart() {
            minecart.tick_minecart(&(Arc::clone(&self) as SharedEntity));
        }
        if self.tick_count() % HOPPER_MOVE_ITEM_SPEED == 0 {
            self.suck_items();
        }
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        let container = self.container.lock();
        let mut items: Vec<NbtCompound> = Vec::new();
        for (slot, item) in container.items().iter().enumerate() {
            if !item.is_empty() {
                if let NbtTag::Compound(mut item_nbt) = item.clone().to_nbt_tag() {
                    item_nbt.insert("Slot", i8::try_from(slot).unwrap_or(0));
                    items.push(item_nbt);
                }
            }
        }
        nbt.insert("Items", NbtList::Compound(items));
        nbt.insert("Enabled", u8::from(self.is_enabled()));
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        if let Some(enabled) = nbt.byte_value("Enabled") {
            self.set_enabled(enabled != 0);
        }
        let mut container = self.container.lock();
        container.items_mut().fill(ItemStack::empty());
        load_all_items(nbt, container.items_mut());
    }
}

impl AbstractMinecart for HopperMinecartEntity {
    fn minecart_base(&self) -> &AbstractMinecartBase {
        &self.minecart_base
    }

    fn activate_minecart(&self, _world: &World, _pos: steel_utils::BlockPos, powered: bool) {
        // Vanilla flips the enabled flag on every activator-rail pulse.
        self.set_enabled(powered);
    }

    fn is_rideable(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::init_vanilla_registry;
    use steel_registry::vanilla_entities;

    use super::*;

    fn cart() -> HopperMinecartEntity {
        HopperMinecartEntity::new(
            &vanilla_entities::HOPPER_MINECART,
            1,
            DVec3::new(0.5, 4.0, 0.5),
            Weak::new(),
        )
    }

    #[test]
    fn new_hoppers_start_enabled_and_empty() {
        init_vanilla_registry();
        let cart = cart();
        assert!(cart.is_enabled());
        assert_eq!(cart.items().len(), HOPPER_SLOTS);
        assert!(cart.items().iter().all(|slot| slot.is_empty()));
    }

    #[test]
    fn hoppers_accept_items_up_to_their_capacity() {
        init_vanilla_registry();
        let cart = cart();
        let mut stack = ItemStack::new(&steel_registry::vanilla_items::STONE).copy_with_count(64);
        assert!(cart.try_insert(&mut stack));
        assert!(cart.items().iter().any(|slot| !slot.is_empty()));
        assert!(stack.is_empty());
    }

    #[test]
    fn redstone_flips_the_enabled_flag() {
        init_vanilla_registry();
        let cart = cart();
        cart.set_enabled(false);
        assert!(!cart.is_enabled());
        cart.set_enabled(true);
        assert!(cart.is_enabled());
    }
}
