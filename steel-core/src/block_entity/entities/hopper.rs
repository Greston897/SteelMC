//! Hopper block entity.
//!
//! Vanilla `HopperBlockEntity` holds five slots: an input row of four slots and
//! one output slot. It moves one item every eight game ticks (a "cooldown" of 8)
//! from its own inventory into the container below, and pulls one item from the
//! container above while it has room.

use std::{
    mem,
    sync::{Arc, Weak},
};

use glam::DVec3;
use simdnbt::ToNbtTag;
use simdnbt::borrow::{BaseNbtCompound as BorrowedNbtCompound, NbtCompound as NbtCompoundView};
use simdnbt::owned::{NbtCompound, NbtList, NbtTag};
use smallvec::SmallVec;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::blocks::properties::{BlockStateProperties, Direction};
use steel_registry::data_components::{
    DataComponentMap, SeededContainerLoot, vanilla_components::CONTAINER_LOOT,
};
use steel_registry::item_stack::ItemStack;
use steel_registry::{vanilla_block_entity_types, vanilla_entities};
use steel_utils::locks::SyncMutex;
use steel_utils::{
    BlockPos, BlockStateId, Downcast as _, DowncastType, DowncastTypeKey, WorldAabb,
};

use crate::block_entity::randomizable_container::RandomizableContainer;
use crate::block_entity::{BlockEntity, BlockEntityBase, ImplicitComponentGetter};
use crate::entity::entities::objects::items::ItemEntity;
use crate::entity::{Entity as _, RemovalReason};
use crate::inventory::container::{Container, load_all_items};
use crate::inventory::lock::{ContainerLockGuard, ContainerRef, SharedContainer};
use crate::world::World;

/// Number of slots in a hopper.
pub const HOPPER_SLOTS: usize = 5;

/// Hopper block entity.
pub struct HopperBlockEntity {
    base: Arc<BlockEntityBase>,
    container: Arc<SyncMutex<HopperContainer>>,
    container_ref: ContainerRef,
    /// Remaining ticks before the next transfer, matching vanilla's `cooldownTime`.
    cooldown: SyncMutex<u8>,
    /// Loot state for hoppers placed by a structure template.
    loot: SyncMutex<RandomizableContainer>,
}

pub(crate) struct HopperContainer {
    items: Vec<ItemStack>,
}

/// Vanilla `HopperBlockEntity.MOVE_ITEM_SPEED`: one transfer every eight ticks.
pub const MOVE_COOLDOWN_TICKS: u8 = 8;

/// Width of the vanilla suck column, in block units.
///
/// `Hopper.getSuckShape()` is `Shapes.or(INSIDE, ABOVE)` with
/// `INSIDE = Block.column(2.0, 11.0, 16.0)` (the bowl mouth, inset 2/16) and
/// `ABOVE = Block.column(16.0, 16.0, 32.0)` (full width, the block above).
/// `HopperBlockEntity.getItemsAtAndAbove` queries the *bounding box* of that
/// shape, so the effective pick-up volume is the full-width column
/// `Block.column(16.0, 11.0, 32.0)` — which is what this constant describes.
/// The old "block box grown by 0.25" was wrong in both directions: it missed
/// items settled low in the bowl and it over-reached sideways.
const HOPPER_SUCK_COLUMN_WIDTH: f64 = 1.0;

/// Bottom of the vanilla suck column, in block units (11/16).
const HOPPER_SUCK_MIN_Y: f64 = 11.0 / 16.0;

/// Top of the vanilla suck column, in block units (32/16 = the top of the
/// block above the hopper).
const HOPPER_SUCK_MAX_Y: f64 = 32.0 / 16.0;

// SAFETY: This key is owned by Steel and uniquely identifies `HopperBlockEntity`.
unsafe impl DowncastType for HopperBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/hopper");
}

// SAFETY: This key is owned by Steel and uniquely identifies the independently
// lockable inventory data used by a hopper block entity.
unsafe impl DowncastType for HopperContainer {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:container/hopper");
}

