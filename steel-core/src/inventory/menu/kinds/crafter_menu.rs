//! Crafter menu (3x3 grid with a slot-disable mask).
//!
//! Slot layout (46 total):
//! - Slots 0-8: the crafter's 3x3 grid
//! - Slots 9-35: Main inventory (27)
//! - Slots 36-44: Hotbar (9)
//! - Slot 45: a display slot for the unused result the client builds
//!
//! Vanilla `CrafterMenu` shows the grid, mirrors the block entity's
//! `disabled_slots` bitmask into data slot 0 and lets the client toggle a slot
//! while the cursor is empty (the `SContainerSlotStateChanged` packet, handled
//! by [`Player::handle_container_slot_state_changed`]). A disabled grid slot
//! refuses placement, exactly like vanilla's `CrafterSlot.mayPlace`, and a slot
//! that already holds an item can never be disabled.
//!
//! Slot 45 is not part of the vanilla layout but the client always builds 46
//! slots for `crafter_3x3`, so it is added as a display slot that can never be
//! used.

use std::sync::Arc;

use crate::block_entity::SharedBlockEntity;
use crate::block_entity::entities::crafter::{CRAFTER_SLOTS, CrafterBlockEntity};
use crate::inventory::prelude::*;
use crate::player::player_inventory::PlayerInventory;
use steel_registry::vanilla_blocks;
use steel_registry::vanilla_menu_types;
use steel_utils::{BlockPos, Downcast as _, locks::SyncMutex};

/// Builds the crafter menu.
///
/// # Panics
/// Panics if the client-side slot count for `crafter_3x3` is not 46, which is
/// exactly the layout assumed here.
#[must_use]
pub fn crafter(
    inventory: Shared<PlayerInventory>,
    container_id: u8,
    block_pos: BlockPos,
    container: impl Into<ContainerRef>,
    crafter: SharedBlockEntity,
) -> Menu {
    assert_eq!(
        vanilla_menu_types::CRAFTER_3X3.slot_count,
        46,
        "crafter menu layout assumes the vanilla 46-slot crafter_3x3"
    );

    let initial_mask = crafter
        .downcast_ref::<CrafterBlockEntity>()
        .map_or(0, CrafterBlockEntity::disabled_slots);
    let mask = Arc::new(SyncMutex::new(initial_mask));

    // Vanilla `CrafterSlot.mayPlace`: nothing may enter a disabled slot.
    let placement_mask = Arc::clone(&mask);
    let rules = SectionKind::restricted(move |slot, _stack| {
        slot >= CRAFTER_SLOTS || *placement_mask.lock() & (1 << slot) == 0
    });

    let mut builder = MenuBuilder::new(&vanilla_menu_types::CRAFTER_3X3, container_id);
    let grid = builder.section_all_with(container, rules);
    let player = builder.player_inventory(&inventory);
    // Slot 45 exists client-side but is never usable; vanilla `CrafterMenu`
    // adds it as a result slot that stays empty forever.
    builder.section_all_with(
        SimpleContainer::new(1).into_shared(),
        SectionKind::Display,
    );
    let disabled_slots = builder.data_slot(initial_mask as i16);

    builder.route(grid, player.all(), FillDirection::Forward);
    builder.route(
        player.main(),
        [grid, player.hotbar()],
        FillDirection::Forward,
    );
    builder.route(
        player.hotbar(),
        [grid, player.main()],
        FillDirection::Forward,
    );
    builder.drain(grid);

    builder.build(CrafterKind {
        block_pos,
        crafter,
        mask,
        disabled_slots,
        last_synced: u16::MAX,
    })
}

/// Per-menu crafter state: the block entity whose mask the client toggles.
pub struct CrafterKind {
    block_pos: BlockPos,
    crafter: SharedBlockEntity,
    /// The mask shared with the grid's placement rule.
    mask: Arc<SyncMutex<u16>>,
    /// Data slot 0, mirrors the block entity's disabled mask.
    disabled_slots: DataSlot,
    /// Last mask pushed to the client, so `on_tick` only syncs on change.
    last_synced: u16,
}

