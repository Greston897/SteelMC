//! Menus for the crafting stations whose input slots live in the menu.
//!
//! Loom, cartography table, smithing table and stonecutter have no block entity:
//! vanilla keeps their input slots in a `SimpleContainer` owned by the menu kind.
//! Steel mirrors that so the slot layout and the menu type match vanilla; the
//! result-slot rules that need a recipe registry (loom patterns, smithing
//! templates, stonecutter recipes) are not wired yet and leave the result empty.

use steel_registry::vanilla_menu_types;
use steel_utils::locks::IntoShared;

use crate::inventory::container::SimpleContainer;
use crate::inventory::prelude::*;
use crate::player::player_inventory::PlayerInventory;

/// Menu kind shared by the station menus.
pub struct StationKind {
    /// Input container backing the menu's own slots.
    pub input: Shared<SimpleContainer>,
    input_section: Option<Section>,
    player: Option<PlayerInventorySections>,
}

impl StationKind {
    /// Creates the kind for a given input container.
    #[must_use]
    pub fn new(input: Shared<SimpleContainer>) -> Self {
        Self {
            input,
            input_section: None,
            player: None,
        }
    }

    fn with_sections(mut self, input_section: Section, player: PlayerInventorySections) -> Self {
        self.input_section = Some(input_section);
        self.player = Some(player);
        self
    }
}

// SAFETY: This Steel-owned key uniquely identifies the concrete menu kind
// within the process.
unsafe impl steel_utils::DowncastType for StationKind {
    const TYPE_KEY: steel_utils::DowncastTypeKey =
        steel_utils::DowncastTypeKey::new("steel:menu/station");
}

impl MenuKind for StationKind {}

/// Builds the loom menu (four inputs: banner, dye, pattern item, pattern).
#[must_use]
pub fn loom(inventory: Shared<PlayerInventory>, container_id: u8) -> Menu {
    let input = SimpleContainer::new(4).into_shared();
    build_station(
        &vanilla_menu_types::LOOM,
        inventory,
        container_id,
        input,
        StationKind::new,
    )
}

/// Builds the cartography table menu (two inputs).
#[must_use]
pub fn cartography_table(inventory: Shared<PlayerInventory>, container_id: u8) -> Menu {
    let input = SimpleContainer::new(2).into_shared();
    build_station(
        &vanilla_menu_types::CARTOGRAPHY_TABLE,
        inventory,
        container_id,
        input,
        StationKind::new,
    )
}

/// Builds the smithing table menu (template, base, addition).
#[must_use]
pub fn smithing(inventory: Shared<PlayerInventory>, container_id: u8) -> Menu {
    let input = SimpleContainer::new(3).into_shared();
    build_station(
        &vanilla_menu_types::SMITHING,
        inventory,
        container_id,
        input,
        StationKind::new,
    )
}

/// Builds the stonecutter menu (one input plus a recipe selector).
#[must_use]
pub fn stonecutter(inventory: Shared<PlayerInventory>, container_id: u8) -> Menu {
    let input = SimpleContainer::new(1).into_shared();
    build_station(
        &vanilla_menu_types::STONECUTTER,
        inventory,
        container_id,
        input,
        StationKind::new,
    )
}

/// Shared builder used by every station menu.
fn build_station(
    menu_type: &'static steel_registry::menu_type::MenuTypeRef,
    inventory: Shared<PlayerInventory>,
    container_id: u8,
    input: Shared<SimpleContainer>,
    make_kind: impl FnOnce(Shared<SimpleContainer>) -> StationKind,
) -> Menu {
    let mut builder = MenuBuilder::new(menu_type, container_id);
    let input_section = builder.section_all(input);
    let player = builder.player_inventory(&inventory);
    builder.route(input_section, player.all(), FillDirection::Forward);
    let kind = make_kind(input).with_sections(input_section, player);
    builder.build(kind)
}