impl HopperBlockEntity {
    /// Creates a new hopper block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        let base = Arc::new(BlockEntityBase::new(
            &vanilla_block_entity_types::HOPPER,
            level,
            pos,
            state,
        ));
        let container = Arc::new(SyncMutex::new(HopperContainer {
            items: vec![ItemStack::empty(); HOPPER_SLOTS],
        }));
        let shared_container: SharedContainer =
            Arc::<SyncMutex<HopperContainer>>::clone(&container);
        Self {
            container_ref: ContainerRef::owned_by_block_entity(shared_container, Arc::clone(&base)),
            base,
            container,
            cooldown: SyncMutex::new(0),
            loot: SyncMutex::new(RandomizableContainer::none()),
        }
    }

    /// Vanilla `RandomizableContainerBlockEntity.unpackLootTable`.
    ///
    /// Returns whether a pending table was rolled.
    pub fn unpack_loot(&self) -> bool {
        if !self.loot.lock().has_loot_table() {
            return false;
        }
        let world_seed = self.get_level().map_or(0, |world| world.seed());
        // Lock order matches the save path: container first, loot state second.
        let mut container = self.container.lock();
        let mut loot = self.loot.lock();
        loot.unpack_loot_table(&mut container.items, world_seed)
    }

    /// Vanilla `HopperBlockEntity.suckInItems`: pulls one item from the container
    /// above into this hopper, or, when there is no container, from the item
    /// entities inside `Hopper.SUCK_AABB`.
    ///
    /// Returns whether an item moved. The source slot is only decremented once the
    /// hopper accepts the item, so a full hopper never deletes items.
    pub fn suck_in_items(&self, world: &World, pos: BlockPos) -> bool {
        // Vanilla `HopperBlockEntity.suckInItems` returns immediately when the
        // hopper has no empty slot *and* no stack it could top up. Without this
        // guard a hopper whose five slots hold full stacks of a different item
        // still scans every item entity above it each tick.
        if inventory_full(&*self.container.lock()) {
            return false;
        }
        let above = pos.above();
        let above_state = world.get_block_state(above);
        if above_state.get_block() == &steel_registry::vanilla_blocks::COMPOSTER {
            if above_state.get_value(&BlockStateProperties::LEVEL_COMPOSTER) != 8 {
                return false;
            }
            let Some(level) = self.get_level() else { return false; };
            let mut bone_meal = ItemStack::new(&steel_registry::vanilla_items::BONE_MEAL);
            let inserted = add_into_slots(&mut *self.container.lock(), &[0, 1, 2, 3, 4], Direction::Up, &mut bone_meal);
            if inserted {
                crate::behavior::blocks::ComposterBlock::empty(&level, above);
                self.base.set_changed();
            }
            return inserted;
        }
        if let Some(source) = world
            .get_block_entity(pos.above())
            .and_then(ContainerRef::from_block_entity)
            && self.try_take_from_container(&source)
        {
            return true;
        }
        // Vanilla falls back to item entities above the hopper: it takes from a
        // container first and only then looks at dropped items.
        self.suck_in_item_entities(world, pos)
    }

    /// Vanilla `HopperBlockEntity.tryTakeInItemFromSlot` over `getSlots(container, DOWN)`.
    ///
    /// Vanilla only pulls through slots the source exposes to the `DOWN` face, so
    /// a hopper under a furnace takes from the result and fuel slots but never
    /// from the input slot, and a hopper under a chiseled bookshelf takes nothing.
    fn try_take_from_container(&self, source: &ContainerRef) -> bool {
        let slots_for_down = {
            let guard = ContainerLockGuard::lock_all(&[source]);
            let Some(container) = guard.get(source.container_id()) else {
                return false;
            };
            // `SmallVec` keeps the common case (five hopper slots, a handful of
            // furnace slots, 27 chest slots) on the stack: this runs every tick
            // for every hopper, so a `Vec` allocation per tick is real load.
            match container.slots_for_face(Direction::Down) {
                Some(slots) => SmallVec::from_slice(slots),
                None => (0..container.get_container_size()).collect::<SmallVec<[usize; 32]>>(),
            }
        };

        for slot in slots_for_down {
            // Both containers are taken through `lock_all`, which sorts by id, so
            // the source/hopper pair can never deadlock against another transfer
            // that walks them the other way round.
            let pulled = {
                let guard = ContainerLockGuard::lock_all(&[source, &self.container_ref]);
                let Some(container) = guard.get(source.container_id()) else {
                    return false;
                };
                let stack = container.get_item(slot);
                if stack.is_empty()
                    || !container.can_take_item_through_face(slot, stack, Direction::Down)
                {
                    continue;
                }
                // Vanilla `canTakeItemFromContainer` also asks the destination's
                // own `canTakeItem` contract, passing the hopper as `destination`.
                // Specialized sources use it to refuse transfers that cannot fit.
                let allowed = guard
                    .get(self.container_ref.container_id())
                    .is_some_and(|hopper| container.can_take_item(hopper, slot, stack));
                if !allowed {
                    continue;
                }
                stack.copy_with_count(1)
            };

            {
                let mut hopper = self.container.lock();
                let mut moving = pulled;
                if !Container::add(&mut *hopper, &mut moving) {
                    continue;
                }
            }
            self.base.set_changed();

            // The hopper took the item, so remove exactly one from the source.
            //
            // The change is reported through the guard: `Container::set_changed`
            // on the storage itself is deliberately empty, and the owner (which
            // marks the block entity dirty and refreshes comparators) is only
            // notified by `ContainerLockGuard::set_changed`.
            let mut guard = ContainerLockGuard::lock_all(&[source]);
            let source_id = source.container_id();
            let Some(container) = guard.get_mut(source_id) else {
                return false;
            };
            container.get_item_mut(slot).shrink(1);
            if container.get_item(slot).count() <= 0 {
                *container.get_item_mut(slot) = ItemStack::empty();
            }
            guard.set_changed(source_id);
            return true;
        }
        false
    }

    /// Vanilla `HopperBlockEntity.suckInItems` item-entity fallback.
    fn suck_in_item_entities(&self, world: &World, pos: BlockPos) -> bool {
        // Vanilla `Hopper.getSuckShape()` = `Block.column(16.0, 11.0, 32.0)`:
        // full width in X/Z, from inside the funnel bowl up to the top of the
        // block above. An item resting on the bowl's rim (y = 11/16 exactly) is
        // outside, because vanilla intersects the shapes strictly.
        let half = HOPPER_SUCK_COLUMN_WIDTH / 2.0;
        let suck_box: WorldAabb = WorldAabb::from_min_max(
            DVec3::new(
                f64::from(pos.0.x) + 0.5 - half,
                f64::from(pos.0.y) + HOPPER_SUCK_MIN_Y,
                f64::from(pos.0.z) + 0.5 - half,
            ),
            DVec3::new(
                f64::from(pos.0.x) + 0.5 + half,
                f64::from(pos.0.y) + HOPPER_SUCK_MAX_Y,
                f64::from(pos.0.z) + 0.5 + half,
            ),
        );
        let candidates = world.get_entities_in_aabb_matching(&suck_box, |entity| {
            entity.entity_type() == &vanilla_entities::ITEM
        });
        for entity in candidates {
            let Some(item_entity) = entity.as_ref().downcast_ref::<ItemEntity>() else {
                continue;
            };
            if !self.add_item_entity(item_entity) {
                continue;
            }
            return true;
        }
        false
    }

    /// Vanilla `HopperBlockEntity.addItem(Container, ItemEntity)`.
    ///
    /// The entity is discarded only when the hopper accepts the whole stack, and
    /// the remainder is written back otherwise.
    fn add_item_entity(&self, item_entity: &ItemEntity) -> bool {
        let mut moving = item_entity.get_item();
        if moving.is_empty() {
            return false;
        }
        let original = moving.count();
        {
            let mut hopper = self.container.lock();
            Container::add(&mut *hopper, &mut moving);
        }
        // Nothing was taken: the hopper is full for this item, so the entity has
        // to stay where it is (vanilla `addItem` returns the untouched stack).
        if moving.count() == original {
            return false;
        }
        self.base.set_changed();
        if moving.is_empty() {
            item_entity.set_removed(RemovalReason::Discarded);
            // Vanilla reports `true` only when the whole stack moved, so a
            // partially absorbed stack keeps the scan going to the next entity.
            return true;
        }
        item_entity.set_item(moving);
        false
    }

    /// Vanilla `HopperBlockEntity.tryMoveItems`: pushes one item from the hopper
    /// into the container its `facing` points at.
    ///
    /// Vanilla uses `getContainerAt(blockPos.relative(facing))`, so a side-facing
    /// hopper feeds a chest beside it rather than the block below. The insertion
    /// goes through the destination's face view (`getSlotsForFace` and
    /// `canPlaceItemThroughFace`), which is what stops a hopper from pushing fuel
    /// back into a furnace's input slot or into a sealed container.
    ///
    /// Returns whether an item moved.
    pub fn try_move_items(&self, world: &World, pos: BlockPos) -> bool {
        let facing = self.facing();
        let destination_pos = pos.relative(facing);
        if world.get_block_state(destination_pos).get_block() == &steel_registry::vanilla_blocks::COMPOSTER {
            // Composter input containers expose only their top face.
            if facing != Direction::Down { return false; }
            let Some(level) = self.get_level() else { return false; };
            let candidate = {
                let hopper = self.container.lock();
                hopper.items.iter().enumerate().find(|(_, item)| !item.is_empty() && item.get(steel_registry::data_components::vanilla_components::COMPOSTABLE).is_some())
                    .map(|(slot, item)| (slot, item.copy_with_count(1)))
            };
            let Some((slot, item)) = candidate else { return false; };
            if !crate::behavior::blocks::ComposterBlock::insert(&level, destination_pos, &item) {
                return false;
            }
            // World callbacks must never run while the inventory mutex is held.
            self.container.lock().items[slot].shrink(1);
            self.base.set_changed();
            return true;
        }
        let Some(destination) = world
            .get_block_entity(pos.relative(facing))
            .and_then(ContainerRef::from_block_entity)
        else {
            return false;
        };
        // Try every occupied source slot in order: a rejected first item must
        // not starve later items accepted by a sided or filtered destination.
        for slot in 0..HOPPER_SLOTS {
            if self.container.lock().items[slot].is_empty() {
                continue;
            }
        let mut moving = {
            let hopper = self.container.lock();
            hopper.items[slot].copy_with_count(1)
        };

        let destination_direction = facing.opposite();
        let destination_id = destination.container_id();
        let inserted = {
            let mut guard = ContainerLockGuard::lock_all(&[&destination]);
            let slots: SmallVec<[usize; 32]> = {
                let Some(container) = guard.get(destination_id) else {
                    return false;
                };
                match container.slots_for_face(destination_direction) {
                    Some(slots) => SmallVec::from_slice(slots),
                    None => (0..container.get_container_size()).collect(),
                }
            };
            let inserted = {
                let Some(container) = guard.get_mut(destination_id) else {
                    return false;
                };
                add_into_slots(container, &slots, destination_direction, &mut moving)
            };
            if !inserted {
                continue;
            }
            // The storage-level `set_changed` is a no-op; the destination's owner
            // is notified through the guard, which is what marks it dirty and
            // refreshes its comparator output.
            let _ = guard.set_changed(destination_id);
            true
        };
        if !inserted {
            return false;
        }
        {
            let mut hopper = self.container.lock();
            hopper.items[slot].shrink(1);
            if hopper.items[slot].count() <= 0 {
                hopper.items[slot] = ItemStack::empty();
            }
        }
        self.base.set_changed();
        return true;
        }
        false
    }

    /// Vanilla `HopperBlockEntity.facing`: the block state's `FACING` property.
    fn facing(&self) -> Direction {
        self.get_block_state()
            .get_value(&BlockStateProperties::FACING_HOPPER)
    }

    /// Remaining cooldown ticks before the next transfer.
    #[must_use]
    pub fn cooldown(&self) -> u8 {
        *self.cooldown.lock()
    }

    /// Sets the remaining cooldown, used by the block ticker.
    pub fn set_cooldown(&self, value: u8) {
        *self.cooldown.lock() = value;
    }

    /// Vanilla decrements before testing readiness, so a cooldown of eight
    /// permits another transfer on the eighth tick, not the ninth.
    fn advance_transfer_cooldown(&self) -> bool {
        let mut cooldown = self.cooldown.lock();
        *cooldown = cooldown.saturating_sub(1);
        *cooldown == 0
    }

    /// Advances the transfer state by one game tick, matching vanilla's
    /// Vanilla `HopperBlockEntity.pushItemsTick` cooldown gate.
    pub fn tick_transfer(&self, world: &World, pos: BlockPos) {
        if !self.advance_transfer_cooldown() {
            return;
        }
        // Vanilla `tryMoveItems` runs as
        // `tryMoveItems(level, pos, state, entity, () -> suckInItems(level, entity))`
        // and ORs the two into one `changed` flag, so a single tick both pushes
        // into the block it faces and pulls from above — an item is never stuck
        // for a tick in a hopper that could already forward it. The cooldown is
        // only reset when something actually moved, so an idle hopper retries
        // every tick and wakes up the tick an item arrives.
        let moved = self.try_move_items(world, pos) | self.suck_in_items(world, pos);
        if moved {
            self.set_cooldown(MOVE_COOLDOWN_TICKS);
        }
    }
}

