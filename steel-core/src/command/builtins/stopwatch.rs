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
        CommandSource, SteelArgumentType, SteelCommandContext, SteelCommandRuntime, argument,
        literal,
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
            literal("create")
                .then(argument("id", SteelArgumentType::identifier()).executes(create_stopwatch)),
        )
        .then(
            literal("query").then(
                argument("id", SteelArgumentType::identifier())
                    .executes(|context| query_stopwatch(context, 1.0))
                    .then(
                        argument("scale", ArgumentType::double(f64::MIN, f64::MAX))
                            .executes(query_stopwatch_scaled),
                    ),
            ),
        )
        .then(
            literal("restart")
                .then(argument("id", SteelArgumentType::identifier()).executes(restart_stopwatch)),
        )
        .then(
            literal("remove")
                .then(argument("id", SteelArgumentType::identifier()).executes(remove_stopwatch)),
        )
        .then(literal("list").executes(list_stopwatches))
}

/// Resolves the domain's scoreboard for the duration of one command.
///
/// The board lives on the server for the whole session, so the borrow can be
/// tied to `source`'s server reference rather than to `source` itself.
fn scoreboard_of(source: &CommandSource) -> Result<&Scoreboard, CommandSyntaxError> {
    let domain = source.world().domain().to_owned();
    source
        .server()
        .scoreboards
        .get(&domain)
        .ok_or_else(|| CommandSyntaxError::dynamic(format!("Domain '{domain}' has no scoreboard")))
}

fn create_stopwatch(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let name = context.identifier("id")?.to_string();
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

fn query_stopwatch_scaled(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let scale = context.double("scale")?;
    query_stopwatch(context, scale)
}

fn query_stopwatch(
    context: &SteelCommandContext<CommandSource>,
    scale: f64,
) -> Result<i32, CommandSyntaxError> {
    let name = context.identifier("id")?.to_string();
    let scoreboard = scoreboard_of(context.source())?;
    let Some(ticks) = scoreboard.stopwatch_ticks(&name) else {
        return Err(CommandSyntaxError::dynamic(format!(
            "No stopwatch named '{name}' exists"
        )));
    };
    let seconds = ticks as f64 / 20.0;
    let scaled = (seconds * scale).floor();
    #[expect(
        clippy::cast_possible_truncation,
        reason = "clamped into i32 range before truncating per vanilla StopwatchCommand.queryStopwatch"
    )]
    let result = scaled.clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32;
    context.source().send_success(
        &TextComponent::plain(format!(
            "Elapsed time for stopwatch '{name}' is {seconds:.3}s"
        )),
        false,
    );
    Ok(result)
}

fn restart_stopwatch(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let name = context.identifier("id")?.to_string();
    let scoreboard = scoreboard_of(context.source())?;
    if !scoreboard.restart_stopwatch(&name) {
        return Err(CommandSyntaxError::dynamic(format!(
            "No stopwatch named '{name}' exists"
        )));
    }
    context.source().send_success(
        &TextComponent::plain(format!("Restarted stopwatch '{name}'")),
        false,
    );
    Ok(1)
}

fn remove_stopwatch(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let name = context.identifier("id")?.to_string();
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
    context: &SteelCommandContext<CommandSource>,
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
