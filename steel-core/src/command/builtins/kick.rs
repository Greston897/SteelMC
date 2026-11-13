//! Vanilla `/kick` command.

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
    CommandRegistration::new(Identifier::vanilla_static("kick"), |_| command())
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("kick").then(
        argument("targets", SteelArgumentType::players())
            .then(argument("reason", ArgumentType::greedy_string()).executes(kick))
            .executes(kick_default),
    )
}

fn kick(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let reason = context.string("reason")?.to_owned();
    do_kick(context, TextComponent::plain(reason))
}

/// Vanilla default: `multiplayer.disconnect.kicked`.
fn kick_default(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    do_kick(
        context,
        TextComponent::translated(TranslatedMessage {
            key: std::borrow::Cow::Borrowed("multiplayer.disconnect.kicked"),
            fallback: None,
            args: None,
        }),
    )
}

fn do_kick(
    context: &SteelCommandContext<CommandSource>,
    reason: TextComponent,
) -> Result<i32, CommandSyntaxError> {
    let targets = context.players("targets")?;
    let mut kicked = 0;
    for target in &targets {
        // A player may not kick themselves, matching vanilla's self-check.
        if context
            .source()
            .player()
            .is_some_and(|player| player.base().uuid() == target.base().uuid())
        {
            continue;
        }
        target.disconnect(reason.clone());
        kicked += 1;
    }
    if kicked == 0 {
        return Err(CommandSyntaxError::dynamic(
            "No player matched (you cannot kick yourself)",
        ));
    }
    Ok(kicked)
}
