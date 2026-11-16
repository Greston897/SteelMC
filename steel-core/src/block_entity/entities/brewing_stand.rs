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
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::blocks::properties::{BlockStateProperties, Direction};
use steel_registry::data_components::vanilla_components;
use steel_registry::item_stack::ItemStack;
use steel_registry::{level_events, vanilla_block_entity_types};
use steel_utils::nbt::NbtValueInput as _;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey, locks::SyncMutex};

use super::potion_brewing;
use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::entity::Entity as _;
use crate::inventory::container::{Container, load_all_items};
use crate::inventory::lock::{ContainerRef, SharedContainer};
use crate::world::World;
use steel_registry::vanilla_criteria_triggers;
use steel_utils::types::UpdateFlags;

/// Number of slots in a brewing stand: three bottles, the ingredient and fuel.
///
/// Vanilla `BrewingStandBlockEntity` has five slots (`0..=2` bottles, `3`
/// ingredient, `4` fuel), not four. The four-slot layout kept the ingredient
/// outside the container, which made the brewing menu (41 slots: 5 container +
/// 36 player) disagree with the block entity it mirrors.
pub const BREWING_STAND_SLOTS: usize = 5;

/// Index of the fuel slot, matching vanilla's `BrewingStandBlockEntity.FUEL_SLOT`.
pub const FUEL_SLOT: usize = 4;

/// Index of the ingredient slot, matching vanilla's `INGREDIENT_SLOT`.
pub const INGREDIENT_SLOT: usize = 3;

/// Number of potion slots, matching `BrewingStandBlockEntity.BREWING_SLOTS`.
pub const POTION_SLOTS: usize = 3;

/// Ticks one brew takes, matching vanilla `BrewingStandBlockEntity.BREW_TIME`.
///
/// A single brew is 400 ticks (20 seconds); the old value of 20 made potions
/// brew twenty times too fast.
pub const BREW_TIME: i32 = 400;

/// Slots a hopper above the stand may fill (vanilla
/// `BrewingStandBlockEntity.SLOTS_FOR_UP`): the ingredient slot only.
const SLOTS_FOR_UP: &[usize] = &[INGREDIENT_SLOT];

/// Slots a hopper below the stand may drain (vanilla `SLOTS_FOR_DOWN`): the
/// three bottle slots.
const SLOTS_FOR_DOWN: &[usize] = &[0, 1, 2];

/// Slots a sideways hopper may fill or drain (vanilla `SLOTS_FOR_SIDES`): the
/// bottles and the fuel.
const SLOTS_FOR_SIDES: &[usize] = &[0, 1, 2, FUEL_SLOT];

/// Vanilla `BrewingStandBlockEntity` default fuel uses.
///
/// `blaze_powder` names `minecraft:brewing/uses_default`, which resolves to 20
/// brews per item.
pub const DEFAULT_BREWING_FUEL_USES: i32 = 20;

/// Vanilla `minecraft:brewing/speed_default` — no speed change.
pub const DEFAULT_BREWING_SPEED_MULTIPLIER: f32 = 1.0;

/// Squared vanilla radius (five blocks) within which a brew grants the
/// `brewed_potion` advancement.
const BREW_TRIGGER_RADIUS_SQUARED: f64 = 25.0;

/// Brewing stand state persisted alongside the inventory.
struct BrewingState {
    /// Remaining ticks of the current brew, counting down to zero.
    brew_time: i32,
    /// Total ticks the current brew was started with (vanilla
    /// `totalBrewTime`), used to scale the progress bar.
    total_brew_time: i32,
    /// Remaining fuel units; one unit is one brew operation.
    fuel: i32,
    /// Total units the last fuel item provided (vanilla `totalFuel`).
    total_fuel: i32,
    /// Brewing-speed multiplier of the fuel in use (vanilla
    /// `speedMultiplier`); `<= 0.0` means "default".
    speed_multiplier: f32,
}

