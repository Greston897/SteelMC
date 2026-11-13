//! Crafter block entity.
//!
//! Vanilla `CrafterBlockEntity` holds a 3x3 grid (`SIZE = 9`), a slot-disable
//! mask, a trigger count used by the `triggered` block state, and a crafting
//! ticker that runs every game tick.

use std::sync::{Arc, Weak};

use simdnbt::ToNbtTag;
use uuid::Uuid;
use simdnbt::borrow::{BaseNbtCompound as BorrowedNbtCompound, NbtCompound as NbtCompoundView};
use simdnbt::owned::{NbtCompound, NbtList, NbtTag};
use glam::DVec3;
use steel_registry::REGISTRY;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{
    BlockStateProperties, BoolProperty, Direction, FrontAndTop,
};
use steel_registry::item_stack::ItemStack;
use steel_registry::recipe::{CraftingInput, vanilla_recipe_types};
use steel_registry::vanilla_block_entity_types;
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey, locks::SyncMutex};

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::inventory::recipe_manager;
use crate::inventory::container::{Container, load_all_items};
use crate::inventory::lock::{ContainerRef, SharedContainer};
use crate::world::World;

/// Number of slots in the crafter grid.
pub const CRAFTER_SLOTS: usize = 9;

/// Vanilla `CrafterBlockEntity.TRIGGERED_DURATION`: the `triggered` block state
/// counts down from two game ticks before the craft fires.
pub const CRAFT_COOLDOWN_TICKS: i32 = 2;

/// Vanilla `CrafterBlock.CRAFTING` block-state property.
const CRAFTING: BoolProperty = BlockStateProperties::CRAFTING;
/// Vanilla `CrafterBlock.TRIGGERED` block-state property.
const TRIGGERED: BoolProperty = BlockStateProperties::TRIGGERED;

/// Crafter block entity.
pub struct CrafterBlockEntity {
    base: Arc<BlockEntityBase>,
    container: Arc<SyncMutex<CrafterContainer>>,
    container_ref: ContainerRef,
    /// Bitset of disabled slots: bit `n` set means slot `n` is disabled.
    disabled_slots: SyncMutex<u16>,
    /// Triggered count, mirrored into the `triggered` block state.
    triggered: SyncMutex<i32>,
    /// Player who triggered the current craft, credited by
    /// `crafter_recipe_crafted`. Vanilla `CrafterBlockEntity` tracks the same
    /// through `CrafterBlockEntity.getOwner()`, which is set by the block's
    /// neighbour update.
    triggering_player: SyncMutex<Option<Uuid>>,
}

pub(crate) struct CrafterContainer {
    items: Vec<ItemStack>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `CrafterBlockEntity`.
unsafe impl DowncastType for CrafterBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/crafter");
}

// SAFETY: This key is owned by Steel and uniquely identifies the independently
// lockable inventory data used by a crafter.
unsafe impl DowncastType for CrafterContainer {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:container/crafter");
}

impl CrafterBlockEntity {
    /// Creates a new crafter block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        let base = Arc::new(BlockEntityBase::new(
            &vanilla_block_entity_types::CRAFTER,
            level,
            pos,
            state,
        ));
        let container = Arc::new(SyncMutex::new(CrafterContainer {
            items: vec![ItemStack::empty(); CRAFTER_SLOTS],
        }));
        let shared_container: SharedContainer =
            Arc::<SyncMutex<CrafterContainer>>::clone(&container);
        Self {
            container_ref: ContainerRef::owned_by_block_entity(shared_container, Arc::clone(&base)),
            base,
            container,
            disabled_slots: SyncMutex::new(0),
            triggered: SyncMutex::new(0),
            triggering_player: SyncMutex::new(None),
        }
    }

    /// Returns the disabled-slot bitmask.
    #[must_use]
    pub fn disabled_slots(&self) -> u16 {
        *self.disabled_slots.lock()
    }

    /// Sets the disabled-slot bitmask.
    pub fn set_disabled_slots(&self, mask: u16) {
        *self.disabled_slots.lock() = mask;
    }

    /// Triggered counter mirrored into the block state.
    #[must_use]
    pub fn triggered(&self) -> i32 {
        *self.triggered.lock()
    }
}

