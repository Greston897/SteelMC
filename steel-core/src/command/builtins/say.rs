//! Vanilla `/say` and `/me` broadcast commands.

use steel_registry::vanilla_chat_types;
use steel_utils::Identifier;
use text_components::TextComponent;

use super::super::{
    brigadier::{ArgumentType, CommandNodeBuilder, CommandSyntaxError},
    execution::{CommandSource, SteelCommandContext, SteelCommandRuntime, argument, literal},
    registration::CommandRegistration,
};

pub(super) fn say_registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("say"), |_| say_command())
}

pub(super) fn me_registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("me"), |_| me_command())
}

fn say_command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("say").then(argument("message", ArgumentType::greedy_string()).executes(broadcast_say))
}

fn me_command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("me").then(argument("action", ArgumentType::greedy_string()).executes(broadcast_me))
}

/// Vanilla `SayCommand`: broadcast through `minecraft:say_command`.
///
/// The chat type carries the `[sender] message` decoration, so the server sends
/// the plain message body and lets the client format it — that is what makes
/// `/say` respect the client's chat-visibility setting.
fn broadcast_say(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let message = context.string("message")?.to_owned();
    context.source().server().broadcast_decorated_chat(
        &vanilla_chat_types::SAY_COMMAND,
        &TextComponent::plain(message),
        context.source().display_name_component(),
        None,
    );
    Ok(1)
}

/// Vanilla `MeCommand`: broadcast through `minecraft:emote_command`.
fn broadcast_me(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let action = context.string("action")?.to_owned();
    context.source().server().broadcast_decorated_chat(
        &vanilla_chat_types::EMOTE_COMMAND,
        &TextComponent::plain(action),
        context.source().display_name_component(),
        None,
    );
    Ok(1)
}
