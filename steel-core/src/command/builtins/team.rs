//! `/team` — manage scoreboard teams.
//!
//! Implements vanilla's `TeamCommand`: `add`, `remove`, `empty`, `join`, `leave`,
//! `list`, and the `modify` children that map onto [`TeamState`].

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
use crate::scoreboard::{ScoreHolder, TeamState, TeamVisibility};

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("team"), |_| command())
}

/// Vanilla `TeamCommand` literals, in the order the help listing shows them.
const VISIBILITIES: [TeamVisibility; 4] = [
    TeamVisibility::Always,
    TeamVisibility::HideForOtherTeams,
    TeamVisibility::HideForOwnTeam,
    TeamVisibility::Never,
];

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("team")
        .then(
            literal("add").then(
                argument("team", ArgumentType::word()).executes(add_team),
            ),
        )
        .then(
            literal("remove").then(
                argument("team", ArgumentType::word()).executes(remove_team),
            ),
        )
        .then(
            literal("empty").then(
                argument("team", ArgumentType::word()).executes(empty_team),
            ),
        )
        .then(
            literal("join").then(
                argument("team", ArgumentType::word())
                    .then(argument("members", SteelArgumentType::score_holders())
                        .executes(join_team)),
            ),
        )
        .then(
            literal("leave").then(
                argument("members", SteelArgumentType::score_holders())
                    .executes(leave_team),
            ),
        )
        .then(literal("list").then(argument("team", ArgumentType::word()).executes(list_members)))
        .then(literal("list").executes(list_teams))
        .then(modify_commands())
}

fn modify_commands() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    let mut modify = literal("modify").then(
        argument("team", ArgumentType::word()),
    );

    // Friendly fire
    let mut friendly_fire = literal("friendlyFire");
    for (name, allowed) in [("true", true), ("false", false)] {
        friendly_fire = friendly_fire.then(literal(name).executes(move |context| {
            modify_team(context, move |team| team.allow_friendly_fire = allowed)
        }));
    }

    // Colour
    let mut color = literal("color");
    for name in COLOR_NAMES {
        color = color.then(literal(*name).executes(move |context| {
            modify_team(context, move |team| team.color = Some((*name).to_string()))
        }));
    }
    color = color.then(literal("reset").executes(|context| {
        modify_team(context, |team| team.color = None)
    }));

    // Visibility triples
    let mut visibility = literal("visibility");
    let mut nametag = literal("nametagVisibility");
    let mut death_message = literal("deathMessageVisibility");
    for value in VISIBILITIES {
        let name = value.name();
        visibility = visibility.then(
            literal(name).executes(move |context| {
                modify_team(context, move |team| team.visibility = value)
            }),
        );
        nametag = nametag.then(
            literal(name).executes(move |context| {
                modify_team(context, move |team| team.name_tag_visibility = value)
            }),
        );
        death_message = death_message.then(
            literal(name).executes(move |context| {
                modify_team(context, move |team| team.death_message_visibility = value)
            }),
        );
    }

    // Prefix / suffix
    let prefix = literal("prefix").then(
        argument("prefix", SteelArgumentType::component())
            .executes(|context| modify_component(context, ComponentPart::Prefix)),
    );
    let suffix = literal("suffix").then(
        argument("suffix", SteelArgumentType::component())
            .executes(|context| modify_component(context, ComponentPart::Suffix)),
    );

    modify = modify
        .then(friendly_fire)
        .then(color)
        .then(visibility)
        .then(nametag)
        .then(death_message)
        .then(prefix)
        .then(suffix);

    modify
}

const COLOR_NAMES: &[&str] = &[
    "black", "dark_blue", "dark_green", "dark_aqua", "dark_red", "dark_purple", "gold", "gray",
    "dark_gray", "blue", "green", "aqua", "red", "light_purple", "yellow", "white",
];

enum ComponentPart {
    Prefix,
    Suffix,
}

fn add_team(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let name = context.string("team")?.to_owned();
    let scoreboard = scoreboard(context)?;
    scoreboard
        .add_team(name.clone())
        .map_err(|error| CommandSyntaxError::dynamic(error.to_string()))?;

    context.source().send_success(
        &TextComponent::plain(format!("Created new team {name}")),
        true,
    );
    Ok(1)
}

fn remove_team(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let name = context.string("team")?.to_owned();
    let scoreboard = scoreboard(context)?;
    if !scoreboard.remove_team(&name) {
        return Err(CommandSyntaxError::dynamic(format!(
            "Team {name} does not exist"
        )));
    }

    context.source().send_success(
        &TextComponent::plain(format!("Removed team {name}")),
        true,
    );
    Ok(1)
}