/// Brewing stand block entity.
pub struct BrewingStandBlockEntity {
    base: Arc<BlockEntityBase>,
    container: Arc<SyncMutex<BrewingStandContainer>>,
    container_ref: ContainerRef,
    state: SyncMutex<BrewingState>,
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
                total_brew_time: BREW_TIME,
                fuel: 0,
                total_fuel: 0,
                speed_multiplier: 0.0,
            }),
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

    /// The two values vanilla `BrewingStandMenu` publishes as data slots:
    /// elapsed brew ticks and remaining fuel uses.
    #[must_use]
    pub fn menu_data(&self) -> [i16; 2] {
        let state = self.state.lock();
        [
            state.brew_time.clamp(0, i32::from(i16::MAX)) as i16,
            state.fuel.clamp(0, i32::from(i16::MAX)) as i16,
        ]
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
        let mut state = self.state.lock();
        state.brew_time = nbt_view.int("BrewTime").unwrap_or(0);
        state.fuel = nbt_view.int("Fuel").unwrap_or(0);
        state.total_brew_time = nbt_view.int("total_brew_time").unwrap_or(BREW_TIME);
        state.total_fuel = nbt_view.int("total_fuel").unwrap_or(0);
        state.speed_multiplier = nbt_view.float_value("speed_multiplier").unwrap_or(0.0);
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
        // Vanilla 26.3 persists the elapsed brew time and the totals it scales
        // against; `Fuel` stayed the remaining uses.
        nbt.insert("BrewTime", state.brew_time);
        nbt.insert("Fuel", state.fuel);
        nbt.insert("total_brew_time", state.total_brew_time);
        nbt.insert("total_fuel", state.total_fuel);
        nbt.insert("speed_multiplier", NbtTag::Float(state.speed_multiplier));
    }

    fn container_ref(&self) -> Option<ContainerRef> {
        Some(self.container_ref.clone())
    }

    fn tick(&self, world: &Arc<World>) {
        let pos = self.get_block_pos();
        let state = self.state.lock();
        // Vanilla `BrewingStandBlockEntity.serverTick`.
        //
        // 1. Refuel first: an empty fuel counter takes one item from the fuel
        //    slot and sets `fuel = getUses(...)`, `totalFuel = fuel` and
        //    `speedMultiplier` from the item's `brewing_fuel` component.
        // 2. Then brew: a running brew counts down and finishes at zero, and an
        //    idle stand starts a brew when the ingredient can change at least
        //    one bottle and fuel is available.
        drop(state);

        let needs_refuel = {
            let state = self.state.lock();
            state.fuel <= 0
        };
        if needs_refuel {
            let fuel_stack = self.container.lock().items[FUEL_SLOT].clone();
            let uses = Self::fuel_uses(&fuel_stack);
            if uses > 0 {
                let mut state = self.state.lock();
                state.fuel = uses;
                state.total_fuel = uses;
                state.speed_multiplier = Self::fuel_speed_multiplier(&fuel_stack);
                drop(state);
                // Vanilla consumes the fuel item when it starts a burn and
                // leaves the slot empty (or with the crafting remainder).
                let mut container = self.container.lock();
                container.items[FUEL_SLOT].shrink(1);
                if container.items[FUEL_SLOT].count() <= 0 {
                    container.items[FUEL_SLOT] = ItemStack::empty();
                }
                drop(container);
                self.container_ref.mark_changed();
            }
        }

        let mut state = self.state.lock();
        if state.brew_time > 0 {
            state.brew_time -= 1;
            if state.brew_time == 0 {
                drop(state);
                self.brew(world, pos);
            }
            return;
        }
        drop(state);

        // Vanilla `BrewingStandBlockEntity.serverTick` keeps the block's
        // `has_bottle_0..2` properties in sync with the three bottle slots so the
        // model shows which holders are filled. Steel never updated them, so a
        // stand always rendered as empty.
        self.sync_bottle_properties(world, pos);

        let brewable = self.can_brew();
        let mut state = self.state.lock();
        if brewable && state.fuel > 0 && state.brew_time <= 0 {
            state.fuel -= 1;
            let multiplier = if state.speed_multiplier > 0.0 {
                state.speed_multiplier
            } else {
                1.0
            };
            // Vanilla `Math.ceil(400.0F / speedMultiplier)`.
            state.brew_time = ((BREW_TIME as f32) / multiplier).ceil() as i32;
            state.total_brew_time = state.brew_time;
            drop(state);
            self.container_ref.mark_changed();
        }
    }
}

