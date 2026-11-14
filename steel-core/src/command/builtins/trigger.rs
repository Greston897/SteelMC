//! Vanilla `/trigger` command.
//!
//! `/trigger` is the one way a non-operator player may write a scoreboard score.
//! Vanilla `TriggerCommand` resolves the objective named by the player, refuses
//! it unless the criterion is `trigger` and the score was enabled by
//! `/scoreboard players enable`, sets the value, and relocks the score so the
//! next write needs another `enable`. `add` and `set` differ only in whether the
//! argument is added to the current value or replaces it.

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
use crate::scoreboard::{ScoreHolder, Scoreboard, ScoreboardObjective};
use crate::server::scoreboard_sync;

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    // Vanilla grants `/trigger` to everyone (`Commands.LEVEL_ALL`), which is the
    // reason it exists at all: a map maker ships a datapack and a player who is
    // not an operator can still poke the map's state.
    CommandRegistration::new(Identifier::vanilla_static("trigger"), |_| command())
        .default_access()
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("trigger")
        .then(
            argument("objective", ArgumentType::word())
                .executes(add_one)
                .then(literal("add").then(
                    argument("value", ArgumentType::integer(i32::MIN, i32::MAX))
                        .executes(add_value),
                ))
                .then(literal("set").then(
                    argument("value", ArgumentType::integer(i32::MIN, i32::MAX))
                        .executes(set_value),
                )),
        )
}

/// The scoreboard owned by the domain the command runs in.
fn scoreboard_of<'a>(
    context: &'a SteelCommandContext<CommandSource>,
) -> Result<&'a Scoreboard, CommandSyntaxError> {
    // The domain is read from the command source's world, like `/scoreboard`
    // does, so `/execute in <domain>` projects `/trigger` onto that scoreboard.
    let domain = context.source().world().domain().to_owned();
    context
        .source()
        .server()
        .scoreboards
        .get(&domain)
        .ok_or_else(|| CommandSyntaxError::dynamic(format!("Domain '{domain}' has no scoreboard")))
}

/// Vanilla resolves the objective by the *player's* name, and fails the whole
/// command when it is missing or not a `trigger` objective.
fn trigger_objective(
    context: &SteelCommandContext<CommandSource>,
    scoreboard: &Scoreboard,
) -> Result<ScoreboardObjective, CommandSyntaxError> {
    let name = context.string("objective")?.to_owned();
    let objective = scoreboard
        .objective(&name)
        .ok_or_else(|| CommandSyntaxError::dynamic(format!("Unknown objective '{name}'")))?;
    if !objective.is_trigger() {
        return Err(CommandSyntaxError::dynamic(format!(
            "Objective '{name}' is not a trigger objective"
        )));
    }
    Ok(objective)
}

/// The owning player and their score holder name, or an error for a non-player
/// sender (`/trigger` has no target argument, by design).
fn triggering_player(
    context: &SteelCommandContext<CommandSource>,
) -> Result<ScoreHolder, CommandSyntaxError> {
    let Some(player) = context.source().player() else {
        return Err(CommandSyntaxError::dynamic(
            "This command can only be run by a player",
        ));
    };
    // Vanilla uses `player.getScoreboardName()`, not the display name, so a
    // renamed nickname still writes the same score entry.
    Ok(scoreboard_sync::holder_name(player))
}

/// `count` for the command result and the vanilla success message.
fn report(
    context: &SteelCommandContext<CommandSource>,
    scoreboard: &Scoreboard,
    objective: &ScoreboardObjective,
    holder: &ScoreHolder,
    previous: i32,
    value: i32,
) -> Result<i32, CommandSyntaxError> {
    let domain = context.source().world().domain().to_owned();
    scoreboard_sync::sync_score_changed(
        context.source().server(),
        &domain,
        holder.name(),
        objective.name(),
        value,
    );
    context.source().send_success(
        &TextComponent::plain(format!(
            "Triggered {} (set to {value}, previously {previous})",
            objective.name()
        )),
        false,
    );
    Ok(1)
}

/// `/trigger <objective>`: the 1.13 shorthand for `add 1`, not `add 0`.
///
/// It still relocks the score, so a map that only needs a "button press" can
/// watch the score and re-`enable` afterwards.
fn add_one(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    add_amount(context, 1)
}

fn add_value(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    add_amount(context, context.integer("value")?)
}

/// Vanilla `TriggerCommand.executeAdd`: `score + amount` with Java `int` wrap.
fn add_amount(
    context: &SteelCommandContext<CommandSource>,
    amount: i32,
) -> Result<i32, CommandSyntaxError> {
    let scoreboard = scoreboard_of(context)?;
    let objective = trigger_objective(context, scoreboard)?;
    let holder = triggering_player(context)?;
    let previous = scoreboard.score(&holder, &objective).unwrap_or(0);
    apply_with_previous(
        context,
        scoreboard,
        &objective,
        &holder,
        previous,
        previous.wrapping_add(amount),
    )
}

/// `/trigger <objective> set <value>`: replaces the score outright.
fn set_value(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let value = context.integer("value")?;
    let scoreboard = scoreboard_of(context)?;
    let objective = trigger_objective(context, scoreboard)?;
    let holder = triggering_player(context)?;
    let previous = scoreboard.score(&holder, &objective).unwrap_or(0);
    apply_with_previous(context, scoreboard, &objective, &holder, previous, value)
}

fn apply_with_previous(
    context: &SteelCommandContext<CommandSource>,
    scoreboard: &Scoreboard,
    objective: &ScoreboardObjective,
    holder: &ScoreHolder,
    previous: i32,
    value: i32,
) -> Result<i32, CommandSyntaxError> {
    scoreboard
        .trigger_score(holder, objective, value)
        .map_err(|error| CommandSyntaxError::dynamic(error.to_string()))?;
    report(context, scoreboard, objective, holder, previous, value)
}
