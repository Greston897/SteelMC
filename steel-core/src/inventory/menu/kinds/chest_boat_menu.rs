//! Chest boat menu.
//!
//! Vanilla `AbstractChestBoatEntity#createMenu` builds
//! `GenericContainerScreenHandler.createGeneric9x3`, i.e. a three-row chest menu
//! whose validity follows the *entity* rather than a block position.

use std::sync::Weak;

use steel_utils::{DowncastType, DowncastTypeKey};

use crate::entity::Entity;
use crate::inventory::prelude::*;
use crate::player::player_inventory::PlayerInventory;

use super::chest_menu::chest_with_kind;

/// Container rows in a chest boat (vanilla `INVENTORY_SIZE = 27`).
pub const CHEST_BOAT_ROWS: usize = 3;

/// Vanilla `AbstractChestBoatEntity` access radius, in blocks.
const CHEST_BOAT_ACCESS_RADIUS: f64 = 8.0;

/// Builds the 3×9 chest boat menu plus the player inventory.
#[must_use]
pub fn chest_boat(
    inventory: Shared<PlayerInventory>,
    container_id: u8,
    container: Shared<SimpleContainer>,
    owner: Weak<dyn Entity>,
) -> Menu {
    chest_with_kind(
        inventory,
        container_id,
        container,
        CHEST_BOAT_ROWS,
        ChestBoatKind { owner },
    )
}

/// Per-menu chest boat state: the owning entity, for the access check.
pub struct ChestBoatKind {
    owner: Weak<dyn Entity>,
}

// SAFETY: This Steel-owned key uniquely identifies the concrete menu kind
// within the process.
unsafe impl DowncastType for ChestBoatKind {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:menu/chest_boat");
}

impl MenuKind for ChestBoatKind {
    /// Vanilla `AbstractChestBoatEntity#canPlayerUse` → `canPlayerAccess`:
    /// the boat must still exist and the player must stay within eight blocks.
    fn still_valid(&self, _behavior: &MenuBehavior, player: &Player) -> bool {
        let Some(owner) = self.owner.upgrade() else {
            return false;
        };
        if owner.is_removed() {
            return false;
        }
        let delta = owner.position() - player.position();
        delta.length_squared() <= CHEST_BOAT_ACCESS_RADIUS * CHEST_BOAT_ACCESS_RADIUS
    }
}
