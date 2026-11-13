//! `/function` — runs a datapack function.
//!
//! Vanilla `FunctionCommand` resolves the identifier against the function
//! registry and queues every line of the body, so a function's commands run in
//! order on the tick it was submitted. Steel submits the lines through the
//! server's ordinary command queue, which preserves their order while keeping
//! the tick loop bounded.
//!
//! Messages use vanilla's translation keys with a plain-text fallback, so the
//! command reads like vanilla even before the language file is generated.

use std::borrow::Cow;

use steel_utils::Identifier;
use text_components::TextComponent;
use text_components::translation::TranslatedMessage;

use super::super::{
    brigadier::{CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandSource, SteelArgumentType, SteelCommandContext, SteelCommandRuntime, argument,
        literal,
    },
    registration::CommandRegistration,
};
use crate::command::sender::CommandSender;

const FUNCTION_ARGUMENT: &str = "function";

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("function"), |_| command())
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("function")
        .then(argument(FUNCTION_ARGUMENT, SteelArgumentType::identifier()).executes(run_function))
}

/// Builds a translated component with a literal fallback.
fn translated(key: &'static str, fallback: String, args: Vec<TextComponent>) -> TextComponent {
    TextComponent::translated(TranslatedMessage {
        key: Cow::Borrowed(key),
        fallback: Some(fallback),
        args: Some(args.into_boxed_slice()),
    })
}

fn run_function(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let function_id = context.identifier(FUNCTION_ARGUMENT)?.clone();
    let Some(function) = steel_registry::REGISTRY.functions.by_key(&function_id) else {
        return Err(CommandSyntaxError::dynamic(translated(
            "commands.function.error.argument_mismatch",
            format!("Can't find function {}", function_id),
            vec![TextComponent::from(function_id.to_string())],
        )));
    };

    let server = context.source().server();
    let mut queued = 0;
    for command in function.commands {
        if server
            .submit_command(CommandSender::Console, (*command).to_owned())
            .is_err()
        {
            // The queue is full: stop rather than silently skipping a line,
            // since the rest of the function may depend on it.
            break;
        }
        queued += 1;
    }

    let message = translated(
        "commands.function.success.single",
        format!("Executed function {} with {} commands", function_id, queued),
        vec![
            TextComponent::from(function_id.to_string()),
            TextComponent::from(queued.to_string()),
        ],
    );
    context.source().send_success(&message, true);
    Ok(queued)
}