// SAFETY: This Steel-owned key uniquely identifies the concrete menu kind
// within the process.
unsafe impl steel_utils::DowncastType for CrafterKind {
    const TYPE_KEY: steel_utils::DowncastTypeKey =
        steel_utils::DowncastTypeKey::new("steel:menu/crafter");
}

impl CrafterKind {
    fn entity(&self) -> Option<&CrafterBlockEntity> {
        self.crafter.downcast_ref::<CrafterBlockEntity>()
    }

    /// Pushes the block entity's mask into the data slot when it changed.
    fn sync_mask(&mut self, behavior: &mut MenuBehavior) {
        let mask = self.entity().map_or(0, CrafterBlockEntity::disabled_slots);
        if mask == self.last_synced {
            return;
        }
        *self.mask.lock() = mask;
        self.last_synced = mask;
        self.disabled_slots.set(behavior, mask as i16);
    }

    /// Applies a slot-state toggle coming from `SContainerSlotStateChanged`.
    ///
    /// Mirrors vanilla `CrafterMenu.setSlotState`: a slot that still holds an
    /// item can never be disabled, and an unchanged mask is a no-op.
    /// Returns whether the mask changed.
    ///
    /// Returns `false` for a slot outside the 3x3 grid, which a malicious
    /// client can send.
    pub fn apply_slot_state(
        &mut self,
        behavior: &mut MenuBehavior,
        slot: usize,
        disabled: bool,
        occupied: bool,
    ) -> bool {
        if slot >= CRAFTER_SLOTS {
            return false;
        }
        let bit = 1u16 << slot;
        let current = self.entity().map_or(0, CrafterBlockEntity::disabled_slots);
        let next = if disabled {
            if current & bit != 0 || occupied {
                return false;
            }
            current | bit
        } else {
            if current & bit == 0 {
                return false;
            }
            current & !bit
        };
        if let Some(entity) = self.entity() {
            entity.set_disabled_slots(next);
        }
        *self.mask.lock() = next;
        self.last_synced = next;
        self.disabled_slots.set(behavior, next as i16);
        true
    }
}

impl MenuKind for CrafterKind {
    fn still_valid(&self, _behavior: &MenuBehavior, player: &Player) -> bool {
        still_valid_block(
            &player.get_world(),
            self.block_pos,
            &vanilla_blocks::CRAFTER,
            player,
        )
    }

    fn on_open(
        &mut self,
        behavior: &mut MenuBehavior,
        _guard: &mut ContainerLockGuard,
        _player: &Player,
    ) {
        // Seed the client with the persisted mask before the first render.
        self.sync_mask(behavior);
    }

    fn on_tick(
        &mut self,
        behavior: &mut MenuBehavior,
        _guard: &mut ContainerLockGuard,
        _player: &Player,
    ) {
        // The crafter ticker can change the mask (e.g. it consumes a slot), so
        // poll instead of relying on the menu to be the only writer.
        self.sync_mask(behavior);
    }

    /// Vanilla `CrafterMenu.slotClicked`: right-clicking a grid slot with an
    /// empty cursor toggles it and does not move any item.
    fn on_slot_clicked(
        &mut self,
        behavior: &mut MenuBehavior,
        guard: &mut ContainerLockGuard,
        click: Click,
        _player: &Player,
    ) -> ClickOutcome {
        let Click::Pickup {
            slot,
            button: MouseButton::Right,
        } = click
        else {
            return ClickOutcome::Fallthrough;
        };
        if slot >= CRAFTER_SLOTS {
            return ClickOutcome::Fallthrough;
        }
        let disabled = self.entity().map_or(0, CrafterBlockEntity::disabled_slots) & (1 << slot) == 0;
        let occupied = !behavior.slots()[slot].get_item(guard).is_empty();
        // A failed toggle still consumes the click, so the cursor item is not
        // placed into an occupied slot.
        let _ = self.apply_slot_state(behavior, slot, disabled, occupied);
        ClickOutcome::Consume
    }
}