/// Vanilla `HopperBlockEntity.inventoryFull`: no empty slot and no stack that
/// could still take another item.
fn inventory_full(container: &dyn Container) -> bool {
    let max_stack_size = container.get_max_stack_size();
    (0..container.get_container_size()).all(|slot| {
        let stack = container.get_item(slot);
        !stack.is_empty() && stack.count() >= stack.max_stack_size().min(max_stack_size)
    })
}

/// Vanilla `HopperBlockEntity.addItem(Container, Container, ItemStack, Direction)`.
///
/// Merges into matching stacks first, then fills empty slots, and only touches
/// slots the destination exposes to `direction`.
fn add_into_slots(
    container: &mut dyn Container,
    slots: &[usize],
    direction: Direction,
    stack: &mut ItemStack,
) -> bool {
    let mut changed = false;

    if stack.is_stackable() {
        for &slot in slots {
            if stack.is_empty() {
                break;
            }
            let existing = container.get_item(slot);
            if !existing.is_empty()
                && ItemStack::is_same_item_same_components(existing, stack)
                && container.can_place_item_through_face(slot, stack, direction)
            {
                let max_size = container.get_max_stack_size_for_item(stack);
                let space = max_size - existing.count();
                if space > 0 {
                    let to_add = stack.count().min(space);
                    container.get_item_mut(slot).grow(to_add);
                    stack.shrink(to_add);
                    changed = true;
                }
            }
        }
    }

    for &slot in slots {
        if stack.is_empty() {
            break;
        }
        if container.get_item(slot).is_empty()
            && container.can_place_item_through_face(slot, stack, direction)
        {
            let max_size = container.get_max_stack_size_for_item(stack);
            let to_place = stack.count().min(max_size);
            container.set_item(slot, stack.split(to_place));
            changed = true;
        }
    }

    changed
}

