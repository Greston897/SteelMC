//! `/schedule` — runs a command after a delay.
//!
//! Vanilla `ScheduleCommand` has two forms: `schedule function <id> <time>` and
//! `schedule clear <id>`. Both address schedules by the function identifier, so
//! the command is the first half of the datapack story — the second half,
//! `/function`, needs the function bodies, which Steel loads from its built-in
//! datapack.
//!
//! Steel's schedules live on the server and are drained by the game tick loop
//! (see [`crate::server::schedule`]).

use std::borrow::Cow;

use steel_utils::Identifier;
use text_components::translation::TranslatedMessage;
use text_components::TextComponent;

use super::super::{
    brigadier::{ArgumentType, CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandSource, SteelArgumentType, SteelCommandContext, SteelCommandRuntime, argument,
        literal,
    },
    registration::CommandRegistration,
};
use crate::server::schedule::ScheduleMode;

const FUNCTION_ARGUMENT: &str = "function";
const TIME_ARGUMENT: &str = "time";
const MODE_ARGUMENT: &str = "mode";

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("schedule"), |_| command())
}


/// Builds a translatable component with an English fallback.
///
/// The extracted `en_us.json` does not carry the `commands.schedule.*` keys yet,
/// so the fallback is what the client actually renders today; the key is kept so
/// the message switches to the packaged translation as soon as it lands.
fn schedule_message(key: &'static str, fallback: &str, args: Vec<TextComponent>) -> TextComponent {
    TextComponent::translated(TranslatedMessage {
        key: Cow::Borrowed(key),
        fallback: Some(Cow::Owned(fallback.to_owned())),
        args: Some(args.into_boxed_slice()),
    })
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    // `schedule function <id> <time> [replace|append]`
    let function = literal("function")
        .then(
            argument(FUNCTION_ARGUMENT, SteelArgumentType::identifier()).then(
                argument(TIME_ARGUMENT, SteelArgumentType::time(0))
                    .executes(|context| schedule_function(context, ScheduleMode::Replace))
                    .then(
                        argument(MODE_ARGUMENT, ArgumentType::greedy_string())
                            .executes(parse_mode),
                    ),
            ),
        );

    // `schedule clear <id>`
    let clear = literal("clear")
        .then(argument(FUNCTION_ARGUMENT, SteelArgumentType::identifier()).executes(clear_id));

    literal("schedule").then(function).then(clear)
}

/// `schedule function <id> <time> <mode>`: the mode is validated by hand
/// because vanilla exposes it as two literals rather than a parser.
fn parse_mode(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let mode = context.string(MODE_ARGUMENT)?;
    match ScheduleMode::parse(mode) {
        Some(ScheduleMode::Append) => schedule_function(context, ScheduleMode::Append),
        // `replace` and an unknown mode both fall back to vanilla's default.
        _ => schedule_function(context, ScheduleMode::Replace),
    }
}

fn schedule_function(
    context: &SteelCommandContext<CommandSource>,
    mode: ScheduleMode,
) -> Result<i32, CommandSyntaxError> {
    let function = context.identifier(FUNCTION_ARGUMENT)?.clone();
    let delay = context.time(TIME_ARGUMENT)?;
    let Ok(delay) = u64::try_from(delay) else {
        return Err(CommandSyntaxError::dynamic(TextComponent::plain(
            "Schedule delay must not be negative",
        )));
    };

    let server = context.source().server();
    let due_tick = server.current_tick().saturating_add(delay);
    let replaced = server
        .schedules
        .schedule_function(&function, due_tick, mode);

    let message = if replaced == 0 {
        schedule_message(
            "commands.schedule.created.function",
            "Scheduled function %s in %s ticks at gametime %s",
            vec![TextComponent::from(function.to_string())],
        )
    } else {
        schedule_message(
            "commands.schedule.created.function",
            "Scheduled function %s in %s ticks at gametime %s",
            vec![
                TextComponent::from(function.to_string()),
                TextComponent::from(replaced.to_string()),
            ],
        )
    };
    context.source().send_success(&message, false);
    Ok(1)
}

fn clear_id(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let function = context.identifier(FUNCTION_ARGUMENT)?;
    let removed = context.source().server().schedules.clear(&function.to_string());

    let message = if removed == 0 {
        TextComponent::from("No scheduled function was found")
    } else {
        schedule_message(
            "commands.schedule.cleared.success",
            "Removed %s scheduled functions",
            vec![TextComponent::from(removed.to_string())],
        )
    };
    context.source().send_success(&message, false);
    Ok(removed as i32)
}
