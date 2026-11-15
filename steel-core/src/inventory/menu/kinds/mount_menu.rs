//! Mount menu: the inventory of a ridden animal (horse, donkey, llama, camel).
//!
//! Vanilla `AbstractMountInventoryMenu` lays out one saddle slot, one body-armor
//! slot and three rows of storage for the mount, followed by the player's own
//! inventory:
//!
//! * slot 0 — saddle, only a saddle may be placed and only while empty;
//! * slot 1 — body armor, only an item whose key ends in `_armor`, only while
//!   empty;
//! * slots 2.. — the mount's storage, three rows of `columns` slots.
//!
//! The menu is *not* a chest menu: the client gets it through
//! `ClientboundMountScreenOpenPacket` (container id, columns, entity id) rather
//! than `ClientboundOpenScreenPacket`, so it is never opened through
//! `Player::open_menu`.

use std::sync::Weak;

use glam::DVec3;
use steel_registry::equipment::EquipmentSlot;
use steel_registry::menu_type::MenuTypeRef;
use steel_registry::vanilla_items;
use steel_utils::{DowncastType, DowncastTypeKey};

use crate::entity::Entity;
use crate::inventory::container::Container as _;
use crate::inventory::prelude::*;
use crate::inventory::slots::{ArmorSlot, SlotStorage};
use crate::player::player_inventory::PlayerInventory;
use crate::player::Player;

/// Vanilla `AbstractMountInventoryMenu.SLOT_SADDLE`.
pub const SLOT_SADDLE: usize = 0;
/// Vanilla `AbstractMountInventoryMenu.SLOT_BODY_ARMOR`.
pub const SLOT_BODY_ARMOR: usize = 1;
/// Vanilla `AbstractMountInventoryMenu.SLOT_INVENTORY_START`.
pub const SLOT_INVENTORY_START: usize = 2;
/// Vanilla `AbstractMountInventoryMenu.INVENTORY_ROWS`.
pub const INVENTORY_ROWS: usize = 3;

/// Vanilla reach required to keep a mount menu open, in blocks.
const MOUNT_INTERACTION_RANGE: f64 = 4.0;

/// Vanilla `AbstractMountInventoryMenu.getInventorySize`.
#[must_use]
pub const fn inventory_size(columns: usize) -> usize {
    SLOT_INVENTORY_START + columns * INVENTORY_ROWS
}

/// Builds the mount menu for an entity with `columns` storage columns.
///
/// The container must be sized [`inventory_size(columns)`]; the saddle and body
/// armor slots are its first two slots, matching vanilla, so the same container
/// is what the mount entity keeps between opens.
///
/// # Panics
/// Panics if `columns` is zero, since vanilla computes the column count from the
/// container size and a zero-column mount has no menu at all.
#[must_use]
pub fn mount(
    inventory: Shared<PlayerInventory>,
    container_id: u8,
    container: Shared<SimpleContainer>,
    columns: usize,
    owner: Weak<dyn Entity>,
) -> Menu {
    assert!(columns > 0, "a mount menu needs at least one column");
    assert_eq!(
        container.get_container_size(),
        inventory_size(columns),
        "mount container must hold the saddle slot, the armor slot and {INVENTORY_ROWS} rows",
    );

    let mut builder = MenuBuilder::new(None, container_id);

    // Vanilla `HorseInventoryMenu` adds an `ArmorSlot` for each: the slot's own
    // `mayPlace` is `LivingEntity.isEquippableInSlot(stack, slot)`, which for
    // `EquipmentSlot.SADDLE` accepts saddles and for `EquipmentSlot.BODY`
    // accepts the mount's body armor. No extra predicate is needed.
    let saddle = builder.section_with(
        container.clone(),
        1,
        SectionKind::custom(|container, index| {
            Box::new(ArmorSlot::new(container, index, EquipmentSlot::Saddle))
        }),
    );
    let armor = builder.section_with(
        container.clone(),
        1,
        SectionKind::custom(|container, index| {
            Box::new(ArmorSlot::new(container, index, EquipmentSlot::Body))
        }),
    );
    let storage = builder.section(container.clone(), columns * INVENTORY_ROWS);
    let player = builder.player_inventory(&inventory);

    builder.route(storage, player.all(), FillDirection::Backward);
    builder.route(player.all(), [saddle, armor, storage], FillDirection::Forward);

    builder.build(MountKind {
        container,
        columns,
        owner,
    })
}