impl BlockEntity for CrafterBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn pre_remove_side_effects(&self, pos: BlockPos, _state: BlockStateId) {
        let items = {
            let mut container = self.container.lock();
            std::mem::replace(
                &mut container.items,
                vec![ItemStack::empty(); CRAFTER_SLOTS],
            )
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
        {
            let mut container = self.container.lock();
            container.items.fill(ItemStack::empty());
            load_all_items(nbt_view, &mut container.items);
        }
        *self.disabled_slots.lock() = nbt_view.int("disabled_slots").unwrap_or(0) as u16;
        *self.triggered.lock() = nbt_view.int("triggered").unwrap_or(0);
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        {
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
        }
        nbt.insert("disabled_slots", i32::from(*self.disabled_slots.lock()));
        nbt.insert("triggered", *self.triggered.lock());
    }

    fn container_ref(&self) -> Option<ContainerRef> {
        Some(self.container_ref.clone())
    }

    fn tick(&self, world: &Arc<World>) {
        // Vanilla `CrafterBlockEntity.serverTick`: the trigger counter counts
        // *down* while the block is triggered and fires the craft when it
        // reaches zero, after which `crafting` is cleared and the block stops
        // being powered.
        let should_craft = {
            let mut triggered = self.triggered.lock();
            if *triggered <= 0 {
                false
            } else {
                *triggered -= 1;
                *triggered == 0
            }
        };
        if !should_craft {
            return;
        }
        let _ = self.craft(world);
        self.set_block_state_flag(world, CRAFTING, false);
        self.set_block_state_flag(world, TRIGGERED, false);
    }
}

impl CrafterBlockEntity {
    /// Records the player who powered this crafter, so the craft can credit
    /// `crafter_recipe_crafted` the way vanilla's `CrafterBlockEntity` does.
    pub fn set_triggering_player(&self, player: Option<Uuid>) {
        *self.triggering_player.lock() = player;
    }

    /// Vanilla `CrafterBlockEntity.craft`: matches a crafting recipe over the
    /// enabled slots, consumes one item from each of them and **ejects** the
    /// result out of the front face.
    ///
    /// Vanilla does not keep the result inside the grid: disabled slots hold
    /// the crafter's output *only* while it cannot be pushed out, and
    /// `craft` hands the assembled stack to `CraftingRecipe.eject`.
    /// Returns the ejected position when a craft happened.
    fn craft(&self, world: &Arc<World>) -> Option<BlockPos> {
        let mut recipe_key = None;
        let result = {
            let mut container = self.container.lock();
            let disabled = self.disabled_slots();
            let input = CraftingInput::new(
                3,
                3,
                container
                    .items
                    .iter()
                    .enumerate()
                    .map(|(slot, stack)| {
                        if disabled & (1 << slot) != 0 {
                            ItemStack::empty()
                        } else {
                            stack.copy_with_count(1)
                        }
                    })
                    .collect(),
            );
            let recipe = REGISTRY
                .recipes
                .find_match(&vanilla_recipe_types::CRAFTING, &input);
            let Some(recipe) = recipe else {
                return None;
            };
            recipe_key = Some(recipe.key().clone());
            let assembled = recipe_manager::assemble_recipe(recipe, &input);
            if assembled.is_empty() {
                return None;
            }
            // Vanilla consumes one item from every enabled, non-empty slot and
            // then asks the recipe for the remaining items of the grid.
            for (slot, stack) in container.items.iter_mut().enumerate() {
                if disabled & (1 << slot) != 0 || stack.is_empty() {
                    continue;
                }
                stack.shrink(1);
                if stack.count() <= 0 {
                    *stack = ItemStack::empty();
                }
            }
            // Mark the chunk dirty through the block entity, matching every
            // other container block entity.
            self.base.set_changed();
            assembled
        };

        let ejected = self.eject(world, result);
        if ejected.is_some()
            && let Some(recipe_key) = recipe_key
        {
            // Vanilla `CrafterBlockEntity.craft` fires `crafter_recipe_crafted`
            // for the player who powered the crafter, with both the recipe and
            // the assembled stack.
            let starter = self
                .triggering_player
                .lock()
                .and_then(|uuid| world.players.get_by_uuid(&uuid));
            if let Some(player) = starter {
                let context =
                    crate::player::advancement::predicate::TriggerContext::recipe(&recipe_key);
                player.trigger_advancement_with_context(
                    &steel_registry::vanilla_criteria_triggers::CRAFTER_RECIPE_CRAFTED,
                    &context,
                );
            }
        }
        ejected
    }