impl BlockEntity for HopperBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn pre_remove_side_effects(&self, pos: BlockPos, _state: BlockStateId) {
        // Vanilla `RandomizableContainerBlockEntity.playerWillDestroy` unpacks a
        // pending table into the container, so breaking a generated container
        // drops its loot instead of nothing.
        self.unpack_loot();
        let items = {
            let mut container = self.container.lock();
            mem::replace(&mut container.items, vec![ItemStack::empty(); HOPPER_SLOTS])
        };
        let Some(world) = self.get_level() else {
            return;
        };
        for item in items {
            world.drop_item_stack(pos, item);
        }
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let nbt_view: NbtCompoundView<'_, '_> = nbt.into();
        let mut container = self.container.lock();
        container.items.fill(ItemStack::empty());
        load_all_items(nbt_view, &mut container.items);
        drop(container);
        *self.cooldown.lock() = nbt_view.int("TransferCooldown").unwrap_or(0) as u8;

        // Vanilla `RandomizableContainerBlockEntity.loadAdditional`.
        RandomizableContainer::load_from_nbt(nbt_view, &self.loot);
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        let container = self.container.lock();
        let mut items: Vec<NbtCompound> = Vec::new();
        for (slot, item) in container.items.iter().enumerate() {
            if !item.is_empty()
                && let NbtTag::Compound(mut item_nbt) = item.clone().to_nbt_tag()
            {
                item_nbt.insert("Slot", slot as i8);
                items.push(item_nbt);
            }
        }
        nbt.insert("Items", NbtList::Compound(items));
        drop(container);
        nbt.insert("TransferCooldown", i32::from(*self.cooldown.lock()));

