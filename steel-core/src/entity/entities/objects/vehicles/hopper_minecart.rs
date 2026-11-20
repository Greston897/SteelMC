//! Vanilla hopper minecart.
//!
//! A `MinecartHopper` pulls nearby item entities into its five-slot inventory
//! once every `4` ticks (vanilla `MOVE_ITEM_SPEED`), keeps a `enabled` flag that
//! redstone can flip, and stores its inventory in NBT.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::ToNbtTag as _;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::{NbtCompound, NbtList, NbtTag};
use steel_macros::entity_behavior;
use steel_registry::blocks::properties::Direction;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_entities;
use steel_registry::vanilla_entity_data::MinecartEntityData;
use steel_registry::vanilla_game_events;
use steel_utils::locks::{IntoShared as _, Shared, SyncMutex};
use steel_utils::types::InteractionHand;
use steel_utils::{BlockPos, BlockStateId};
use steel_utils::{Downcast as _, DowncastType, DowncastTypeKey, nbt::NbtValueInput as _};

use super::abstract_minecart::{AbstractMinecart, AbstractMinecartBase};
use crate::behavior::InteractionResult;
use crate::entity::entities::ItemEntity;
use crate::entity::{
    Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, RemovalReason, SharedEntity,
};
use crate::inventory::container::SimpleContainer;
use crate::inventory::container::{Container, load_all_items};
use crate::inventory::lock::{ContainerLockGuard, ContainerRef};
use crate::inventory::menu::kinds::hopper;
use crate::player::Player;
use crate::world::World;

/// Vanilla `Hopper` container size.
const HOPPER_SLOTS: usize = 5;
/// Vanilla `MinecartHopper` pick-up box inflation.
///
/// `MinecartHopper.suckInItems` uses `getBoundingBox().inflate(0.25, 0.0, 0.25)`
/// around the cart — a quarter block out sideways and *nothing* vertically, so
/// an item is only taken when it is level with the cart. The block hopper's
/// one-block-high `Hopper.SUCK_AABB` does not apply here.
const HOPPER_PICKUP_RADIUS: f64 = 0.25;
/// Vanilla `MinecartHopper` has no vertical inflation.
const HOPPER_PICKUP_HEIGHT: f64 = 0.0;
/// Vanilla `MinecartHopper.MOVE_ITEM_SPEED`: eight ticks between successful
/// pickups (the wiki's `TransferCooldown` range of 1..=8).
const HOPPER_MOVE_COOLDOWN: i32 = 8;

