//! `/bossbar` — create and modify server boss bars.
//!
//! Mirrors vanilla `BossBarCommand`: `add`, `remove`, `list`, `get`, and the
//! `set` children (`name`, `color`, `style`, `value`, `max`, `visible`, `players`).

use steel_protocol::packets::game::{BossBarColor, BossBarOverlay, BossEventAction, CBossEvent};
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
use crate::server::boss_bars::BossBar;

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("bossbar"), |_| command())
}

/// Every valid bar colour, in vanilla ordinal order.
const COLORS: [BossBarColor; 7] = [
    BossBarColor::Pink,
    BossBarColor::Blue,
    BossBarColor::Red,
    BossBarColor::Green,
    BossBarColor::Yellow,
    BossBarColor::Purple,
    BossBarColor::White,
];

/// Every valid bar overlay, in vanilla ordinal order.
const OVERLAYS: [BossBarOverlay; 5] = [
    BossBarOverlay::Progress,
    BossBarOverlay::Notched6,
    BossBarOverlay::Notched10,
    BossBarOverlay::Notched12,
    BossBarOverlay::Notched20,
];

/// Command-visible name of a colour, matching vanilla's `bossbar set color` literals.
const fn color_name(color: BossBarColor) -> &'static str {
    match color {
        BossBarColor::Pink => "pink",
        BossBarColor::Blue => "blue",
        BossBarColor::Red => "red",
        BossBarColor::Green => "green",
        BossBarColor::Yellow => "yellow",
        BossBarColor::Purple => "purple",
        BossBarColor::White => "white",
    }
}

/// Command-visible name of an overlay, matching vanilla's `bossbar set style` literals.
const fn overlay_name(overlay: BossBarOverlay) -> &'static str {
    match overlay {
        BossBarOverlay::Progress => "progress",
        BossBarOverlay::Notched6 => "notched_6",
        BossBarOverlay::Notched10 => "notched_10",
        BossBarOverlay::Notched12 => "notched_12",
        BossBarOverlay::Notched20 => "notched_20",
    }
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    let mut body = literal("add")
        .then(
            argument("id", ArgumentType::greedy_string())
                .then(argument("name", SteelArgumentType::component()).executes(add_with_name))
                .executes(add_with_id),
        )
        .then(
            literal("remove")
                .then(argument("id", ArgumentType::greedy_string()).executes(remove_bar)),
        )
        .then(literal("list").executes(list_bars))
        .then(
            literal("get")
                .then(argument("id", ArgumentType::greedy_string()).executes(get_bar)),
        );

    body = body.then(set_commands());

    literal("bossbar").then(body)
}

fn set_commands() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    let name = literal("name").then(
        argument("name", SteelArgumentType::component()).executes(set_name),
    );

    let mut color = literal("color");
    for value in COLORS {
        color = color.then(
            literal(color_name(value)).executes(move |context| set_color(context, value)),
        );
    }

    let mut style = literal("style");
    for value in OVERLAYS {
        style = style.then(
            literal(overlay_name(value)).executes(move |context| set_style(context, value)),
        );
    }

    let value = literal("value")
        .then(argument("value", ArgumentType::integer(0, i32::MAX)).executes(set_value));

    let max = literal("max")
        .then(argument("value", ArgumentType::integer(1, i32::MAX)).executes(set_max));

    let visible =
        literal("visible").then(argument("visible", ArgumentType::bool()).executes(set_visible));

    let players = literal("players")
        .then(argument("players", SteelArgumentType::players()).executes(set_players));

    literal("set")
        .then(name)
        .then(color)
        .then(style)
        .then(value)
        .then(max)
        .then(visible)
        .then(players)
}

fn add_with_id(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let id = context.string("id")?.to_owned();
    let name = TextComponent::plain(id.clone());
    add_bar(context, id, name)
}

fn add_with_name(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let id = context.string("id")?.to_owned();
    let name = context.text_component("name")?.clone();
    add_bar(context, id, name)
}

fn add_bar(
    context: &SteelCommandContext<CommandSource>,
    id: String,
    name: TextComponent,
) -> Result<i32, CommandSyntaxError> {
    let registry = &context.source().server().boss_bars;
    if registry.get(&id).is_some() {
        return Err(CommandSyntaxError::dynamic(format!(
            "A boss bar already exists with id {id}"
        )));
    }

    let bar = BossBar {
        uuid: uuid_for(&id),
        name,
        progress: 1.0,
        color: BossBarColor::White,
        overlay: BossBarOverlay::Progress,
        flags: 0,
        visible: true,
    };

    // Vanilla shows a freshly added bar to every online player.
    for player in context.source().server().get_players() {
        player.send_packet(bar.add_packet());
    }

    let bar_id = id.clone();
    registry.insert(id, bar);

    context.source().send_success(
        &TextComponent::plain(format!("Created custom boss bar [{bar_id}]")),
        true,
    );

    Ok(1)
}

fn remove_bar(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let id = context.string("id")?;
    let registry = &context.source().server().boss_bars;
    let Some(bar) = registry.get(id) else {
        return Err(no_such_bar(id));
    };

    for player in context.source().server().get_players() {
        player.send_packet(bar.remove_packet());
    }
    registry.remove(id);

    context.source().send_success(
        &TextComponent::plain(format!("Removed custom boss bar [{id}]")),
        true,
    );

    Ok(1)
}

