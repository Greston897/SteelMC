//! `/function` — runs a datapack function.
//!
//! Vanilla `FunctionCommand` resolves the identifier against the function
//! registry and queues every line of the body, so a function's commands run in
//! order in a child execution frame. The source and shared execution limits
//! survive nested functions, execute transformations, and suspended commands.
//!
//! Messages use vanilla's translation keys with a plain-text fallback, so the
//! command reads like vanilla even before the language file is generated.

use std::{borrow::Cow, sync::Arc};

use steel_utils::Identifier;
use text_components::TextComponent;
use text_components::translation::TranslatedMessage;

use super::super::{
    brigadier::{CommandNodeBuilder, CommandSyntaxError},
    execution::{
        ChainModifiers, CommandSource, CustomCommandExecutor, ExecutionCommandSource,
        ExecutionControl, CommandResultCallback, SteelArgumentType, SteelCommandRuntime, SteelContextChain,
        argument, literal,
    },
    registration::CommandRegistration,
};


const FUNCTION_ARGUMENT: &str = "function";

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("function"), |_| command())
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("function")
        .then(argument(FUNCTION_ARGUMENT, SteelArgumentType::identifier()).executes_custom(RunFunction))
}

/// Builds a translated component with a literal fallback.
fn translated(key: &'static str, fallback: String, args: Vec<TextComponent>) -> TextComponent {
    TextComponent::translated(TranslatedMessage {
        key: Cow::Borrowed(key),
        fallback: Some(Cow::Owned(fallback)),
        args: Some(args.into_boxed_slice()),
    })
}

struct RunFunction;

impl CustomCommandExecutor<CommandSource> for RunFunction {
    fn run(
        &self,
        source: Arc<CommandSource>,
        chain: &SteelContextChain<CommandSource>,
        modifiers: ChainModifiers,
        control: &mut ExecutionControl<'_, CommandSource>,
    ) {
        let result = (|| {
            let id = chain.top_context().identifier(FUNCTION_ARGUMENT)?;
            let commands = source.server().function_commands(id).ok_or_else(||
                CommandSyntaxError::dynamic(translated("commands.function.error.argument_mismatch",
                    format!("Can't find function {id}"), vec![TextComponent::from(id.to_string())])))?;
            // Parse the entire body first: an invalid line cannot leave half a
            // function queued. No console re-submission, privilege escalation,
            // lost execute context, independent limit, or silent queue truncation.
            let body_source = Arc::new(source.with_suppressed_output().with_callback(CommandResultCallback::empty()));
            let chains = commands.iter().map(|command|
                source.server().parse_function_command(command, &body_source))
                .collect::<Result<Vec<_>, _>>()?;
            let count = chains.len();
            let callback = if modifiers.is_return() {
                let frame = control.current_frame().clone();
                control.discard_frame();
                CommandResultCallback::new(move |success, value| {
                    if success { frame.return_success(value); } else { frame.return_failure(); }
                })
            } else { source.callback() };
            control.queue_function(chains, body_source, callback);
            source.send_success(&translated("commands.function.success.single",
                format!("Executing function {id} with {count} commands"),
                vec![TextComponent::from(id.to_string()), TextComponent::from(count.to_string())]), true);
            Ok::<(), CommandSyntaxError>(())
        })();
        if let Err(error) = result {
            source.handle_error(&error, modifiers.is_forked());
            source.callback().on_result(false, 0);
            if modifiers.is_return() { control.return_failure(); }
        }
    }
}