#[entity_behavior(class = "MinecartHopper")]
/// Vanilla hopper minecart with a five-slot inventory and item pickup.
pub struct HopperMinecartEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    entity_data: SyncMutex<MinecartEntityData>,
    minecart_base: AbstractMinecartBase,
    container: Shared<SimpleContainer>,
    enabled: SyncMutex<bool>,
    /// Vanilla `MinecartHopper.transferCooldown` (NBT `TransferCooldown`).
    transfer_cooldown: SyncMutex<i32>,
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
            entity_data: SyncMutex::new(MinecartEntityData::new()),
            minecart_base: AbstractMinecartBase::new(),
            container: SimpleContainer::new(HOPPER_SLOTS).into_shared(),
            enabled: SyncMutex::new(true),
            transfer_cooldown: SyncMutex::new(0),
        }
    }

    /// Creates a hopper minecart entity from saved data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            entity_data: SyncMutex::new(MinecartEntityData::new()),
            minecart_base: AbstractMinecartBase::new(),
            container: SimpleContainer::new(HOPPER_SLOTS).into_shared(),
            enabled: SyncMutex::new(true),
            transfer_cooldown: SyncMutex::new(0),
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
        let inserted = {
            let mut container = self.container.lock();
            let before = stack.count();
            container.add(stack);
            stack.count() != before
        };
        // Minecarts have no block entity behind them, so `setChanged` is a no-op
        // and there is nothing to notify here; the return value is what the
        // caller (block pickup, hopper transfer) uses.
        inserted
    }

    /// Vanilla `MinecartHopper.suckItems` container half: pull one item out of
    /// the container directly above the cart, through that container's `DOWN`
    /// face so a furnace only gives what a hopper may take.
    ///
    /// Returns whether a single item moved.
    fn take_from_container_above(&self, source: &ContainerRef) -> bool {
        let source_id = source.container_id();
        let slots_for_down = {
            let guard = ContainerLockGuard::lock_all(&[source]);
            let Some(container) = guard.get(source_id) else {
                return false;
            };
            match container.slots_for_face(Direction::Down) {
                Some(slots) => slots.to_vec(),
                None => (0..container.get_container_size()).collect(),
            }
        };

        for slot in slots_for_down {
            // The source is locked first and the cart's own mutex second; nothing
            // in the codebase takes them in the other order, so the pair cannot
            // deadlock. Every borrow is scoped so the guard is free again before
            // the source is mutated.
            let moved = {
                let guard = ContainerLockGuard::lock_all(&[source]);
                let Some(container) = guard.get(source_id) else {
                    return false;
                };
                let stack = container.get_item(slot);
                if stack.is_empty()
                    || !container.can_take_item_through_face(slot, stack, Direction::Down)
                {
                    false
                } else {
                    let mut cart = self.container.lock();
                    // Vanilla passes the cart as `destination` so a source can
                    // refuse a transfer that cannot fit (`Container::canTakeItem`).
                    if container.can_take_item(&*cart, slot, stack) {
                        let mut moving = stack.copy_with_count(1);
                        Container::add(&mut *cart, &mut moving);
                        moving.is_empty()
                    } else {
                        false
                    }
                }
            };
            if !moved {
                continue;
            }
            let mut guard = ContainerLockGuard::lock_all(&[source]);
            {
                let Some(container) = guard.get_mut(source_id) else {
                    return false;
                };
                container.get_item_mut(slot).shrink(1);
                if container.get_item(slot).count() <= 0 {
                    *container.get_item_mut(slot) = ItemStack::empty();
                }
            }
            let _ = guard.set_changed(source_id);
            return true;
        }
        false
    }

    /// Vanilla `MinecartHopper.suckItems`: pull nearby item entities in.
    ///
    /// The scan uses `World::get_entities_in_aabb_matching` so only item
    /// entities are considered, and the box is the cart's own bounding box
    /// grown by `1.0` horizontally and `0.5` vertically, exactly like
    /// vanilla's `Hopper.SUCK_AABB`.
    fn suck_items(self: &Arc<Self>) -> bool {
        if !self.is_enabled() {
            return false;
        }
        let Some(world) = self.level() else {
            return false;
        };
        // Vanilla `MinecartHopper` is not grid-aligned, so its container slot is
        // resolved from `getLevelY() = getY() + 0.5` rather than from the cart's
        // floor: a cart nudged slightly down still reads the block above the
        // cell its middle sits in.
        let cart_pos = BlockPos::from(self.position() + DVec3::new(0.0, 0.5, 0.0));
        let above = cart_pos.above();
        if let Some(source) = world
            .get_block_entity(above)
            .and_then(ContainerRef::from_block_entity)
            && self.take_from_container_above(&source)
        {
            return true;
        }
        let pickup_box = self.bounding_box().inflate_xyz(
            HOPPER_PICKUP_RADIUS,
            HOPPER_PICKUP_HEIGHT,
            HOPPER_PICKUP_RADIUS,
        );
        let candidates = world.get_entities_in_aabb_matching(&pickup_box, |entity| {
            entity.entity_type() == &vanilla_entities::ITEM
        });
        for entity in candidates {
            let Some(item_entity) = entity.as_ref().downcast_ref::<ItemEntity>() else {
                continue;
            };
            let mut moving = item_entity.get_item();
            if moving.is_empty() {
                continue;
            }
            let original = moving.count();
            {
                let mut container = self.container.lock();
                container.add(&mut moving);
            }
            if moving.count() == original {
                // The cart is full for this item; leave the entity alone.
                continue;
            }
            if moving.is_empty() {
                item_entity.set_removed(RemovalReason::Discarded);
            } else {
                item_entity.set_item(moving);
            }
            // Vanilla stops at the first entity it could take something from,
            // because `tryConsumeItems` only moves one item per tick.
            return true;
        }
        false
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
        player: &Player,
        _hand: InteractionHand,
        _location: DVec3,
    ) -> InteractionResult {
        if self.level().is_none() {
            return InteractionResult::Pass;
        }
        let player_inventory = Arc::clone(&player.inventory);
        let container = Arc::clone(&self.container);
        player.open_menu(self.display_name(), move |context| {
            hopper(player_inventory, context.container_id, container)
        });
        self.game_event_with_source_entity(
            &vanilla_game_events::CONTAINER_OPEN,
            Some(player as &dyn Entity),
        );
        InteractionResult::Success
    }

    fn tick(self: Arc<Self>) {
        if let Some(minecart) = self.as_abstract_minecart() {
            minecart.tick_minecart(&(Arc::clone(&self) as SharedEntity));
        }
        // Vanilla `MinecartHopper.tick`: the countdown runs down every tick and
        // the cart only tries to consume an item once it reaches zero, so the
        // eight-tick cooldown is a floor rather than a cadence. `suckItems`
        // itself honours the activator-rail `Enabled` flag.
        let ready = {
            let mut cooldown = self.transfer_cooldown.lock();
            if *cooldown > 0 {
                *cooldown -= 1;
                false
            } else {
                true
            }
        };
        if ready && self.suck_items() {
            *self.transfer_cooldown.lock() = HOPPER_MOVE_COOLDOWN;
        }
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        let container = self.container.lock();
        let mut items: Vec<NbtCompound> = Vec::new();
        for (slot, item) in container.items().iter().enumerate() {
            if !item.is_empty()
                && let NbtTag::Compound(mut item_nbt) = item.clone().to_nbt_tag()
            {
                item_nbt.insert("Slot", i8::try_from(slot).unwrap_or(0));
                items.push(item_nbt);
            }
        }
        nbt.insert("Items", NbtList::Compound(items));
        nbt.insert("Enabled", u8::from(self.is_enabled()));
        // Vanilla persists both fields; a reloaded cart must not start its
        // countdown from zero.
        nbt.insert("TransferCooldown", *self.transfer_cooldown.lock());
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        if let Some(enabled) = nbt.byte_value("Enabled") {
            self.set_enabled(enabled != 0);
        }
        if let Some(cooldown) = nbt.int("TransferCooldown") {
            *self.transfer_cooldown.lock() = cooldown;
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
use simdnbt::borrow::read_compound;
use steel_registry::vanilla_items::STONE;

    use super::*;
use std::io::Cursor;

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
        assert!(cart.items().iter().all(ItemStack::is_empty));
    }

    #[test]
    fn hoppers_accept_items_up_to_their_capacity() {
        init_vanilla_registry();
        let cart = cart();
        let mut stack = ItemStack::new(&STONE).copy_with_count(64);
        assert!(cart.try_insert(&mut stack));
        assert!(cart.items().iter().any(|slot| !slot.is_empty()));
        assert!(stack.is_empty());
    }

    /// Vanilla `MinecartHopper.suckInItems` inflates the cart box by
    /// `(0.25, 0.0, 0.25)`; the block hopper's one-block-high suck column does
    /// not apply, and the old `(1.0, 0.5, 1.0)` reached three blocks out.
    #[test]
    fn the_pickup_box_is_the_vanilla_quarter_block_inflation() {
        assert!((HOPPER_PICKUP_RADIUS - 0.25).abs() < 1e-9);
        assert!((HOPPER_PICKUP_HEIGHT - 0.0).abs() < 1e-9);
    }

    /// Vanilla persists `TransferCooldown`, so a reloaded cart keeps its wait.
    #[test]
    fn transfer_cooldown_is_persisted_in_nbt() {
        init_vanilla_registry();
        let cart = cart();
        *cart.transfer_cooldown.lock() = 5;
        let mut nbt = NbtCompound::new();
        cart.save_additional(&mut nbt);
        let mut bytes = Vec::new();
        nbt.write(&mut bytes);
        let mut cursor = Cursor::new(bytes.as_slice());
        let borrowed = read_compound(&mut cursor).expect("compound");
        let view: simdnbt::borrow::NbtCompound<'_, '_> = (&borrowed).into();
        assert_eq!(view.int("TransferCooldown"), Some(5));
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
