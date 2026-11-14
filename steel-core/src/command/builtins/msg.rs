//! Vanilla `/msg` (aliases `tell`, `w`) private-message command.

use crate::entity::Entity;
use steel_registry::vanilla_chat_types;
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

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("msg"), |_| command())
        .alias("tell")
        .alias("w")
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("msg").then(
        argument("targets", SteelArgumentType::players()).then(
            argument("message", ArgumentType::greedy_string()).executes(send_message),
        ),
    )
}

/// Vanilla `MsgCommand`: every recipient sees `minecraft:msg_command_incoming`,
/// the sender sees `minecraft:msg_command_outgoing`.
///
/// Both go through `disguised_chat`: the chat type carries the decoration and
/// the `target` parameter, so the *client* renders "you whisper to X" instead
/// of the server pre-formatting a system message.
fn send_message(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let targets = context.players("targets")?;
    let message = context.string("message")?.to_owned();
    let source = context.source();
    let server = source.server();
    let sender_component = source.display_name_component();
    let body = TextComponent::plain(message);

    for target in &targets {
        server.broadcast_decorated_chat_to(
            &vanilla_chat_types::MSG_COMMAND_INCOMING,
            &body,
            sender_component.clone(),
            None,
            std::slice::from_ref(target),
        );
    }

    // Vanilla sends the outgoing line to the sender only, with the target name
    // resolved to the first recipient. A console/Rcon sender has no chat window,
    // so its "outgoing" line goes to the command feedback (as vanilla does for
    // any non-player source).
    let outgoing_target = targets.first().map(|target| target.display_name());
    match source.player() {
        Some(player) => server.broadcast_decorated_chat_to(
            &vanilla_chat_types::MSG_COMMAND_OUTGOING,
            &body,
            sender_component,
            outgoing_target,
            std::slice::from_ref(player),
        ),
        None => source.send_success(
            &crate::server::broadcasting::decorated_chat_component(
                &vanilla_chat_types::MSG_COMMAND_OUTGOING,
                &sender_component,
                outgoing_target.as_ref(),
                &body,
            ),
            false,
        ),
    }
    Ok(i32::try_from(targets.len()).unwrap_or(0))
}