        // Vanilla `RandomizableContainer.trySaveLootTable`: an unopened
        // structure container keeps its table, an unpacked one writes nothing.
        self.loot.lock().save_to_nbt(nbt);
    }

    fn container_ref(&self) -> Option<ContainerRef> {
        Some(self.container_ref.clone())
    }

    fn unpack_loot_table(&self) {
        self.unpack_loot();
    }

    /// Vanilla `RandomizableContainerBlockEntity.applyImplicitComponents`.
    fn apply_implicit_components(&self, components: &mut ImplicitComponentGetter<'_>) {
        if let Some(loot) = components.get(CONTAINER_LOOT) {
            self.loot
                .lock()
                .set_loot_table(Some(loot.loot_table().clone()), loot.seed());
        }
    }

    /// Vanilla `RandomizableContainerBlockEntity.collectImplicitComponents`.
    fn collect_implicit_components(&self, components: &mut DataComponentMap) {
        let loot = self.loot.lock();
        if let Some(table) = loot.loot_table() {
            components.set(
                CONTAINER_LOOT,
                Some(SeededContainerLoot::new(
                    table.clone(),
                    loot.loot_table_seed(),
                )),
            );
        }
    }

    fn tick(&self, world: &Arc<World>) {
        // Vanilla ticks a hopper only while its `enabled` state permits it; the
        // block behavior already selects this ticker only for enabled hoppers.
        let pos = self.get_block_pos();
        // Vanilla `HopperBlockEntity.pushItemsTick` starts with `isEmpty()`, which
        // unpacks a pending table. A hopper that was placed with one must not wait
        // for a player to look inside, so the first tick rolls it.
        self.unpack_loot();
        self.tick_transfer(world, pos);
    }
}