fn empty_team(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let name = context.string("team")?.to_owned();
    let scoreboard = scoreboard(context)?;

    let members: Vec<String> = scoreboard
        .team_members(&name)
        .into_iter()
        .collect();
    if members.is_empty() {
        return Err(CommandSyntaxError::dynamic(format!(
            "Team {name} has no members"
        )));
    }
    for member in &members {
        scoreboard.remove_holder_from_team(&ScoreHolder::new(member.clone()));
    }

    context.source().send_success(
        &TextComponent::plain(format!("Emptied team {name}")),
        true,
    );
    i32::try_from(members.len())
        .map_err(|_| CommandSyntaxError::dynamic("Too many team members".to_string()))
}

fn join_team(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let name = context.string("team")?.to_owned();
    let holders = context.score_holders("members", crate::command::execution::ScoreHolderWildcard::Tracked)?;
    let scoreboard = scoreboard(context)?;

    let Some(team) = scoreboard.team(&name) else {
        return Err(CommandSyntaxError::dynamic(format!(
            "Team {name} does not exist"
        )));
    };

    for holder in &holders {
        scoreboard
            .add_holder_to_team(holder, &team)
            .map_err(|error| CommandSyntaxError::dynamic(error.to_string()))?;
    }

    context.source().send_success(
        &TextComponent::plain(format!(
            "Added {} member(s) to team {name}",
            holders.len()
        )),
        true,
    );
    i32::try_from(holders.len())
        .map_err(|_| CommandSyntaxError::dynamic("Too many members".to_string()))
}

fn leave_team(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let holders = context.score_holders("members", crate::command::execution::ScoreHolderWildcard::Tracked)?;
    let scoreboard = scoreboard(context)?;

    let mut removed = 0;
    for holder in &holders {
        if scoreboard.remove_holder_from_team(holder) {
            removed += 1;
        }
    }

    context.source().send_success(
        &TextComponent::plain(format!("Removed {removed} member(s) from their teams")),
        true,
    );
    Ok(removed)
}

fn list_members(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let name = context.string("team")?.to_owned();
    let scoreboard = scoreboard(context)?;

    let members: Vec<String> = scoreboard.team_members(&name).into_iter().collect();
    let message = if members.is_empty() {
        format!("Team {name} has no members")
    } else {
        format!("Team {name} has {} member(s): {}", members.len(), members.join(", "))
    };
    context
        .source()
        .send_success(&TextComponent::plain(message), false);

    i32::try_from(members.len())
        .map_err(|_| CommandSyntaxError::dynamic("Too many members".to_string()))
}

fn list_teams(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let teams = scoreboard(context)?.team_names();
    let message = if teams.is_empty() {
        "There are no teams".to_string()
    } else {
        format!("There are {} team(s): {}", teams.len(), teams.join(", "))
    };
    context
        .source()
        .send_success(&TextComponent::plain(message), false);

    i32::try_from(teams.len())
        .map_err(|_| CommandSyntaxError::dynamic("Too many teams".to_string()))
}

fn modify_team<F>(
    context: &SteelCommandContext<CommandSource>,
    f: F,
) -> Result<i32, CommandSyntaxError>
where
    F: FnOnce(&mut TeamState),
{
    let name = context.string("team")?.to_owned();
    let scoreboard = scoreboard(context)?;
    if scoreboard.update_team(&name, f).is_none() {
        return Err(CommandSyntaxError::dynamic(format!(
            "Team {name} does not exist"
        )));
    }

    context.source().send_success(
        &TextComponent::plain(format!("Modified team {name}")),
        true,
    );
    Ok(1)
}

fn modify_component(
    context: &SteelCommandContext<CommandSource>,
    part: ComponentPart,
) -> Result<i32, CommandSyntaxError> {
    let field = match part {
        ComponentPart::Prefix => "prefix",
        ComponentPart::Suffix => "suffix",
    };
    let text = context.string(field)?.to_owned();
    modify_team(context, move |team| match part {
        ComponentPart::Prefix => team.prefix = text.clone(),
        ComponentPart::Suffix => team.suffix = text.clone(),
    })
}

fn scoreboard<'a>(
    context: &'a SteelCommandContext<CommandSource>,
) -> Result<&'a crate::scoreboard::Scoreboard, CommandSyntaxError> {
    let domain = context.source().world().domain();
    context
        .source()
        .server()
        .scoreboards
        .get(domain)
        .ok_or_else(|| CommandSyntaxError::dynamic("Scoreboard unavailable".to_string()))
}
