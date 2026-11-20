//! Vanilla `/loot` command.
//!
//! Implements `loot <targets> give <loot_table> [<count>]`: rolls a registered
//! loot table `count` times and hands every resulting stack to the target players.
//!
//! The other vanilla forms (`insert`, `spawn`, `replace`) and the `fish`/`kill`/
//! `mine` sources need runtime contexts Steel does not model yet — a container
//! lookup for `insert`, an entity death context for `kill` — so only the plain
//! `give` form is registered.

use std::sync::Arc;

use steel_registry::REGISTRY;
use steel_registry::item_stack::ItemStack;
use steel_registry::loot_table::LootTableRef;
use steel_utils::{BlockPos, Identifier};
use text_components::TextComponent;

use super::super::{
    brigadier::{CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandSource, SteelArgumentType, SteelCommandContext, SteelCommandRuntime, argument,
        literal,
    },
    registration::CommandRegistration,
};
use crate::behavior::block::drop_from_block_interact_loot_table;
use crate::player::Player;
use crate::player::player_inventory::Prediction::ServerOnly;
use crate::world::World;

type Builder = CommandNodeBuilder<CommandSource, SteelCommandRuntime>;

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("loot"), |_| command())
}

fn command() -> Builder {
    // Vanilla grammar: loot give <targets> loot <loot_table>
    // Also keeps loot <targets> give <loot_table> [<count>] subbranch.
    literal("loot")
        .then(
            literal("give").then(
                argument("targets", SteelArgumentType::players()).then(
                    literal("loot").then(
                        argument("loot_table", SteelArgumentType::identifier())
                            .executes(give)
                            .then(
                                argument(
                                    "count",
                                    super::super::brigadier::ArgumentType::integer(1, 99),
                                )
                                .executes(give_with_count),
                            ),
                    ),
                ),
            ),
        )
        .then(
            argument("targets", SteelArgumentType::players()).then(
                literal("give").then(
                    argument("loot_table", SteelArgumentType::identifier())
                        .executes(give)
                        .then(
                            argument(
                                "count",
                                super::super::brigadier::ArgumentType::integer(1, 99),
                            )
                            .executes(give_with_count),
                        ),
                ),
            ),
        )
}

fn loot_table_of(
    context: &SteelCommandContext<CommandSource>,
) -> Result<LootTableRef, CommandSyntaxError> {
    let key = context.identifier("loot_table")?;
    REGISTRY
        .loot_tables
        .by_key(key)
        .ok_or_else(|| CommandSyntaxError::dynamic(format!("Unknown loot table '{key}'")))
}

fn roll(table: LootTableRef, world: &Arc<World>, pos: BlockPos) -> Vec<ItemStack> {
    let state = world.get_block_state(pos);
    let mut random = rand::rng();
    drop_from_block_interact_loot_table(table, state, None, None, None, &mut random)
}

fn give(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    insert_or_give(context, 1)
}

fn give_with_count(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let count = context.integer("count")?;
    insert_or_give(context, count)
}

fn insert_or_give(
    context: &SteelCommandContext<CommandSource>,
    count: i32,
) -> Result<i32, CommandSyntaxError> {
    let table = loot_table_of(context)?;
    let targets = context.players("targets")?;
    let world = context.source().world();
    // Vanilla resolves the loot table at the source position; the command source
    // always has one, so the block state used for `loot` predicates comes from here.
    let source_position = context.source().position();
    let pos = steel_utils::BlockPos::new(
        source_position.x.floor() as i32,
        source_position.y.floor() as i32,
        source_position.z.floor() as i32,
    );

    let mut total = 0;
    for _ in 0..count {
        let items = roll(table, world, pos);
        for item in items {
            for target in &targets {
                give_to_player(target, item.clone());
            }
            total += 1;
        }
    }

    let message = TextComponent::plain(format!("Gave {total} stacks of loot"));
    context.source().send_success(&message, true);
    Ok(total)
}

fn give_to_player(player: &Player, item: ItemStack) {
    player.add_item_or_drop(item, ServerOnly);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_loot_tables_are_rejected() {
        let key = Identifier::vanilla_static("not_a_real_table");
        assert!(REGISTRY.loot_tables.by_key(&key).is_none());
    }

    #[test]
    fn vanilla_loot_tables_are_registered() {
        steel_registry::init_vanilla_registry();
        let key = Identifier::vanilla_static("chests/simple_dungeon");
        assert!(REGISTRY.loot_tables.by_key(&key).is_some());
    }
}