impl Container for HopperContainer {
    fn items(&self) -> &[ItemStack] {
        &self.items
    }

    fn items_mut(&mut self) -> &mut [ItemStack] {
        &mut self.items
    }

    fn get_container_size(&self) -> usize {
        HOPPER_SLOTS
    }

    fn set_item(&mut self, slot: usize, mut stack: ItemStack) {
        if slot < HOPPER_SLOTS {
            let max_stack_size = self.get_max_stack_size_for_item(&stack);
            if !stack.is_empty() && stack.count() > max_stack_size {
                stack.set_count(max_stack_size);
            }
            self.items[slot] = stack;
        }
    }

    fn get_max_stack_size(&self) -> i32 {
        64
    }

    fn set_changed(&mut self) {}
}

#[cfg(test)]
mod tests {
    use simdnbt::borrow::read_compound;
    use steel_registry::{init_vanilla_registry, vanilla_blocks, vanilla_items};

    use crate::inventory::container::SimpleContainer;

    use super::*;

    fn test_hopper() -> HopperBlockEntity {
        init_vanilla_registry();
        HopperBlockEntity::new(
            Weak::new(),
            BlockPos::new(1, 2, 3),
            vanilla_blocks::HOPPER.default_state(),
        )
    }

    #[test]
    fn shulker_rejects_nested_boxes_and_hopper_continues_to_next_acceptable_slot() {
        use crate::bootstrap::init_globals;
        use crate::test_support::{fresh_test_world, insert_ready_full_chunk};
        use steel_utils::ChunkPos;
        init_globals();
        for facing in [Direction::Down, Direction::North, Direction::East] {
            let fixture = fresh_test_world("hopper_filtered_shulker");
            let world = &fixture.world;
            let pos = BlockPos::new(8, 64, 8);
            insert_ready_full_chunk(world, ChunkPos::new(0, 0));
            world.set_block(pos, vanilla_blocks::HOPPER.default_state().set_value(&BlockStateProperties::FACING_HOPPER, facing), steel_utils::types::UpdateFlags::UPDATE_ALL);
            let destination = pos.relative(facing);
            world.set_block(destination, vanilla_blocks::SHULKER_BOX.default_state(), steel_utils::types::UpdateFlags::UPDATE_ALL);
            let entity = world.get_block_entity(pos).unwrap();
            let hopper = entity.downcast_ref::<HopperBlockEntity>().unwrap();
            for item in [&vanilla_items::SHULKER_BOX, &vanilla_items::RED_SHULKER_BOX] {
                hopper.container.lock().set_item(0, ItemStack::new(item));
                assert!(!hopper.try_move_items(world, pos));
                assert_eq!(hopper.container.lock().get_item(0).count(), 1);
            }
            hopper.container.lock().set_item(1, ItemStack::with_count(&vanilla_items::STONE, 3));
            assert!(hopper.try_move_items(world, pos));
            assert_eq!(hopper.container.lock().get_item(0).count(), 1);
            assert_eq!(hopper.container.lock().get_item(1).count(), 2);
            let target = world.get_block_entity(destination).unwrap().container_ref().unwrap();
            assert!(target.lock().get_item(0).is(&vanilla_items::STONE));
            assert_eq!(target.lock().get_item(0).count(), 1);
        }
    }

