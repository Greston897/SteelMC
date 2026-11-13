//! Brewing stand block entity.
//!
//! Vanilla `BrewingStandBlockEntity` owns four slots: three potion slots
//! (`0..=2`) and a fuel slot (`3`). Brewing costs 20 ticks per potion and one
//! blaze powder per operation; each powder fuels 20 operations.

use std::{
    mem,
    sync::{Arc, Weak},
};

use simdnbt::ToNbtTag;
use simdnbt::borrow::{BaseNbtCompound as BorrowedNbtCompound, NbtCompound as NbtCompoundView};
use simdnbt::owned::{NbtCompound, NbtList, NbtTag};
use steel_registry::item_stack::ItemStack;
use steel_registry::{level_events, vanilla_block_entity_types};
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey, locks::SyncMutex};

use super::potion_brewing;
use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::inventory::container::{Container, load_all_items};
use crate::inventory::lock::{ContainerRef, SharedContainer};
use crate::world::World;
use steel_registry::vanilla_criteria_triggers;

/// Number of slots in a brewing stand (three potions plus fuel).
pub const BREWING_STAND_SLOTS: usize = 4;

/// Index of the fuel slot, matching vanilla's `BrewingStandBlockEntity.FUEL_SLOT`.
pub const FUEL_SLOT: usize = 3;

/// Number of potion slots, matching `BrewingStandBlockEntity.BREWING_SLOTS`.
pub const POTION_SLOTS: usize = 3;

/// Ticks one brew takes in vanilla (`BREWING_TIME = 400` ticks is the whole
/// fuel burn; one operation is 20 ticks).
pub const BREW_TIME: i32 = 20;

/// Squared vanilla radius (five blocks) within which a brew grants the
/// `brewed_potion` advancement.
const BREW_TRIGGER_RADIUS_SQUARED: f64 = 25.0;

/// Brewing stand state persisted alongside the inventory.
struct BrewingState {
    /// Remaining ticks of the current brew, counting down to zero.
    brew_time: i32,
    /// Remaining fuel units; one unit is one brew operation.
    fuel: i32,
}

/// Brewing stand block entity.
pub struct BrewingStandBlockEntity {
    base: Arc<BlockEntityBase>,
    container: Arc<SyncMutex<BrewingStandContainer>>,
    container_ref: ContainerRef,
    state: SyncMutex<BrewingState>,
    /// Ingredient on top of the stand, matching vanilla's fifth slot.
    ingredient: SyncMutex<ItemStack>,
}

pub(crate) struct BrewingStandContainer {
    items: Vec<ItemStack>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `BrewingStandBlockEntity`.
unsafe impl DowncastType for BrewingStandBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/brewing_stand");
}

// SAFETY: This key is owned by Steel and uniquely identifies the independently
// lockable inventory data used by a brewing stand.
unsafe impl DowncastType for BrewingStandContainer {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:container/brewing_stand");
}

impl BrewingStandBlockEntity {
    /// Creates a new brewing stand block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        let base = Arc::new(BlockEntityBase::new(
            &vanilla_block_entity_types::BREWING_STAND,
            level,
            pos,
            state,
        ));
        let container = Arc::new(SyncMutex::new(BrewingStandContainer {
            items: vec![ItemStack::empty(); BREWING_STAND_SLOTS],
        }));
        let shared_container: SharedContainer =
            Arc::<SyncMutex<BrewingStandContainer>>::clone(&container);
        Self {
            container_ref: ContainerRef::owned_by_block_entity(shared_container, Arc::clone(&base)),
            base,
            container,
            state: SyncMutex::new(BrewingState {
                brew_time: 0,
                fuel: 0,
            }),
            ingredient: SyncMutex::new(ItemStack::empty()),
        }
    }

    /// Remaining fuel units, matching vanilla's `BrewingStandBlockEntity.fuel`.
    #[must_use]
    pub fn fuel(&self) -> i32 {
        self.state.lock().fuel
    }

    /// Remaining ticks of the current brew.
    #[must_use]
    pub fn brew_time(&self) -> i32 {
        self.state.lock().brew_time
    }
}

