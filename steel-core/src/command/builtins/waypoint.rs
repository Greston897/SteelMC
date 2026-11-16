//! Vanilla MC 26.3 `/waypoint` command (`WaypointCommand`).
//!
//! Supports:
//! - `/waypoint list` — lists active locator-bar transmitters in the dimension.
//! - `/waypoint modify <player> color (<color>|hex <rrggbb>|reset)`
//! - `/waypoint modify <player> style (reset|set <style>)`

use steel_utils::{Identifier, translations};
use text_components::TextComponent;

use super::super::{
    brigadier::{ArgumentType, CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandSource, SteelArgumentType, SteelCommandContext, SteelCommandRuntime, argument,
        literal,
    },
    registration::CommandRegistration,
};

/// Exact vanilla `ChatFormatting` RGB triples for `/waypoint modify ... color <name>`.
const NAMED_COLORS: &[(&str, [u8; 3])] = &[
    ("black", [0x00, 0x00, 0x00]),
    ("dark_blue", [0x00, 0x00, 0xAA]),
    ("dark_green", [0x00, 0xAA, 0x00]),
    ("dark_aqua", [0x00, 0xAA, 0xAA]),
    ("dark_red", [0xAA, 0x00, 0x00]),
    ("dark_purple", [0xAA, 0x00, 0xAA]),
    ("gold", [0xFF, 0xAA, 0x00]),
    ("gray", [0xAA, 0xAA, 0xAA]),
    ("dark_gray", [0x55, 0x55, 0x55]),
    ("blue", [0x55, 0x55, 0xFF]),
    ("green", [0x55, 0xFF, 0x55]),
    ("aqua", [0x55, 0xFF, 0xFF]),
    ("red", [0xFF, 0x55, 0x55]),
    ("light_purple", [0xFF, 0x55, 0xFF]),
    ("yellow", [0xFF, 0xFF, 0x55]),
    ("white", [0xFF, 0xFF, 0xFF]),
];

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("waypoint"), |_| command())
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    let mut color = literal("color")
        .then(literal("reset").executes(|ctx| set_color(ctx, None, "")))
        .then(
            literal("hex").then(
                argument("color", ArgumentType::word()).executes(set_hex_color),
            ),
        );
    for &(name, rgb) in NAMED_COLORS {
        color = color.then(literal(name).executes(move |ctx| set_color(ctx, Some(rgb), name)));
    }

    let style = literal("style")
        .then(literal("reset").executes(reset_style))
        .then(
            literal("set").then(
                argument("style", SteelArgumentType::identifier()).executes(set_style),
            ),
        );

    literal("waypoint")
        .then(literal("list").executes(list_waypoints))
        .then(
            literal("modify").then(
                argument("waypoint", SteelArgumentType::player())
                    .then(color)
                    .then(style),
            ),
        )
}

fn list_waypoints(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let source = context.source();
    let world = source.world();
    let dimension = world.key.to_string();
    let mut names = Vec::new();
    world.players.iter_players(|uuid, player| {
        if world.waypoints.transmitters().contains(uuid) {
            names.push(player.gameprofile.name().to_owned());
        }
        true
    });
    names.sort_unstable();

    if names.is_empty() {
        source.send_success(
            &TextComponent::translated(
                translations::COMMANDS_WAYPOINT_LIST_EMPTY
                    .message([TextComponent::plain(dimension)]),
            ),
            false,
        );
        return Ok(0);
    }

    let count = names.len() as i32;
    source.send_success(
        &TextComponent::plain(format!(
            "There are {count} waypoint(s) in {dimension}: {}",
            names.join(", ")
        )),
        false,
    );
    Ok(count)
}

fn set_color(
    context: &SteelCommandContext<CommandSource>,
    color: Option<[u8; 3]>,
    label: &str,
) -> Result<i32, CommandSyntaxError> {
    let target = context.player("waypoint")?;
    let world = target.get_world();
    world
        .waypoints
        .set_player_icon_color(target.gameprofile.id(), color);
    if color.is_some() {
        context.source().send_success(
            &TextComponent::translated(
                translations::COMMANDS_WAYPOINT_MODIFY_COLOR
                    .message([TextComponent::plain(label.to_owned())]),
            ),
            false,
        );
    } else {
        context.source().send_success(
            &TextComponent::translated(translations::COMMANDS_WAYPOINT_MODIFY_COLOR_RESET.msg()),
            false,
        );
    }
    Ok(1)
}

fn set_hex_color(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let raw = context.string("color")?;
    let trimmed = raw.strip_prefix('#').unwrap_or(raw);
    if trimmed.len() != 6 {
        return Err(CommandSyntaxError::dynamic(format!(
            "Invalid 6-digit hex color '{raw}'"
        )));
    }
    let parsed = u32::from_str_radix(trimmed, 16)
        .map_err(|_| CommandSyntaxError::dynamic(format!("Invalid 6-digit hex color '{raw}'")))?;
    let rgb = [
        ((parsed >> 16) & 0xFF) as u8,
        ((parsed >> 8) & 0xFF) as u8,
        (parsed & 0xFF) as u8,
    ];
    let hex_upper = trimmed.to_ascii_uppercase();
    set_color(context, Some(rgb), &hex_upper)
}

fn reset_style(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let target = context.player("waypoint")?;
    target
        .get_world()
        .waypoints
        .reset_player_icon_style(target.gameprofile.id());
    context.source().send_success(
        &TextComponent::translated(translations::COMMANDS_WAYPOINT_MODIFY_STYLE.msg()),
        false,
    );
    Ok(1)
}

fn set_style(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let target = context.player("waypoint")?;
    let style = context.identifier("style")?.clone();
    target
        .get_world()
        .waypoints
        .set_player_icon_style(target.gameprofile.id(), style);
    context.source().send_success(
        &TextComponent::translated(translations::COMMANDS_WAYPOINT_MODIFY_STYLE.msg()),
        false,
    );
    Ok(1)
}