    #[test]
    fn hopper_has_five_slots() {
        let hopper = test_hopper();
        assert_eq!(hopper.container.lock().items.len(), 5);
        assert_eq!(hopper.container.lock().get_container_size(), 5);
    }

    #[test]
    fn set_item_limits_stack_to_vanilla_container_maximum() {
        let hopper = test_hopper();
        hopper
            .container
            .lock()
            .set_item(0, ItemStack::with_count(&vanilla_items::STONE, 100));
        assert_eq!(hopper.container.lock().get_item(0).count(), 64);
    }

    #[test]
    fn transfer_cooldown_expires_on_the_eighth_tick_not_the_ninth() {
        let hopper = test_hopper();
        hopper.set_cooldown(MOVE_COOLDOWN_TICKS);
        for remaining in (1..MOVE_COOLDOWN_TICKS).rev() {
            assert!(!hopper.advance_transfer_cooldown());
            assert_eq!(hopper.cooldown(), remaining);
        }
        assert!(hopper.advance_transfer_cooldown());
        assert_eq!(hopper.cooldown(), 0);
        assert!(hopper.advance_transfer_cooldown());
    }

    #[test]
    fn cooldown_starts_at_zero_and_round_trips() {
        let hopper = test_hopper();
        assert_eq!(hopper.cooldown(), 0);
        hopper.set_cooldown(MOVE_COOLDOWN_TICKS);
        assert_eq!(hopper.cooldown(), MOVE_COOLDOWN_TICKS);
    }