impl BrewingStandBlockEntity {
    /// Vanilla `BrewingStandBlockEntity.canBrew`: the ingredient slot holds a
    /// brewing ingredient and at least one bottle slot has a matching recipe.
    fn can_brew(&self) -> bool {
        let container = self.container.lock();
        let ingredient = &container.items[INGREDIENT_SLOT];
        if ingredient.is_empty() || !potion_brewing::is_ingredient(ingredient) {
            return false;
        }
        // Vanilla `BrewingStandBlockEntity.canBrew` asks `isBrewable` per bottle,
        // which means the mix has to *change* the bottle. A glass bottle over
        // nether wart is a valid mix (`water` → `awkward`) because the result
        // item is a potion, so the identity check has to compare the produced
        // potion, not just "a recipe exists".
        (0..POTION_SLOTS).any(|slot| Self::is_brewable_stack(&container.items[slot], ingredient))
    }

    /// Vanilla `BrewingStandBlockEntity.isBrewable(ItemStack)`.
    ///
    /// The bottle must be a brewable container and the mix must produce a
    /// different potion than the bottle already holds.
    fn is_brewable_stack(stack: &ItemStack, ingredient: &ItemStack) -> bool {
        if stack.is_empty() || !potion_brewing::is_brewable_potion(stack) {
            return false;
        }
        match potion_brewing::mix(stack, ingredient) {
            // A glass bottle becomes an `awkward` potion, which is a change.
            Some(result) => stack.potion_key().as_ref() != Some(&result),
            None => false,
        }
    }

    /// Vanilla `BrewingStandBlockEntity.getUses`: how many brews a fuel item
    /// provides.
    ///
    /// The value comes from the item's `minecraft:brewing_fuel` component, whose
    /// `uses` field names a `number_provider`; every vanilla provider in use is
    /// the default of 20 (`blaze_powder`). Steel has no `number_provider`
    /// registry yet, so the default is returned for any item that carries the
    /// component and zero otherwise.
    fn fuel_uses(stack: &ItemStack) -> i32 {
        if stack.is_empty() {
            return 0;
        }
        if stack.has(vanilla_components::BREWING_FUEL) {
            DEFAULT_BREWING_FUEL_USES
        } else {
            0
        }
    }

    /// Vanilla `BrewingStandBlockEntity.getSpeedMultiplier`.
    fn fuel_speed_multiplier(stack: &ItemStack) -> f32 {
        if stack.is_empty() || !stack.has(vanilla_components::BREWING_FUEL) {
            return 0.0;
        }
        DEFAULT_BREWING_SPEED_MULTIPLIER
    }

    /// Mirrors the three bottle slots into the block's `has_bottle_*` states,
    /// as vanilla `BrewingStandBlockEntity.serverTick` does every tick.
    fn sync_bottle_properties(&self, world: &Arc<World>, pos: BlockPos) {
        let occupancy = {
            let container = self.container.lock();
            [
                !container.items[0].is_empty(),
                !container.items[1].is_empty(),
                !container.items[2].is_empty(),
            ]
        };
        let mut state = self.base.block_state();
        let mut changed = false;
        for (index, property) in [
            BlockStateProperties::HAS_BOTTLE_0,
            BlockStateProperties::HAS_BOTTLE_1,
            BlockStateProperties::HAS_BOTTLE_2,
        ]
        .into_iter()
        .enumerate()
        {
            if state.get_value(&property) != occupancy[index] {
                state = state.set_value(&property, occupancy[index]);
                changed = true;
            }
        }
        if changed {
            world.set_block(pos, state, UpdateFlags::UPDATE_ALL);
        }
    }