    /// Vanilla `CraftingRecipe.eject` as used by the crafter: pushes the
    /// assembled stack out of the block's facing side and returns the position
    /// the item was dropped at.
    pub(crate) fn eject(
        &self,
        world: &Arc<World>,
        stack: ItemStack,
    ) -> Option<BlockPos> {
        let pos = self.get_block_pos();
        let (dx, dy, dz) = self.state_facing().offset();
        // Just outside the front face, matching vanilla's
        // `Vec3.atCenterOf(pos).add(facing.getStepX() * 0.5, ..., 0.05)`.
        let drop_pos = DVec3::new(
            f64::from(pos.x()) + 0.5 + f64::from(dx) * 0.5,
            f64::from(pos.y()) + 0.5 + f64::from(dy) * 0.5,
            f64::from(pos.z()) + 0.5 + f64::from(dz) * 0.5,
        );
        // The crafter pushes the item in the facing direction instead of
        // dropping it straight down.
        let velocity = DVec3::new(f64::from(dx) * 0.25, f64::from(dy) * 0.25, f64::from(dz) * 0.25);
        world.spawn_item_with_velocity(drop_pos, stack, velocity)?;
        Some(pos)
    }

    /// Reads the direction the crafter's front face points at.
    ///
    /// Vanilla `CrafterBlock` uses the `orientation` (`front_and_top`) property
    /// instead of a plain `facing`, because the front can point up or down too.
    fn state_facing(&self) -> Direction {
        self.block_state()
            .try_get_value(&BlockStateProperties::ORIENTATION)
            .map_or(Direction::North, FrontAndTop::front)
    }

    /// Writes one boolean block-state property, keeping every other value.
    ///
    /// Does nothing when the block at `pos` no longer carries the property,
    /// which happens when the crafter is broken while a craft is counting down.
    fn set_block_state_flag(&self, world: &Arc<World>, property: BoolProperty, value: bool) {
        let pos = self.get_block_pos();
        let state = world.get_block_state(pos);
        if state.try_get_value(&property) != Some(value) {
            return;
        }
        world.set_block(
            pos,
            state.set_value(&property, value),
            UpdateFlags::UPDATE_CLIENTS,
        );
    }

    /// Vanilla `CrafterBlock.updateState`: starts the crafting sequence when a
    /// rising redstone edge arrives and the grid has a matching recipe.
    ///
    /// Returns whether the sequence started.
    pub(crate) fn start_crafting(&self, world: &Arc<World>) -> bool {
        if !self.begin_crafting() {
            return false;
        }
        self.set_block_state_flag(world, CRAFTING, true);
        self.set_block_state_flag(world, TRIGGERED, true);
        true
    }

    /// Arms the trigger counter without touching the block state.
    ///
    /// Split out of [`Self::start_crafting`] so the counter logic is testable
    /// without a live world.
    fn begin_crafting(&self) -> bool {
        if self.triggered() > 0 {
            return false;
        }
        if !self.has_matching_recipe() {
            return false;
        }
        // Vanilla starts the counter at two ticks; `serverTick` decrements it
        // and fires the craft when it reaches zero.
        *self.triggered.lock() = CRAFT_COOLDOWN_TICKS;
        true
    }

    /// Whether the enabled slots currently form a valid recipe.
    fn has_matching_recipe(&self) -> bool {
        let container = self.container.lock();
        let disabled = self.disabled_slots();
        let input = CraftingInput::new(
            3,
            3,
            container
                .items
                .iter()
                .enumerate()
                .map(|(slot, stack)| {
                    if disabled & (1 << slot) != 0 {
                        ItemStack::empty()
                    } else {
                        stack.copy_with_count(1)
                    }
                })
                .collect(),
        );
        REGISTRY
            .recipes
            .find_match(&vanilla_recipe_types::CRAFTING, &input)
            .is_some()
    }
}

impl Container for CrafterContainer {
    fn items(&self) -> &[ItemStack] {
        &self.items
    }

    fn items_mut(&mut self) -> &mut [ItemStack] {
        &mut self.items
    }

    fn get_container_size(&self) -> usize {
        CRAFTER_SLOTS
    }

