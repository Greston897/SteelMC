//! Vanilla `/scoreboard` command.
//!
//! Covers the objective and player halves of the vanilla command graph. Team
//! management exists on `Scoreboard` but has no command tree yet.

use std::sync::Arc;

use steel_utils::Identifier;
use text_components::TextComponent;

use super::super::{
    brigadier::{ArgumentType, CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandSource, ScoreHolderWildcard, SteelArgumentType, SteelCommandContext,
        SteelCommandRuntime, argument, literal,
    },
    registration::CommandRegistration,
};
use crate::scoreboard::{ScoreHolder, Scoreboard, ScoreboardObjective};
use crate::server::scoreboard_sync;

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("scoreboard"), |_| command())
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("scoreboard")
        .then(objectives_node())
        .then(players_node())
}

fn objectives_node() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("objectives")
        .then(
            literal("add").then(
                argument("objective", ArgumentType::word())
                    .then(
                        argument("criteria", ArgumentType::word())
                            .executes(create_objective),
                    )
                    .executes(create_objective),
            ),
        )
        .then(
            literal("remove").then(
                argument("objective", SteelArgumentType::objective()).executes(remove_objective),
            ),
        )
        .then(literal("list").executes(list_objectives))
        .then(
            literal("modify").then(
                argument("objective", SteelArgumentType::objective()).then(
                    literal("displayname")
                        .then(argument("displayName", ArgumentType::word()).executes(set_display_name))
                        .then(literal("reset").executes(reset_display_name)),
                ),
            ),
        )
        .then(
            literal("setdisplay").then(
                argument("slot", ArgumentType::word()).then(
                    argument("objective", SteelArgumentType::objective()).executes(set_display),
                ),
            ),
        )
}

fn players_node() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    let targets = || {
        argument("targets", SteelArgumentType::score_holders())
            .then(argument("objective", SteelArgumentType::objective()))
    };

    literal("players")
        .then(
            literal("set").then(
                targets().then(
                    argument("score", ArgumentType::integer(i32::MIN, i32::MAX))
                        .executes(set_score),
                ),
            ),
        )
        .then(
            literal("add").then(
                targets().then(
                    argument("score", ArgumentType::integer(i32::MIN, i32::MAX))
                        .executes(add_score),
                ),
            ),
        )
        .then(
            literal("remove").then(
                targets().then(
                    argument("score", ArgumentType::integer(i32::MIN, i32::MAX))
                        .executes(remove_score_value),
                ),
            ),
        )
        .then(literal("reset").then(targets().executes(reset_score)))
        .then(literal("get").then(targets().executes(get_score)))
        .then(literal("enable").then(targets().executes(enable_trigger)))
}

/// Domain name owning the scoreboard the command runs against.
fn domain_of(context: &SteelCommandContext<CommandSource>) -> String {
    context.source().world().domain().to_owned()
}

fn scoreboard_of<'a>(
    context: &'a SteelCommandContext<CommandSource>,
) -> Result<&'a Scoreboard, CommandSyntaxError> {
    let source = context.source();
    let domain = source.world().domain().to_owned();
    let server = source.server();
    server.scoreboards.get(&domain).ok_or_else(|| {
        CommandSyntaxError::dynamic(format!("Domain '{domain}' has no scoreboard"))
    })
}

fn objective_of(
    context: &SteelCommandContext<CommandSource>,
    scoreboard: &Scoreboard,
) -> Result<ScoreboardObjective, CommandSyntaxError> {
    let name = context.objective_name("objective")?;
    scoreboard
        .objective(name)
        .ok_or_else(|| CommandSyntaxError::dynamic(format!("Unknown objective '{name}'")))
}

fn holders_of(
    context: &SteelCommandContext<CommandSource>,
) -> Result<Vec<ScoreHolder>, CommandSyntaxError> {
    context
        .score_holders("targets", ScoreHolderWildcard::Tracked)
        .map(|holders| {
            holders
                .into_iter()
                .map(|holder| ScoreHolder::new(holder.name().to_owned()))
                .collect()
        })
}

fn create_objective(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let scoreboard = scoreboard_of(context)?;
    let name = context.objective_name("objective")?;
    scoreboard
        .add_objective(name)
        .map_err(|error| CommandSyntaxError::dynamic(error.to_string()))?;
    // Vanilla `ServerScoreboard#onObjectiveAdded` pushes the new objective to
    // every tracker immediately, so the client can render it right away.
    scoreboard_sync::sync_objective_added(context.source().server(), &domain_of(context), name);
    context.source().send_success(
        &TextComponent::plain(format!("Created objective '{name}'")),
        false,
    );
    Ok(1)
}

fn remove_objective(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let scoreboard = scoreboard_of(context)?;
    let name = context.objective_name("objective")?;
    scoreboard
        .remove_objective(name)
        .map_err(|error| CommandSyntaxError::dynamic(error.to_string()))?;
    scoreboard_sync::sync_objective_removed(context.source().server(), &domain_of(context), name);
    context.source().send_success(
        &TextComponent::plain(format!("Removed objective '{name}'")),
        false,
    );
    Ok(1)
}

/// Vanilla `DisplaySlot` names accepted by `/scoreboard objectives setdisplay`.
const DISPLAY_SLOTS: &[(&str, i32)] = &[
    ("list", scoreboard_sync::display_slot::LIST),
    ("sidebar", scoreboard_sync::display_slot::SIDEBAR),
    ("below_name", 2),
];