    /// Vanilla `BrewingStandBlockEntity.doBrew`: applies every matching recipe,
    /// consumes one ingredient and plays the brew sound.
    fn brew(&self, world: &Arc<World>, pos: BlockPos) {
        let brewed = {
            let mut container = self.container.lock();
            let ingredient = container.items[INGREDIENT_SLOT].clone();
            if ingredient.is_empty() {
                return;
            }
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
            if brewed {
                // Vanilla shrinks the ingredient whether or not a recipe fired,
                // but only after `canBrew` gated the whole operation.
                container.get_item_mut(INGREDIENT_SLOT).shrink(1);
                if container.get_item(INGREDIENT_SLOT).count() <= 0 {
                    *container.get_item_mut(INGREDIENT_SLOT) = ItemStack::empty();
                }
            }
            brewed
        };
        // `Container::set_changed` on the storage is a no-op; the owner is what
        // marks the block dirty and refreshes comparators.
        self.container_ref.mark_changed();
        if !brewed {
            return;
        }
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
        // The ingredient and fuel slots hold a single item; the three bottle
        // slots hold a full stack of one bottle each, exactly as vanilla's
        // `BrewingStandBlockEntity` container does (`getMaxStackSize` is 64 and
        // the potion/ingredient checks are per-slot `canPlaceItem`).
        64
    }

    fn set_changed(&mut self) {}

    /// Vanilla `BrewingStandBlockEntity.canPlaceItem`.
    ///
    /// The bottle slots take only brewable potions and the ingredient slot only
    /// brewing ingredients; the fuel slot accepts anything because the fuel
    /// component decides later whether it can burn.
    fn can_place_item(&self, slot: usize, stack: &ItemStack) -> bool {
        match slot {
            FUEL_SLOT => true,
            INGREDIENT_SLOT => potion_brewing::is_ingredient(stack),
            _ => potion_brewing::is_brewable_potion(stack),
        }
    }