impl BlockEntity for BrewingStandBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn pre_remove_side_effects(&self, pos: BlockPos, _state: BlockStateId) {
        let items = {
            let mut container = self.container.lock();
            mem::replace(
                &mut container.items,
                vec![ItemStack::empty(); BREWING_STAND_SLOTS],
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
        if let Some(ingredient_nbt) = nbt_view.compound("Ingredient") {
            if let Some(stack) = ItemStack::from_borrowed_compound(&ingredient_nbt) {
                *self.ingredient.lock() = stack;
            }
        }
        let mut state = self.state.lock();
        state.brew_time = nbt_view.int("BrewTime").unwrap_or(0);
        state.fuel = nbt_view.int("Fuel").unwrap_or(0);
        drop(state);
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
        let state = self.state.lock();
        nbt.insert("BrewTime", state.brew_time);
        nbt.insert("Fuel", state.fuel);
        drop(state);
        let ingredient = self.ingredient.lock();
        if !ingredient.is_empty() {
            nbt.insert("Ingredient", ingredient.clone().to_nbt_tag());
        }
    }

    fn container_ref(&self) -> Option<ContainerRef> {
        Some(self.container_ref.clone())
    }

    fn tick(&self, world: &Arc<World>) {
        let pos = self.get_block_pos();
        let mut state = self.state.lock();
        // Vanilla `BrewingStandBlockEntity.doTick`: an active brew counts down and
        // starts a new one whenever fuel and a valid recipe are available.
        if state.brew_time > 0 {
            state.brew_time -= 1;
            if state.brew_time == 0 {
                drop(state);
                self.brew(world, pos);
            }
        } else if state.fuel > 0 && self.can_brew() {
            state.fuel -= 1;
            state.brew_time = BREW_TIME;
        }
    }
}

impl BrewingStandBlockEntity {
    /// Vanilla `BrewingStandBlockEntity.canBrew`: an ingredient is present and at
    /// least one potion slot has a matching recipe.
    fn can_brew(&self) -> bool {
        let container = self.container.lock();
        let Some(ingredient) = self.ingredient_stack() else {
            return false;
        };
        if !potion_brewing::is_ingredient(&ingredient) {
            return false;
        }
        (0..POTION_SLOTS).any(|slot| {
            let stack = &container.items[slot];
            !stack.is_empty() && potion_brewing::has_recipe(stack, &ingredient)
        })
    }

    /// Returns the ingredient on top of the stand, matching vanilla's separate
    /// ingredient slot (stored outside the four container slots).
    fn ingredient_stack(&self) -> Option<ItemStack> {
        let ingredient = self.ingredient.lock().clone();
        (!ingredient.is_empty()).then_some(ingredient)
    }

    /// Vanilla `BrewingStandBlockEntity.doBrew`: applies the recipe to every
    /// matching potion slot and consumes one ingredient.
    fn brew(&self, world: &Arc<World>, pos: BlockPos) {
        let mut container = self.container.lock();
        let Some(ingredient) = self.ingredient_stack() else {
            return;
        };
        let mut brewed = false;
        for slot in 0..POTION_SLOTS {
            let stack = container.items[slot].clone();
            if stack.is_empty() {
                continue;
            }
            if let Some(result) = potion_brewing::mix(&stack, &ingredient) {
                let mut result_stack = stack.clone();
                result_stack.set_potion(&result);
                container.items[slot] = result_stack;
                brewed = true;
            }
        }
        container.set_changed();
        drop(container);
        if brewed {
            let mut ingredient = self.ingredient.lock();
            ingredient.shrink(1);
            if ingredient.count() <= 0 {
                *ingredient = ItemStack::empty();
            }
            drop(ingredient);
            world.level_event(level_events::SOUND_BREWING_STAND_BREW, pos, 0, None);
            // Vanilla `BrewingStandBlockEntity.doTick` triggers `brewed_potion` for
            // every player standing within five blocks of the stand;
            // `nether/brew_potion` and `nether/all_potions` hang off it.
            let (center_x, center_y, center_z) = pos.get_center();
            let center = glam::DVec3::new(center_x, center_y, center_z);
            world.players.iter_players(|_uuid, player| {
                if player.position().distance_squared(center) <= BREW_TRIGGER_RADIUS_SQUARED {
                    player.trigger_advancement(&vanilla_criteria_triggers::BREWED_POTION);
                }
                true
            });
        }
    }
}

impl Container for BrewingStandContainer {
    fn items(&self) -> &[ItemStack] {
        &self.items
    }