fn set_display_name(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let scoreboard = scoreboard_of(context)?;
    let name = context.objective_name("objective")?;
    let display = context.string("displayName")?.to_owned();
    scoreboard
        .set_display_name(name, &display)
        .map_err(|error| CommandSyntaxError::dynamic(error.to_string()))?;
    scoreboard_sync::sync_objective_updated(
        context.source().server(),
        &domain_of(context),
        name,
        &display,
        scoreboard.is_read_only(name),
    );
    context.source().send_success(
        &TextComponent::plain(format!("Set display name of objective '{name}' to '{display}'")),
        false,
    );
    Ok(1)
}

fn reset_display_name(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let scoreboard = scoreboard_of(context)?;
    let name = context.objective_name("objective")?;
    // Vanilla resets the display name to the objective name itself.
    scoreboard
        .set_display_name(name, name)
        .map_err(|error| CommandSyntaxError::dynamic(error.to_string()))?;
    scoreboard_sync::sync_objective_updated(
        context.source().server(),
        &domain_of(context),
        name,
        name,
        scoreboard.is_read_only(name),
    );
    context.source().send_success(
        &TextComponent::plain(format!("Reset display name of objective '{name}'")),
        false,
    );
    Ok(1)
}

fn set_display(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let scoreboard = scoreboard_of(context)?;
    let slot_name = context.string("slot")?.to_owned();
    let Some((_, slot)) = DISPLAY_SLOTS.iter().find(|(name, _)| *name == slot_name) else {
        return Err(CommandSyntaxError::dynamic(format!(
            "Unknown display slot '{slot_name}'"
        )));
    };
    let objective = context.objective_name("objective")?.to_owned();
    if scoreboard.objective(&objective).is_none() {
        return Err(CommandSyntaxError::dynamic(format!(
            "Unknown objective '{objective}'"
        )));
    }
    scoreboard.set_display_slot(*slot, &objective);
    scoreboard_sync::sync_display_slot(
        context.source().server(),
        &domain_of(context),
        *slot,
        &objective,
    );
    context.source().send_success(
        &TextComponent::plain(format!("Set display slot '{slot_name}' to objective '{objective}'")),
        false,
    );
    Ok(1)
}

fn list_objectives(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let scoreboard = scoreboard_of(context)?;
    let names = scoreboard.objective_names();
    let message = if names.is_empty() {
        "There are no objectives".to_owned()
    } else {
        format!(
            "There are {} objectives: {}",
            names.len(),
            names.join(", ")
        )
    };
    context
        .source()
        .send_success(&TextComponent::plain(message), false);
    Ok(i32::try_from(names.len()).unwrap_or(0))
}

#[derive(Clone, Copy)]
enum ScoreOp {
    Set,
    Add,
    Remove,
}

fn write_scores(
    context: &SteelCommandContext<CommandSource>,
    op: ScoreOp,
) -> Result<i32, CommandSyntaxError> {
    let scoreboard = scoreboard_of(context)?;
    let objective = objective_of(context, &scoreboard)?;
    let value = context.integer("score")?;
    let holders = holders_of(context)?;
    for holder in &holders {
        let current = scoreboard.score(holder, &objective).unwrap_or(0);
        let next = match op {
            ScoreOp::Set => value,
            ScoreOp::Add => current.saturating_add(value),
            ScoreOp::Remove => current.saturating_sub(value),
        };
        scoreboard
            .set_score(holder, &objective, next)
            .map_err(|error| CommandSyntaxError::dynamic(error.to_string()))?;
        scoreboard_sync::sync_score_changed(
            context.source().server(),
            &domain_of(context),
            holder.name(),
            objective.name(),
            next,
        );
    }
    Ok(i32::try_from(holders.len()).unwrap_or(0))
}

fn set_score(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    write_scores(context, ScoreOp::Set)
}

fn add_score(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    write_scores(context, ScoreOp::Add)
}

fn remove_score_value(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    write_scores(context, ScoreOp::Remove)
}

fn reset_score(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let scoreboard = scoreboard_of(context)?;
    let objective = objective_of(context, &scoreboard)?;
    let holders = holders_of(context)?;
    for holder in &holders {
        scoreboard
            .remove_score(holder, &objective)
            .map_err(|error| CommandSyntaxError::dynamic(error.to_string()))?;
        scoreboard_sync::sync_score_removed(
            context.source().server(),
            &domain_of(context),
            holder.name(),
            objective.name(),
        );
    }
    Ok(i32::try_from(holders.len()).unwrap_or(0))
}

fn get_score(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let scoreboard = scoreboard_of(context)?;
    let objective = objective_of(context, &scoreboard)?;
    let holders = holders_of(context)?;
    let Some(holder) = holders.first() else {
        return Err(CommandSyntaxError::dynamic("No score holder matched"));
    };
    let Some(value) = scoreboard.score(holder, &objective) else {
        return Err(CommandSyntaxError::dynamic(format!(
            "Objective '{}' has no score for '{}'",
            objective.name(),
            holder.name()
        )));
    };
    context.source().send_success(
        &TextComponent::plain(format!(
            "{} has {value} [{}]",
            holder.name(),
            objective.name()
        )),
        false,
    );
    Ok(value)
}

fn enable_trigger(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let scoreboard = scoreboard_of(context)?;
    let objective = objective_of(context, &scoreboard)?;
    let holders = holders_of(context)?;
    for holder in &holders {
        scoreboard
            .set_score_locked(holder, &objective, false)
            .map_err(|error| CommandSyntaxError::dynamic(error.to_string()))?;
    }
    Ok(i32::try_from(holders.len()).unwrap_or(0))
}