    /// Vanilla `BrewingStandBlockEntity.getSlotsForFace`.
    ///
    /// Hoppers above may only insert into the ingredient slot, side hoppers feed
    /// the three bottle slots and the fuel slot, and a hopper below may pull from
    /// the bottle slots only — the ingredient and the fuel stay behind.
    fn slots_for_face(&self, direction: Direction) -> Option<&'static [usize]> {
        Some(match direction {
            Direction::Up => SLOTS_FOR_UP,
            Direction::Down => SLOTS_FOR_DOWN,
            _ => SLOTS_FOR_SIDES,
        })
    }

    fn can_place_item_through_face(
        &self,
        slot: usize,
        stack: &ItemStack,
        direction: Direction,
    ) -> bool {
        match direction {
            Direction::Up => slot == INGREDIENT_SLOT && self.can_place_item(slot, stack),
            Direction::Down => false,
            _ => slot != INGREDIENT_SLOT && self.can_place_item(slot, stack),
        }
    }

    fn can_take_item_through_face(
        &self,
        slot: usize,
        _stack: &ItemStack,
        direction: Direction,
    ) -> bool {
        direction != Direction::Up && slot < POTION_SLOTS
    }
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
    fn brewing_stand_has_five_slots() {
        let brewing_stand = test_brewing_stand();
        assert_eq!(
            brewing_stand.container.lock().items.len(),
            BREWING_STAND_SLOTS
        );
        // Vanilla layout: 0..=2 bottles, 3 ingredient, 4 fuel.
        assert_eq!(BREWING_STAND_SLOTS, 5);
        assert_eq!(INGREDIENT_SLOT, 3);
        assert_eq!(FUEL_SLOT, 4);
    }

    #[test]
    fn the_fuel_slot_accepts_a_stack_and_the_sides_reach_it() {
        let brewing_stand = test_brewing_stand();
        brewing_stand.container.lock().set_item(
            FUEL_SLOT,
            ItemStack::with_count(&vanilla_items::BLAZE_POWDER, 5),
        );
        assert_eq!(
            brewing_stand.container.lock().get_item(FUEL_SLOT).count(),
            5
        );
        // Vanilla exposes the fuel slot to side hoppers only.
        let container = brewing_stand.container.lock();
        let powder = ItemStack::new(&vanilla_items::BLAZE_POWDER);
        assert!(container.can_place_item_through_face(FUEL_SLOT, &powder, Direction::East));
        assert!(!container.can_place_item_through_face(FUEL_SLOT, &powder, Direction::Up));
        assert!(!container.can_take_item_through_face(FUEL_SLOT, &powder, Direction::Down));
    }

    /// Vanilla `BrewingStandBlockEntity.getSlotsForFace`.
    #[test]
    fn face_slots_match_vanilla() {
        let brewing_stand = test_brewing_stand();
        let container = brewing_stand.container.lock();
        assert_eq!(
            container.slots_for_face(Direction::Up),
            Some(&[INGREDIENT_SLOT][..])
        );
        assert_eq!(
            container.slots_for_face(Direction::Down),
            Some(&[0, 1, 2][..])
        );
        assert_eq!(
            container.slots_for_face(Direction::North),
            Some(&[0, 1, 2, FUEL_SLOT][..])
        );
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
    fn the_ingredient_slot_starts_empty() {
        let brewing_stand = test_brewing_stand();
        assert!(
            brewing_stand
                .container
                .lock()
                .get_item(INGREDIENT_SLOT)
                .is_empty()
        );
    }

    /// A single brew takes 400 ticks in vanilla; the old constant of 20 ran
    /// potions twenty times too fast.
    #[test]
    fn one_brew_takes_vanilla_400_ticks() {
        assert_eq!(BREW_TIME, 400);
    }

    /// Vanilla `PotionBrewing.isIngredient` must not accept bottles: they are the
    /// input of a mix, not its ingredient.
    #[test]
    fn bottles_are_not_ingredients_and_nether_wart_is() {
        let mut water = ItemStack::new(&vanilla_items::POTION);
        water.set_potion(&steel_utils::Identifier::new("minecraft", "water"));
        assert!(!potion_brewing::is_ingredient(&water));
        assert!(!potion_brewing::is_ingredient(&ItemStack::new(
            &vanilla_items::GLASS_BOTTLE
        )));
        assert!(potion_brewing::is_ingredient(&ItemStack::new(
            &vanilla_items::NETHER_WART
        )));
        assert!(potion_brewing::is_ingredient(&ItemStack::new(
            &vanilla_items::REDSTONE
        )));
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
        brewing_stand.container.lock().set_item(
            INGREDIENT_SLOT,
            ItemStack::with_count(&steel_registry::vanilla_items::NETHER_WART, 1),
        );
        assert!(brewing_stand.can_brew());
        assert_eq!(brewing_stand.fuel(), 0);
    }

    /// Vanilla derives fuel uses from the `minecraft:brewing_fuel` component, so
    /// blaze powder is fuel and anything without the component is not.
    #[test]
    fn fuel_uses_come_from_the_brewing_fuel_component() {
        init_vanilla_registry();
        assert_eq!(
            BrewingStandBlockEntity::fuel_uses(&ItemStack::new(&vanilla_items::BLAZE_POWDER)),
            DEFAULT_BREWING_FUEL_USES
        );
        assert_eq!(
            BrewingStandBlockEntity::fuel_uses(&ItemStack::new(&vanilla_items::STONE)),
            0
        );
        assert_eq!(BrewingStandBlockEntity::fuel_uses(&ItemStack::empty()), 0);
    }
}