fn list_bars(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let registry = &context.source().server().boss_bars;
    let ids = registry.ids();

    let message = if ids.is_empty() {
        "There are no custom boss bars".to_string()
    } else {
        format!(
            "There are {} custom boss bars active: {}",
            ids.len(),
            ids.join(", ")
        )
    };
    context
        .source()
        .send_success(&TextComponent::plain(message), false);

    i32::try_from(ids.len())
        .map_err(|_| CommandSyntaxError::dynamic("Too many boss bars".to_string()))
}

fn get_bar(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let id = context.string("id")?;
    let Some(bar) = context.source().server().boss_bars.get(id) else {
        return Err(no_such_bar(id));
    };

    let text = format!(
        "Custom boss bar [{id}]: color={}, style={}, value={}, visible={}",
        color_name(bar.color),
        overlay_name(bar.overlay),
        bar.progress,
        bar.visible
    );
    context
        .source()
        .send_success(&TextComponent::plain(text), false);

    Ok(1)
}

fn set_name(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let name = context.text_component("name")?.clone();
    let id = context.string("id")?.to_owned();
    let bar = update_bar(context, |bar| {
        bar.name = name.clone();
        let packet = CBossEvent {
            uuid: bar.uuid,
            action: BossEventAction::UpdateName { name: name.clone() },
        };
        broadcast(context, packet);
    })?;
    let _ = &bar;

    context
        .source()
        .send_success(&TextComponent::plain(id), true);

    Ok(1)
}

fn set_color(
    context: &SteelCommandContext<CommandSource>,
    color: BossBarColor,
) -> Result<i32, CommandSyntaxError> {
    let bar = update_bar(context, |bar| {
        bar.color = color;
        broadcast_style(context, bar);
    })?;

    context
        .source()
        .send_success(&TextComponent::plain(color_name(bar.color)), true);

    Ok(1)
}

fn set_style(
    context: &SteelCommandContext<CommandSource>,
    overlay: BossBarOverlay,
) -> Result<i32, CommandSyntaxError> {
    let bar = update_bar(context, |bar| {
        bar.overlay = overlay;
        broadcast_style(context, bar);
    })?;

    context
        .source()
        .send_success(&TextComponent::plain(overlay_name(bar.overlay)), true);

    Ok(1)
}

fn set_value(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let value = context.integer("value")?;
    let bar = update_bar(context, |bar| {
        // The command's value is absolute in `0..=max`; Steel stores progress.
        bar.progress = f64::from(value).clamp(0.0, 1.0) as f32;
    })?;

    context
        .source()
        .send_success(&TextComponent::plain(value.to_string()), true);

    Ok(1)
}

fn set_max(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let max = context.integer("value")?;
    let bar = update_bar(context, |bar| {
        // Vanilla rescales the current value against the new maximum. Steel keeps
        // progress directly, so the percentage is preserved as-is.
        bar.progress = bar.progress.clamp(0.0, 1.0);
    })?;

    context
        .source()
        .send_success(&TextComponent::plain(max.to_string()), true);

    Ok(1)
}

fn set_visible(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let visible = context.boolean("visible")?;
    let bar = update_bar(context, |bar| {
        bar.visible = visible;
        let packet = if visible {
            bar.add_packet()
        } else {
            bar.remove_packet()
        };
        broadcast(context, packet);
    })?;

    context
        .source()
        .send_success(&TextComponent::plain(bar.visible.to_string()), true);

    Ok(1)
}

fn set_players(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let targets = context.players("players")?;
    let bar = update_bar(context, |_| {})?;

    // Vanilla replaces the viewer set: everyone else stops seeing the bar.
    for player in context.source().server().get_players() {
        if !targets.iter().any(|target| Arc::ptr_eq(target, &player)) {
            player.send_packet(bar.remove_packet());
        }
    }
    if bar.visible {
        for player in &targets {
            player.send_packet(bar.add_packet());
        }
    }

    let count = i32::try_from(targets.len())
        .map_err(|_| CommandSyntaxError::dynamic("Too many target players".to_string()))?;
    context
        .source()
        .send_success(&TextComponent::plain(count.to_string()), true);

    Ok(count)
}

/// Applies `f` to the addressed bar and returns its new state.
fn update_bar<F>(
    context: &SteelCommandContext<CommandSource>,
    f: F,
) -> Result<BossBar, CommandSyntaxError>
where
    F: FnOnce(&mut BossBar),
{
    let id = context.string("id")?;
    context
        .source()
        .server()
        .boss_bars
        .update(id, f)
        .ok_or_else(|| no_such_bar(id))
}

/// Sends one packet to every online player.
fn broadcast(context: &SteelCommandContext<CommandSource>, packet: CBossEvent) {
    for player in context.source().server().get_players() {
        player.send_packet(packet.clone());
    }
}

/// Re-sends a bar's colour and overlay to every online player.
fn broadcast_style(context: &SteelCommandContext<CommandSource>, bar: &BossBar) {
    let packet = CBossEvent {
        uuid: bar.uuid,
        action: BossEventAction::UpdateStyle {
            color: bar.color,
            overlay: bar.overlay,
        },
    };
    broadcast(context, packet);
}

fn no_such_bar(id: &str) -> CommandSyntaxError {
    CommandSyntaxError::dynamic(format!("No boss bar exists with id {id}"))
}

/// Stable wire id derived from the command-visible bar name.
///
/// Vanilla creates a random UUID per bar; a name-derived v5 UUID keeps the id
/// stable across a `/reload` cycle, which matters because clients key their
/// bars by UUID.
fn uuid_for(id: &str) -> uuid::Uuid {
    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, id.as_bytes())
}