    fn items_mut(&mut self) -> &mut [ItemStack] {
        &mut self.items
    }

    fn get_container_size(&self) -> usize {
        BREWING_STAND_SLOTS
    }

    fn set_item(&mut self, slot: usize, mut stack: ItemStack) {
        if slot < BREWING_STAND_SLOTS {
            let max_stack_size = self.get_max_stack_size_for_item(&stack);
            if !stack.is_empty() && stack.count() > max_stack_size {
                stack.set_count(max_stack_size);
            }
            self.items[slot] = stack;
        }
    }

    fn get_max_stack_size(&self) -> i32 {
        // Vanilla restricts the fuel slot to one item and potion slots to one
        // as well, so every slot in a brewing stand behaves as a single slot.
        1
    }

    fn set_changed(&mut self) {}
}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_blocks, vanilla_items};

    use super::*;

    fn test_brewing_stand() -> BrewingStandBlockEntity {
        init_vanilla_registry();
        BrewingStandBlockEntity::new(
            Weak::new(),
            BlockPos::new(1, 2, 3),
            vanilla_blocks::BREWING_STAND.default_state(),
        )
    }

    #[test]
    fn brewing_stand_has_four_slots() {
        let brewing_stand = test_brewing_stand();
        assert_eq!(brewing_stand.container.lock().items.len(), BREWING_STAND_SLOTS);
    }

    #[test]
    fn fuel_slot_holds_one_item() {
        let brewing_stand = test_brewing_stand();
        brewing_stand
            .container
            .lock()
            .set_item(FUEL_SLOT, ItemStack::with_count(&*vanilla_items::BLAZE_POWDER, 5));
        assert_eq!(brewing_stand.container.lock().get_item(FUEL_SLOT).count(), 1);
    }

    #[test]
    fn state_defaults_to_idle() {
        let brewing_stand = test_brewing_stand();
        assert_eq!(brewing_stand.brew_time(), 0);
        assert_eq!(brewing_stand.fuel(), 0);
    }

    #[test]
    fn brew_time_and_fuel_round_trip() {
        let brewing_stand = test_brewing_stand();
        {
            let mut state = brewing_stand.state.lock();
            state.brew_time = 12;
            state.fuel = 7;
        }
        assert_eq!(brewing_stand.brew_time(), 12);
        assert_eq!(brewing_stand.fuel(), 7);
    }

    #[test]
    fn ingredient_starts_empty() {
        let brewing_stand = test_brewing_stand();
        assert!(brewing_stand.ingredient.lock().is_empty());
    }

    #[test]
    fn brewing_requires_fuel_and_a_recipe() {
        let brewing_stand = test_brewing_stand();
        assert!(!brewing_stand.can_brew(), "an empty stand cannot brew");

        // Nether wart over a water bottle is a valid recipe, but without fuel the
        // ticker will not start a brew.
        let mut water = ItemStack::new(&steel_registry::vanilla_items::POTION);
        water.set_potion(&steel_utils::Identifier::new("minecraft", "water"));
        brewing_stand.container.lock().set_item(0, water);
        *brewing_stand.ingredient.lock() =
            ItemStack::with_count(&steel_registry::vanilla_items::NETHER_WART, 1);
        assert!(brewing_stand.can_brew());
        assert_eq!(brewing_stand.fuel(), 0);
    }
}
