//! Vanilla `/stopwatch` command.
//!
//! A stopwatch is a named counter that advances once per server tick while it
//! exists. Vanilla keeps them on the scoreboard, where they persist with the
//! level and can be tested from `/execute if stopwatch`.

use steel_utils::Identifier;
use text_components::TextComponent;

use super::super::{
    brigadier::{ArgumentType, CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandSource, SteelCommandContext, SteelCommandRuntime, argument, literal,
    },
    registration::CommandRegistration,
};
use crate::scoreboard::Scoreboard;

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("stopwatch"), |_| command())
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("stopwatch")
        .then(
            literal("create").then(
                argument("name", ArgumentType::word()).executes(create_stopwatch),
            ),
        )
        .then(
            literal("remove").then(
                argument("name", ArgumentType::word()).executes(remove_stopwatch),
            ),
        )
        .then(literal("list").executes(list_stopwatches))
}

/// Resolves the domain's scoreboard for the duration of one command.
///
/// The board lives on the server for the whole session, so the borrow can be
/// tied to `source`'s server reference rather than to `source` itself.
fn scoreboard_of<'a>(
    source: &'a CommandSource,
) -> Result<&'a Scoreboard, CommandSyntaxError> {
    let domain = source.world().domain().to_owned();
    source
        .server()
        .scoreboards
        .get(&domain)
        .ok_or_else(|| CommandSyntaxError::dynamic(format!("Domain '{domain}' has no scoreboard")))
}

fn create_stopwatch(
    context: SteelCommandContext<'_, CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let name = context.string("name")?.to_owned();
    let scoreboard = scoreboard_of(context.source())?;
    if !scoreboard.create_stopwatch(&name) {
        return Err(CommandSyntaxError::dynamic(format!(
            "A stopwatch named '{name}' already exists"
        )));
    }
    context.source().send_success(
        &TextComponent::plain(format!("Created stopwatch '{name}'")),
        false,
    );
    Ok(1)
}

fn remove_stopwatch(
    context: SteelCommandContext<'_, CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let name = context.string("name")?.to_owned();
    let scoreboard = scoreboard_of(context.source())?;
    if !scoreboard.remove_stopwatch(&name) {
        return Err(CommandSyntaxError::dynamic(format!(
            "No stopwatch named '{name}' exists"
        )));
    }
    context.source().send_success(
        &TextComponent::plain(format!("Removed stopwatch '{name}'")),
        false,
    );
    Ok(1)
}

fn list_stopwatches(
    context: SteelCommandContext<'_, CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let scoreboard = scoreboard_of(context.source())?;
    let names = scoreboard.stopwatch_names();
    if names.is_empty() {
        context
            .source()
            .send_success(&TextComponent::plain("There are no stopwatches"), false);
        return Ok(0);
    }
    for name in &names {
        let ticks = scoreboard.stopwatch_ticks(name).unwrap_or_default();
        context.source().send_success(
            &TextComponent::plain(format!("{name}: {ticks} ticks")),
            false,
        );
    }
    Ok(names.len() as i32)
}
