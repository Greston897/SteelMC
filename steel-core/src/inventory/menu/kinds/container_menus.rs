//! Menus for the block entities whose slot layout is fixed and small.
//!
//! Vanilla has a dedicated menu class per block entity here. Steel keeps them in
//! one module because none of them has behaviour beyond the slot layout and the
//! placement rules:
//!
//! | Menu | Slots | Layout |
//! |---|---|---|
//! | [`dispenser`] (`generic_3x3`) | 45 | 9 grid + 27 main + 9 hotbar |
//! | [`hopper`] | 41 | 5 slots + 27 main + 9 hotbar |
//! | [`brewing_stand`] | 41 | 3 bottles + ingredient + fuel + 27 + 9 |
//! | [`lectern`] | 1 | 1 book slot |
//!
//! The lectern is the odd one out: vanilla shows a single book slot to the
//! player who opened it and the book itself is the block entity's only item, so
//! the menu has no player inventory at all.

use steel_registry::{vanilla_items, vanilla_menu_types};

use crate::block_entity::entities::BrewingStandBlockEntity;
use crate::inventory::menu::kinds::BasicKind;
use crate::inventory::prelude::*;
use steel_utils::Downcast as _;
use crate::player::player_inventory::PlayerInventory;

/// Builds the dispenser/dropper grid menu.
///
/// Vanilla uses `generic_3x3` for both blocks: they differ by behaviour, not by
/// menu type. The layout is the 3x3 grid followed by the player inventory.
#[must_use]
pub fn dispenser(
    inventory: Shared<PlayerInventory>,
    container_id: u8,
    container: impl Into<ContainerRef>,
) -> Menu {
    let container = container.into();
    let mut builder = MenuBuilder::new(&vanilla_menu_types::GENERIC_3X3, container_id);
    let grid = builder.section(&container, 9);
    let player = builder.player_inventory(&inventory);

    builder.route(grid, player.all(), FillDirection::Backward);
    builder.route(player.all(), grid, FillDirection::Forward);
    builder.build(BasicKind)
}

/// Builds the hopper menu: five container slots above the player inventory.
///
/// Vanilla `HopperMenu` lays the five slots out in a single row, so the menu has
/// one row of five rather than a grid.
#[must_use]
pub fn hopper(
    inventory: Shared<PlayerInventory>,
    container_id: u8,
    container: impl Into<ContainerRef>,
) -> Menu {
    let container = container.into();
    let mut builder = MenuBuilder::new(&vanilla_menu_types::HOPPER, container_id);
    let slots = builder.section(&container, 5);
    let player = builder.player_inventory(&inventory);

    builder.route(slots, player.all(), FillDirection::Backward);
    builder.route(player.all(), slots, FillDirection::Forward);
    builder.build(BasicKind)
}

/// Builds the brewing stand menu.
///
/// Vanilla layout: three bottle slots (`0..=2`), the ingredient slot (`3`) and
/// the blaze-powder fuel slot (`4`), then the player inventory. The block
/// entity owns the container, so the menu only mirrors it.
#[must_use]
pub fn brewing_stand(
    inventory: Shared<PlayerInventory>,
    container_id: u8,
    container: impl Into<ContainerRef>,
) -> Menu {
    let container = container.into();
    let mut builder = MenuBuilder::new(&vanilla_menu_types::BREWING_STAND, container_id);
    let stand = builder.section(&container, 5);
    let player = builder.player_inventory(&inventory);
    // Vanilla `BrewingStandMenu` publishes two data slots: the elapsed brew
    // ticks (`BrewTime`) and the remaining fuel uses (`Fuel`), which is what
    // drives the progress bar and the fuel gauge in the screen.
    let brew_time = builder.data_slot(0);
    let fuel = builder.data_slot(0);

    builder.route(stand, player.all(), FillDirection::Backward);
    builder.route(player.all(), stand, FillDirection::Forward);
    builder.build(BrewingStandMenuKind {
        container,
        brew_time,
        fuel,
    })
}

/// State handles for the brewing stand screen.
///
/// Vanilla publishes two data slots through `BrewingStandMenu`: `BrewTime` and
/// the remaining fuel uses, which drive the progress bar and the fuel gauge.
pub struct BrewingStandMenuKind {
    /// The stand's container, used to find the block entity for its data slots.
    container: ContainerRef,
    /// Elapsed brew ticks, vanilla `BrewingStandMenu.DATA_BREW_TIME`.
    pub brew_time: DataSlot,
    /// Remaining fuel uses, vanilla `BrewingStandMenu.DATA_FUEL_USES`.
    pub fuel: DataSlot,
}

