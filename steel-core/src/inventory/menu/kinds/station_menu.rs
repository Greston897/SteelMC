//! Menus for the crafting stations whose input slots live in the menu.
//!
//! Loom, cartography table, smithing table and stonecutter have no block entity:
//! vanilla keeps their input slots in a `SimpleContainer` owned by the menu kind.
//! Steel mirrors that so the slot layout and the menu type match vanilla. The
//! stonecutter resolves its recipe through the shared recipe registry; the loom,
//! cartography table and smithing table still leave the result slot empty because
//! their pattern/trim data is not modelled yet.

use steel_registry::item_stack::ItemStack;
use steel_registry::recipe::{SingleItemRecipeInput, vanilla_recipe_types};
use steel_registry::{REGISTRY, vanilla_menu_types};
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

/// Builds the loom menu: three inputs plus the banner result.
///
/// Vanilla's loom stores the result in a separate slot filled by the selected
/// banner pattern; the pattern selection is not modelled yet, so the result slot
/// stays empty but occupies its vanilla position.
#[must_use]
pub fn loom(inventory: Shared<PlayerInventory>, container_id: u8) -> Menu {
    let input = SimpleContainer::new(3).into_shared();
    let result = SimpleContainer::new(1).into_shared();
    build_station_with_result(
        &vanilla_menu_types::LOOM,
        inventory,
        container_id,
        input,
        result,
        StationKind::new,
    )
}

/// Builds the cartography table menu: two inputs plus the map result.
///
/// The `MapExtendingRecipe` matcher exists in the registry, but the copied-map
/// data it needs is not attached to the input yet, so the result slot stays empty.
#[must_use]
pub fn cartography_table(inventory: Shared<PlayerInventory>, container_id: u8) -> Menu {
    let input = SimpleContainer::new(2).into_shared();
    let result = SimpleContainer::new(1).into_shared();
    build_station_with_result(
        &vanilla_menu_types::CARTOGRAPHY_TABLE,
        inventory,
        container_id,
        input,
        result,
        StationKind::new,
    )
}

/// Builds the smithing table menu: template, base, addition and the result.
///
/// Smithing recipes are in the registry (`SmithingTransformRecipe`), but the
/// trimming/upgrade result also carries enchantment and attribute transfers that
/// are not implemented yet, so the result slot stays empty.
#[must_use]
pub fn smithing(inventory: Shared<PlayerInventory>, container_id: u8) -> Menu {
    let input = SimpleContainer::new(3).into_shared();
    let result = SimpleContainer::new(1).into_shared();
    build_station_with_result(
        &vanilla_menu_types::SMITHING,
        inventory,
        container_id,
        input,
        result,
        StationKind::new,
    )
}

/// Builds the stonecutter menu: one input, one result and the recipe selector.
///
/// Vanilla keeps the chosen recipe index in `StonecutterMenu`, so the result slot
/// is produced by [`StonecutterKind`] rather than by the input container.
#[must_use]
pub fn stonecutter(inventory: Shared<PlayerInventory>, container_id: u8) -> Menu {
    let input = SimpleContainer::new(1).into_shared();
    let result = SimpleContainer::new(1).into_shared();
    build_stonecutter(inventory, container_id, input, result)
}

/// Stonecutter state: the selected recipe index, matching
/// `StonecutterMenu.selectedRecipeIndex`.
pub struct StonecutterKind {
    input: ContainerRef,
    result: ContainerRef,
    /// Index into the matching stonecutting recipes, `-1` when none is selected.
    selected: i32,
}

// SAFETY: This Steel-owned key uniquely identifies the concrete menu kind
// within the process.
unsafe impl steel_utils::DowncastType for StonecutterKind {
    const TYPE_KEY: steel_utils::DowncastTypeKey =
        steel_utils::DowncastTypeKey::new("steel:menu/stonecutter");
}

impl StonecutterKind {
    /// Computes the result stack for a given input and selection.
    ///
    /// The matching set is recomputed on every access so the selection can never
    /// point at a recipe that the current input no longer produces — vanilla keeps
    /// the same invariant by listing the recipes in the menu packet.
    fn result_stack(input: &ItemStack, selected: i32) -> ItemStack {
        if input.is_empty() || selected < 0 {
            return ItemStack::empty();
        }
        let recipe_input = SingleItemRecipeInput::new(input.clone());
        REGISTRY
            .recipes
            .matching(&vanilla_recipe_types::STONECUTTING, &recipe_input)
            .nth(selected as usize)
            .map_or_else(ItemStack::empty, |recipe| recipe.data().result.create())
    }
}

impl MenuKind for StonecutterKind {
    fn slots_changed(
        &mut self,
        _behavior: &mut MenuBehavior,
        guard: &mut ContainerLockGuard,
        _player: &Player,
    ) {
        // Vanilla resets the selection whenever the input changes and recomputes
        // the result from the new input.
        self.selected = -1;
        let input_id = self.input.container_id();
        let Some(input_container) = guard.get(input_id) else {
            return;
        };
        let stack = input_container.get_item(0).clone();
        let result_id = self.result.container_id();
        let Some(result_container) = guard.get_mut(result_id) else {
            return;
        };
        result_container.set_item(0, Self::result_stack(&stack, self.selected));
    }

    fn click_menu_button(
        &mut self,
        _behavior: &mut MenuBehavior,
        button_id: i32,
        _player: &Player,
    ) -> bool {
        // Vanilla `StonecutterMenu.clickMenuButton`: the button id indexes the
        // currently matching recipes.
        if button_id < 0 {
            return false;
        }
        self.selected = button_id;
        true
    }
}

/// Builds the stonecutter layout: input, result, player inventory.
fn build_stonecutter(
    inventory: Shared<PlayerInventory>,
    container_id: u8,
    input: Shared<SimpleContainer>,
    result: Shared<SimpleContainer>,
) -> Menu {
    let input_ref: ContainerRef = input.into();
    let result_ref: ContainerRef = result.into();
    let mut builder = MenuBuilder::new(&vanilla_menu_types::STONECUTTER, container_id);
    let input_section = builder.section_all(input_ref.clone());
    let _result_section = builder.section_at(result_ref.clone(), [0], SectionKind::take_only());
    let player = builder.player_inventory(&inventory);
    builder.route(input_section, player.all(), FillDirection::Forward);
    builder.build(StonecutterKind {
        input: input_ref,
        result: result_ref,
        selected: -1,
    })
}

/// Builds a station menu with a dedicated result slot after the inputs.
fn build_station_with_result(
    menu_type: &'static steel_registry::menu_type::MenuTypeRef,
    inventory: Shared<PlayerInventory>,
    container_id: u8,
    input: Shared<SimpleContainer>,
    result: Shared<SimpleContainer>,
    make_kind: impl FnOnce(Shared<SimpleContainer>) -> StationKind,
) -> Menu {
    let mut builder = MenuBuilder::new(menu_type, container_id);
    let input_section = builder.section_all(input.clone());
    let _result = builder.section_at(result, [0], SectionKind::take_only());
    let player = builder.player_inventory(&inventory);
    builder.route(input_section, player.all(), FillDirection::Forward);
    let kind = make_kind(input).with_sections(input_section, player);
    builder.build(kind)
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