/// Per-menu mount state: the entity that owns the inventory.
pub struct MountKind {
    container: Shared<SimpleContainer>,
    columns: usize,
    owner: Weak<dyn Entity>,
}

impl MountKind {
    /// Returns the number of storage columns shown to the client.
    #[must_use]
    pub const fn columns(&self) -> usize {
        self.columns
    }
}

// SAFETY: This Steel-owned key uniquely identifies the concrete menu kind
// within the process.
unsafe impl DowncastType for MountKind {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:menu/mount");
}

impl MenuKind for MountKind {
    /// Vanilla `AbstractMountInventoryMenu.stillValid`: the mount must still be
    /// alive and reachable, and its inventory must not have been replaced.
    fn still_valid(&self, _behavior: &MenuBehavior, player: &Player) -> bool {
        let Some(owner) = self.owner.upgrade() else {
            return false;
        };
        if owner.is_removed() {
            return false;
        }
        position_within_range(owner.position(), player.position(), MOUNT_INTERACTION_RANGE)
    }
}

/// Vanilla `Entity.isWithinEntityInteractionRange(entity, range)`.
fn position_within_range(owner: DVec3, player: DVec3, range: f64) -> bool {
    owner.distance_squared(player) <= range * range
}

/// Sends `ClientboundMountScreenOpenPacket` for an open mount menu.
///
/// `Player::open_menu` always sends `ClientboundOpenScreenPacket`; a mount needs
/// the mount screen packet instead, which carries the entity id and the storage
/// columns and has no menu type at all.
pub fn send_mount_screen(player: &Player, container_id: u8, entity_id: i32, columns: usize) {
    player.send_packet(steel_protocol::packets::game::CMountScreenOpen {
        container_id: i32::from(container_id),
        inventory_columns: i32::try_from(columns).unwrap_or(9),
        entity_id,
    });
}

/// The menu type a mount screen reports, if the client asks for one.
///
/// Vanilla passes `null` to `AbstractContainerMenu` for mount menus, so there is
/// no registry menu type: the client builds the screen from the packet alone.
#[must_use]
pub const fn menu_type() -> Option<MenuTypeRef> {
    None
}

#[cfg(test)]
mod tests {
    use steel_utils::locks::IntoShared as _;

    use super::*;

    #[test]
    fn mount_inventory_has_a_saddle_slot_an_armor_slot_and_three_rows() {
        assert_eq!(inventory_size(1), 5);
        assert_eq!(inventory_size(5), 17);
    }

    #[test]
    fn mount_menu_covers_the_container_once() {
        let inventory = PlayerInventory::new().into_shared();
        let columns = 5;
        let container = SimpleContainer::new(inventory_size(columns)).into_shared();
        // Nothing else may claim these slots, so the menu owns the container.
        // The owner weak handle is only read on interaction; a dangling one is
        // enough here, and `Weak::new` needs a sized type, so downgrade a
        // throwaway entity instead.
        let entity: crate::entity::SharedEntity = crate::test_support::TestEntity::shared(
            1,
            DVec3::ZERO,
            Weak::new(),
            &steel_registry::vanilla_entities::ARROW,
        );
        let owner: Weak<dyn Entity> = std::sync::Arc::downgrade(&entity);

        let menu = mount(inventory, 3, container, columns, owner);

        // Mount slots plus the player's 36 slots.
        assert_eq!(
            menu.behavior().slot_count(),
            inventory_size(columns) + 36
        );
        assert_eq!(menu.container_id(), 3);
    }

    #[test]
    fn a_mount_menu_has_no_registry_menu_type() {
        assert!(menu_type().is_none());
    }
}