// SAFETY: This Steel-owned key uniquely identifies the concrete menu kind
// within the process.
unsafe impl steel_utils::DowncastType for BrewingStandMenuKind {
    const TYPE_KEY: steel_utils::DowncastTypeKey =
        steel_utils::DowncastTypeKey::new("steel:menu/brewing_stand");
}

impl BrewingStandMenuKind {
    /// Publishes the vanilla data slots.
    ///
    /// The two values live on the block entity, so the menu reaches it through
    /// the container's owner: `ContainerRef::level` + `owner_pos` identify the
    /// block entity, and `Downcast::downcast_ref` gets the concrete stand.
    fn update_data(&self, behavior: &mut MenuBehavior, container: &ContainerRef) {
        let Some(level) = container.level() else {
            return;
        };
        let Some(pos) = container.owner_pos() else {
            return;
        };
        let Some(block_entity) = level.get_block_entity(pos) else {
            return;
        };
        let Some(stand) = block_entity
            .as_ref()
            .downcast_ref::<BrewingStandBlockEntity>()
        else {
            return;
        };
        let [brew_time, fuel] = stand.menu_data();
        self.brew_time.set(behavior, brew_time);
        self.fuel.set(behavior, fuel);
    }
}

impl MenuKind for BrewingStandMenuKind {
    fn on_open(
        &mut self,
        behavior: &mut MenuBehavior,
        guard: &mut ContainerLockGuard,
        _player: &Player,
    ) {
        self.update_data(behavior, &self.container);
        let _ = guard;
    }

    fn on_tick(
        &mut self,
        behavior: &mut MenuBehavior,
        guard: &mut ContainerLockGuard,
        _player: &Player,
    ) {
        let _ = guard;
        self.update_data(behavior, &self.container);
    }
}

/// Builds the lectern menu.
///
/// Vanilla `LecternMenu` shows a single book slot and no player inventory: the
/// book is the block entity's item and is returned when the reader closes the
/// screen. The slot refuses anything that is not a written, writable or
/// knowledge book, matching `LecternMenu.canPlaceItem`.
#[must_use]
pub fn lectern(
    container_id: u8,
    container: impl Into<ContainerRef>,
) -> Menu {
    let container = container.into();
    let mut builder = MenuBuilder::new(&vanilla_menu_types::LECTERN, container_id);
    builder.section_with(
        &container,
        1,
        SectionKind::restricted(|_slot, stack| {
            stack.is(&*vanilla_items::WRITABLE_BOOK)
                || stack.is(&*vanilla_items::WRITTEN_BOOK)
                || stack.is(&*vanilla_items::KNOWLEDGE_BOOK)
        }),
    );
    builder.build(BasicKind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_counts_match_the_vanilla_menu_types() {
        // The client builds its layout from the menu type, so a mismatch would
        // desynchronise the screen from what the server sends.
        assert_eq!(vanilla_menu_types::GENERIC_3X3.slot_count, 45);
        assert_eq!(vanilla_menu_types::HOPPER.slot_count, 41);
        assert_eq!(vanilla_menu_types::BREWING_STAND.slot_count, 41);
        assert_eq!(vanilla_menu_types::LECTERN.slot_count, 1);
    }

    #[test]
    fn the_dispenser_grid_holds_nine_slots() {
        let inventory = PlayerInventory::new().into_shared();
        let container = SimpleContainer::new(9).into_shared();
        let menu = dispenser(inventory, 0, container);
        // 9 grid + 27 main + 9 hotbar
        assert_eq!(menu.behavior().slot_count(), 45);
    }

    #[test]
    fn the_hopper_menu_holds_five_container_slots() {
        let inventory = PlayerInventory::new().into_shared();
        let container = SimpleContainer::new(5).into_shared();
        let menu = hopper(inventory, 0, container);
        assert_eq!(menu.behavior().slot_count(), 41);
    }

    #[test]
    fn the_lectern_menu_has_no_player_inventory() {
        let container = SimpleContainer::new(1).into_shared();
        let menu = lectern(0, container);
        assert_eq!(menu.behavior().slot_count(), 1);
    }
}
