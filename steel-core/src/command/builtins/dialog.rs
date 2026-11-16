//! Vanilla MC 26.3 `/dialog` command (`DialogCommand`).
//!
//! Supports `/dialog show <targets> <dialog>` (inline SNBT compound dialog
//! definition) and `/dialog clear <targets>`.

use steel_protocol::packets::game::{CClearDialog, CShowDialog};
use steel_utils::{Identifier, translations};
use text_components::TextComponent;

use super::super::{
    brigadier::{CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandSource, SteelArgumentType, SteelCommandContext, SteelCommandRuntime, argument,
        literal,
    },
    registration::CommandRegistration,
};
use crate::entity::Entity as _;

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("dialog"), |_| command())
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("dialog")
        .then(
            literal("show").then(
                argument("targets", SteelArgumentType::players()).then(
                    argument("dialog", SteelArgumentType::compound()).executes(show_dialog),
                ),
            ),
        )
        .then(
            literal("clear")
                .then(argument("targets", SteelArgumentType::players()).executes(clear_dialog)),
        )
}

fn show_dialog(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let targets = context.players("targets")?;
    let dialog = context.compound("dialog")?.clone();
    for player in &targets {
        player.send_packet(CShowDialog::direct(dialog.clone()));
    }
    let source = context.source();
    if targets.len() == 1 {
        source.send_success(
            &TextComponent::translated(
                translations::COMMANDS_DIALOG_SHOW_SINGLE.message([targets[0].display_name()]),
            ),
            true,
        );
    } else {
        source.send_success(
            &TextComponent::translated(
                translations::COMMANDS_DIALOG_SHOW_MULTIPLE
                    .message([TextComponent::plain(targets.len().to_string())]),
            ),
            true,
        );
    }
    Ok(targets.len() as i32)
}

fn clear_dialog(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let targets = context.players("targets")?;
    for player in &targets {
        player.send_packet(CClearDialog {});
    }
    let source = context.source();
    if targets.len() == 1 {
        source.send_success(
            &TextComponent::translated(
                translations::COMMANDS_DIALOG_CLEAR_SINGLE.message([targets[0].display_name()]),
            ),
            true,
        );
    } else {
        source.send_success(
            &TextComponent::translated(
                translations::COMMANDS_DIALOG_CLEAR_MULTIPLE
                    .message([TextComponent::plain(targets.len().to_string())]),
            ),
            true,
        );
    }
    Ok(targets.len() as i32)
}
