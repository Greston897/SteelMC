//! Vanilla `/say` and `/me` broadcast commands.

use std::borrow::Cow;

use steel_utils::Identifier;
use text_components::TextComponent;
use text_components::translation::TranslatedMessage;

use super::super::{
    brigadier::{ArgumentType, CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandSource, SteelArgumentType, SteelCommandContext, SteelCommandRuntime, argument,
        literal,
    },
    registration::CommandRegistration,
};

pub(super) fn say_registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("say"), |_| say_command())
}

pub(super) fn me_registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("me"), |_| me_command())
}

fn say_command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("say")
        .then(argument("message", ArgumentType::greedy_string()).executes(broadcast_say))
}

fn me_command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("me")
        .then(argument("action", ArgumentType::greedy_string()).executes(broadcast_me))
}

fn translated(key: &str, args: Vec<TextComponent>) -> TextComponent {
    TextComponent::translated(TranslatedMessage {
        key: Cow::Owned(key.to_owned()),
        fallback: None,
        args: Some(args.into_boxed_slice()),
    })
}

/// Vanilla `SayCommand`: `[sender] message` broadcast to every player.
fn broadcast_say(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let message = context.string("message")?;
    let sender = context.source().source_name();
    let content = TextComponent::plain(format!("[{sender}] {message}"));
    context
        .source()
        .server()
        .broadcast_system_chat(&content, None);
    Ok(1)
}

/// Vanilla `MeCommand`: `chat.type.emote` with the sender and the action.
fn broadcast_me(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let action = context.string("action")?;
    let sender = TextComponent::plain(context.source().source_name());
    let content = translated(
        "chat.type.emote",
        vec![sender, TextComponent::plain(action)],
    );
    context
        .source()
        .server()
        .broadcast_system_chat(&content, None);
    Ok(1)
}
