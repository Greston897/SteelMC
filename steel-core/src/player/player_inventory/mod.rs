//! Player inventory management.

use std::sync::Arc;

use text_components::TextComponent;

use crate::{inventory::menu::Menu, player::Player, world::World};

mod container;
mod core;
mod equipment;
mod player_handlers;

pub use container::InvalidHotbarSlot;
pub(crate) use container::armor_equipment;
pub use core::PlayerInventory;
pub use equipment::EquipmentSwapResult;

/// Inputs supplied when an external menu factory is safe to execute.
pub struct MenuOpenContext<'a> {
    /// Wire container id allocated for this menu.
    pub container_id: u8,
    /// Player opening the menu.
    pub player: &'a Player,
    /// Player's world at factory execution time.
    pub world: &'a Arc<World>,
}

/// Whether the client already predicted a side effect such as the drop swing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prediction {
    /// The client predicted it; only tracking players need the update.
    Predicted,
    /// Only the server knows; the acting player needs the update too.
    ServerOnly,
}

/// Whether a terminal menu removal completed synchronously.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub enum MenuRemovalStatus {
    /// Both the base inventory menu and any external menu were removed.
    Complete,
    /// A callback or in-flight open operation owns menu state; removal will
    /// finish when it unwinds.
    Pending,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum MenuItemDisposition {
    ReturnToInventory,
    Drop,
}

impl MenuItemDisposition {
    const fn combine(self, other: Self) -> Self {
        if matches!(self, Self::Drop) || matches!(other, Self::Drop) {
            Self::Drop
        } else {
            Self::ReturnToInventory
        }
    }
}

pub(super) struct OpenMenuState {
    menu: Option<Menu>,
    dispatch: Option<OpenMenuDispatch>,
    terminal_removal: Option<TerminalMenuRemoval>,
    active_open_operations: usize,
}

pub(super) struct PlayerInventorySyncState {
    pending_slots: [bool; PlayerInventory::CONTAINER_SIZE],
}

impl PlayerInventorySyncState {
    pub(super) const fn new() -> Self {
        Self {
            pending_slots: [false; PlayerInventory::CONTAINER_SIZE],
        }
    }

    fn request(&mut self, slots: impl IntoIterator<Item = usize>) {
        for slot in slots {
            assert!(
                slot < PlayerInventory::CONTAINER_SIZE,
                "logical player inventory slot {slot} is out of bounds"
            );
            self.pending_slots[slot] = true;
        }
    }

    fn take_ready(&mut self, overrides_player_slots: bool) -> Vec<usize> {
        self.pending_slots
            .iter_mut()
            .enumerate()
            .filter_map(|(slot, pending)| {
                if !*pending || (overrides_player_slots && slot < PlayerInventory::INVENTORY_SIZE) {
                    return None;
                }
                *pending = false;
                Some(slot)
            })
            .collect()
    }
}

struct OpenMenuDispatch {
    container_id: u8,
    overrides_player_slots: bool,
    actions: Vec<DeferredMenuAction>,
}

struct TerminalMenuRemoval {
    disposition: MenuItemDisposition,
    main_cleanup_complete: bool,
    pending_cleanup_in_progress: bool,
    pending_menus: Vec<Menu>,
}

enum DeferredMenuAction {
    Close { send_packet: bool },
    Open(Box<PendingMenuOpen>),
    Install(Box<PreparedMenu>),
}

type MenuFactory = Box<dyn for<'a> FnOnce(MenuOpenContext<'a>) -> Menu + Send + 'static>;

struct PendingMenuOpen {
    title: TextComponent,
    create: MenuFactory,
    /// Set for mounts: the client needs a mount screen packet instead of the
    /// ordinary open-screen packet, carrying the entity and the column count.
    mount: Option<MountScreen>,
}

struct PreparedMenu {
    title: TextComponent,
    menu: Menu,
    mount: Option<MountScreen>,
}

/// The extra fields `ClientboundMountScreenOpenPacket` needs.
#[derive(Clone, Copy)]
pub(crate) struct MountScreen {
    /// Entity id of the ridden mount.
    pub entity_id: i32,
    /// Storage columns shown to the client.
    pub columns: usize,
}

enum OpenMenuUnavailable {
    Closed,
    Unavailable,
}

impl OpenMenuState {
    pub(super) const fn new() -> Self {
        Self {
            menu: None,
            dispatch: None,
            terminal_removal: None,
            active_open_operations: 0,
        }
    }

    /// Takes the open menu out of the slot.
    ///
    /// Callers that run a handler without holding the lock take it, run the
    /// handler and put it back, so a re-entrant menu operation cannot deadlock.
    pub(super) const fn take_menu(&mut self) -> Option<Menu> {
        self.menu.take()
    }

    /// Puts a menu back after [`Self::take_menu`].
    pub(super) fn put_menu(&mut self, menu: Menu) {
        self.menu = Some(menu);
    }
}

#[cfg(test)]
mod tests;

impl Player {
    /// Returns None while a menu callback temporarily owns the active menu.
    /// Rechecks must retry instead of treating that interval as a closed menu.
    pub(crate) fn views_container(&self, id: crate::inventory::lock::ContainerId) -> Option<bool> {
        let state = self.open_menu.lock();
        if state.dispatch.is_some() {
            return None;
        }
        Some(
            state
                .menu
                .as_ref()
                .is_some_and(|menu| menu.behavior().references_container(id)),
        )
    }
}
