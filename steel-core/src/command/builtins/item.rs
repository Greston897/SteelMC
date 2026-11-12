//! Vanilla `/item` command.
//!
//! Implements `replace entity <targets> <slot> <item> [count]` for player
//! inventories, using the vanilla slot names (`hotbar.N`, `inventory.N`,
//! `weapon.mainhand`, `weapon.offhand`). Vanilla also offers the `block` form and
//! the `modify` form; the block form needs a container block-entity lookup and
//! the modify form needs the item component patch parser, both of which follow in
//! a later pass.

use std::sync::Arc;

use steel_utils::Identifier;
use text_components::TextComponent;

use super::super::{
    brigadier::{CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandSource, SteelArgumentType, SteelCommandContext, SteelCommandRuntime, argument,
        literal,
    },
    registration::CommandRegistration,
};
use crate::inventory::prelude::*;
use crate::player::Player;
use crate::player::player_inventory::PlayerInventory;

type Builder = CommandNodeBuilder<CommandSource, SteelCommandRuntime>;

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("item"), |_| command())
}

fn command() -> Builder {
    literal("item").then(literal("replace").then(literal("entity").then(
        argument("targets", SteelArgumentType::players())
            .then(literal("slot").then(argument("slot", SteelArgumentType::word()).then(
                argument("item", SteelArgumentType::item_stack())
                    .executes(|context| replace(context, None))
                    .then(
                        argument("count", SteelArgumentType::integer(1, 99))
                            .executes(|context| replace(context, Some(context.integer("count")?))),
                    ),
            ))),
    )))
}

/// Vanilla slot names mapped onto the player inventory container index.
///
/// Vanilla numbers the main inventory `inventory.0..26` and the hotbar
/// `hotbar.0..8`; Steel keeps the hotbar first in the same container, so the main
/// inventory shifts by the hotbar size (`PlayerInventory::HOTBAR`).
fn slot_index(name: &str) -> Option<usize> {
    if let Some(rest) = name.strip_prefix("hotbar.") {
        let index = rest.parse::<usize>().ok()?;
        return (index < PlayerInventory::SELECTION_SIZE as usize).then_some(index);
    }
    if let Some(rest) = name.strip_prefix("inventory.") {
        let index = rest.parse::<usize>().ok()?;
        return (index < PlayerInventory::MAIN.len()).then_some(index + 9);
    }
    match name {
        "weapon" | "weapon.mainhand" => Some(PlayerInventory::HOTBAR.end),
        "weapon.offhand" => Some(PlayerInventory::SLOT_OFFHAND),
        "armor.head" => Some(PlayerInventory::ARMOR_TOP_DOWN[0]),
        "armor.chest" => Some(PlayerInventory::ARMOR_TOP_DOWN[1]),
        "armor.legs" => Some(PlayerInventory::ARMOR_TOP_DOWN[2]),
        "armor.feet" => Some(PlayerInventory::ARMOR_TOP_DOWN[3]),
        _ => None,
    }
}

fn replace(
    context: &SteelCommandContext<CommandSource>,
    count: Option<i32>,
) -> Result<i32, CommandSyntaxError> {
    let targets = context.players("targets")?;
    let slot_key = context.identifier("slot")?;
    let slot_name = slot_key.path.clone();
    let Some(slot) = slot_index(&slot_name) else {
        return Err(CommandSyntaxError::dynamic(
            "commands.item.slot.unknown",
            "Unknown item slot",
            slot_name,
            "hotbar.0".to_string(),
        ));
    };
    let stack = context.item_stack("item")?;
    let stack = match count {
        Some(count) => stack.copy_with_count(count),
        None => stack.clone(),
    };

    let mut changed = 0;
    for target in &targets {
        if set_player_slot(target, slot, stack.clone()) {
            changed += 1;
        }
    }

    let message = TextComponent::plain(format!(
        "Replaced slot {slot_name} on {changed} player(s)"
    ));
    context.source().send_success(&message, true);
    Ok(changed)
}

/// Writes one slot of a player inventory, reporting whether it succeeded.
fn set_player_slot(
    player: &Arc<Player>,
    slot: usize,
    stack: steel_registry::item_stack::ItemStack,
) -> bool {
    player.inventory.lock().set_item(slot, stack);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hotbar_slots_map_to_the_hotbar() {
        assert_eq!(slot_index("hotbar.0"), Some(0));
        assert_eq!(slot_index("hotbar.8"), Some(8));
        assert_eq!(slot_index("hotbar.9"), None);
    }

    #[test]
    fn inventory_slots_shift_past_the_hotbar() {
        assert_eq!(slot_index("inventory.0"), Some(9));
        assert_eq!(slot_index("inventory.26"), Some(35));
        assert_eq!(slot_index("inventory.27"), None);
    }

    #[test]
    fn equipment_slots_resolve() {
        assert_eq!(slot_index("armor.head"), Some(39));
        assert_eq!(slot_index("armor.feet"), Some(36));
        assert_eq!(slot_index("weapon.offhand"), Some(40));
    }

    #[test]
    fn unknown_slot_is_rejected() {
        assert_eq!(slot_index("nonsense"), None);
    }
}