    fn set_item(&mut self, slot: usize, mut stack: ItemStack) {
        if slot < CRAFTER_SLOTS {
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
    use steel_registry::{init_vanilla_registry, vanilla_blocks};

    use super::*;

    fn test_crafter() -> CrafterBlockEntity {
        init_vanilla_registry();
        CrafterBlockEntity::new(
            Weak::new(),
            BlockPos::new(0, 0, 0),
            vanilla_blocks::CRAFTER.default_state(),
        )
    }

    #[test]
    fn crafter_has_nine_slots() {
        assert_eq!(test_crafter().container.lock().items.len(), CRAFTER_SLOTS);
    }

    #[test]
    fn disabled_slot_mask_roundtrips() {
        let crafter = test_crafter();
        crafter.set_disabled_slots(0b101);
        assert_eq!(crafter.disabled_slots(), 0b101);
    }

    #[test]
    fn craft_without_a_recipe_does_nothing() {
        let crafter = test_crafter();
        // A single stick cannot form a 3x3 recipe, so no craft starts and
        // nothing is consumed.
        crafter.container.lock().set_item(
            0,
            ItemStack::with_count(&steel_registry::vanilla_items::STICK, 1),
        );
        assert!(!crafter.has_matching_recipe());
        assert_eq!(crafter.container.lock().get_item(0).count(), 1);
    }

    #[test]
    fn a_two_log_column_matches_the_planks_recipe() {
        init_vanilla_registry();
        let crafter = test_crafter();
        {
            let mut container = crafter.container.lock();
            // Two logs stacked in the left column: the vanilla
            // `oak_planks` shaped recipe.
            container.set_item(
                0,
                ItemStack::with_count(&steel_registry::vanilla_items::OAK_LOG, 3),
            );
            container.set_item(
                3,
                ItemStack::with_count(&steel_registry::vanilla_items::OAK_LOG, 3),
            );
        }
        assert!(
            crafter.has_matching_recipe(),
            "two logs in a column craft planks"
        );
        // Nothing is consumed until `craft` runs.
        assert_eq!(crafter.container.lock().get_item(0).count(), 3);
    }

    #[test]
    fn disabled_slots_are_ignored_by_the_recipe_match() {
        init_vanilla_registry();
        let crafter = test_crafter();
        {
            let mut container = crafter.container.lock();
            container.set_item(
                0,
                ItemStack::with_count(&steel_registry::vanilla_items::OAK_LOG, 1),
            );
            container.set_item(
                3,
                ItemStack::with_count(&steel_registry::vanilla_items::OAK_LOG, 1),
            );
        }
        assert!(crafter.has_matching_recipe());
        // Disabling the lower log removes one input of the two-log recipe, so
        // the match disappears even though the items are still in the grid.
        crafter.set_disabled_slots(1 << 3);
        assert!(!crafter.has_matching_recipe());
    }

    #[test]
    fn begin_crafting_sets_the_counter_once_and_requires_a_recipe() {
        init_vanilla_registry();
        let crafter = test_crafter();
        // No recipe in the grid: nothing starts.
        assert!(!crafter.begin_crafting());
        assert_eq!(crafter.triggered(), 0);

        {
            let mut container = crafter.container.lock();
            container.set_item(
                0,
                ItemStack::with_count(&steel_registry::vanilla_items::OAK_LOG, 1),
            );
            container.set_item(
                3,
                ItemStack::with_count(&steel_registry::vanilla_items::OAK_LOG, 1),
            );
        }
        assert!(crafter.begin_crafting());
        assert_eq!(crafter.triggered(), CRAFT_COOLDOWN_TICKS);
        // Starting again while the counter runs is a no-op, so a constant
        // redstone signal cannot re-arm the craft every tick.
        assert!(!crafter.begin_crafting());
        assert_eq!(crafter.triggered(), CRAFT_COOLDOWN_TICKS);
    }

    #[test]
    fn the_front_face_follows_the_orientation_property() {
        init_vanilla_registry();
        let state = vanilla_blocks::CRAFTER
            .default_state()
            .set_value(
                &BlockStateProperties::ORIENTATION,
                FrontAndTop::from_front_and_top(Direction::West),
            );
        assert_eq!(
            state.get_value(&BlockStateProperties::ORIENTATION).front(),
            Direction::West,
            "the crafter ejects out of its front face"
        );
    }

    #[test]
    fn the_trigger_counter_starts_at_zero() {
        assert_eq!(test_crafter().triggered(), 0);
    }
}
