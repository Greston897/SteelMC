//! Vanilla `/msg` (aliases `tell`, `w`) private-message command.

use std::borrow::Cow;

use steel_protocol::packets::game::CSystemChat;
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

fn translated(key: &str, args: Vec<TextComponent>) -> TextComponent {
    TextComponent::translated(TranslatedMessage {
        key: Cow::Owned(key.to_owned()),
        fallback: None,
        args: Some(args.into_boxed_slice()),
    })
}

/// Vanilla `MsgCommand`: the sender sees `commands.message.display.outgoing`,
/// each recipient sees `commands.message.display.incoming`.
fn send_message(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let targets = context.players("targets")?;
    let message = context.string("message")?.to_owned();
    let sender = context.source().source_name();
    let sender_component = TextComponent::plain(sender.clone());
    let body = TextComponent::plain(message.clone());

    for target in &targets {
        target.send_packet(CSystemChat {
            content: translated(
                "commands.message.display.incoming",
                vec![sender_component.clone(), body.clone()],
            ),
            overlay: false,
        });
    }

    context.source().send_success(
        &translated(
            "commands.message.display.outgoing",
            vec![
                TextComponent::plain(
                    targets
                        .first()
                        .map_or_else(|| "?".to_owned(), |p| p.gameprofile.name().to_owned()),
                ),
                body,
            ],
        ),
        false,
    );
    Ok(i32::try_from(targets.len()).unwrap_or(0))
}