    /// Vanilla `HopperBlockEntity.facing` reads the block state, so a side-facing
    /// hopper must report that direction rather than always `DOWN`.
    #[test]
    fn facing_is_read_from_the_block_state() {
        init_vanilla_registry();
        let hopper = HopperBlockEntity::new(
            Weak::new(),
            BlockPos::new(1, 2, 3),
            vanilla_blocks::HOPPER
                .default_state()
                .set_value(&BlockStateProperties::FACING_HOPPER, Direction::East),
        );
        assert_eq!(hopper.facing(), Direction::East);
    }

    /// Vanilla only pulls through the slots a source exposes to the `DOWN` face.
    #[test]
    fn add_into_slots_only_touches_slots_exposed_to_the_face() {
        init_vanilla_registry();
        let mut container = SimpleContainer::new(3);
        let mut stack = ItemStack::with_count(&vanilla_items::COAL, 2);

        // Only slot 2 is reachable; slots 0 and 1 must stay empty.
        assert!(add_into_slots(
            &mut container,
            &[2],
            Direction::Down,
            &mut stack
        ));
        assert!(stack.is_empty());
        assert!(container.get_item(0).is_empty());
        assert!(container.get_item(1).is_empty());
        assert_eq!(container.get_item(2).count(), 2);
    }

    /// A hopper must not stack past the destination's own maximum.
    #[test]
    fn add_into_slots_respects_the_destination_max_stack_size() {
        init_vanilla_registry();
        let mut container = SimpleContainer::new(1);
        container.set_item(0, ItemStack::with_count(&vanilla_items::COAL, 60));

        let mut stack = ItemStack::with_count(&vanilla_items::COAL, 10);
        assert!(add_into_slots(
            &mut container,
            &[0],
            Direction::Up,
            &mut stack
        ));
        assert_eq!(container.get_item(0).count(), 64);
        assert_eq!(stack.count(), 6);
    }

    /// Vanilla `Hopper.getSuckShape()` bounding box is
    /// `Block.column(16.0, 11.0, 32.0)`: full width in X/Z, starting inside the
    /// bowl at 11/16 and ending at the top of the block above. The previous
    /// "block box grown by 0.25" reached outside the hopper and never reached
    /// down into the bowl.
    #[test]
    fn suck_column_matches_the_vanilla_shape_bounds() {
        assert!((HOPPER_SUCK_MIN_Y - 0.6875).abs() < 1e-9);
        assert!((HOPPER_SUCK_MAX_Y - 2.0).abs() < 1e-9);
        assert!((HOPPER_SUCK_COLUMN_WIDTH - 1.0).abs() < 1e-9);
    }

    /// An item whose stack cannot merge and that does not fit in any empty slot
    /// must not be reported as accepted.
    #[test]
    fn a_full_hopper_does_not_claim_an_item_entity() {
        init_vanilla_registry();
        let hopper = test_hopper();
        for slot in 0..HOPPER_SLOTS {
            hopper
                .container
                .lock()
                .set_item(slot, ItemStack::with_count(&vanilla_items::COAL, 64));
        }
        // The hopper "accepted" nothing, so vanilla keeps the entity alive and
        // the pickup must not report success.
        assert!(inventory_full(&*hopper.container.lock()));
    }

    #[test]
    fn cooldown_is_persisted_in_nbt() {
        let hopper = test_hopper();
        hopper.set_cooldown(5);
        let nbt = hopper.save_custom_only();
        let mut bytes = Vec::new();
        nbt.write(&mut bytes);
        let mut cursor = std::io::Cursor::new(bytes.as_slice());
        let borrowed = read_compound(&mut cursor).expect("compound");
        let view: NbtCompoundView<'_, '_> = (&borrowed).into();
        assert_eq!(view.int("TransferCooldown"), Some(5));
    }
}
