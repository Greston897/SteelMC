//! Vanilla `/save-all`, `/save-off` and `/save-on`.
//!
//! Vanilla stops the automatic `MinecraftServer.saveEverything` loop with
//! `/save-off` and resumes it with `/save-on`, while `/save-all` forces one save
//! pass. Steel owns the same switch: the server keeps an autosave flag, the run
//! loop consults it before scheduling an autosave, and `/save-all` writes every
//! command-owned data set immediately.

use std::sync::Arc;
use steel_utils::Identifier;
use text_components::TextComponent;

use super::super::{
    brigadier::{CommandNodeBuilder, CommandSyntaxError},
    execution::{CommandSource, SteelCommandContext, SteelCommandRuntime, literal},
    registration::CommandRegistration,
};

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("save-all"), |_| command())
}

pub(super) fn save_off_registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("save-off"), |_| {
        literal("save-off").executes(save_off)
    })
}

pub(super) fn save_on_registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("save-on"), |_| {
        literal("save-on").executes(save_on)
    })
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("save-all")
        .executes(save_all)
        .then(literal("flush").executes(save_all))
}

/// Runs one save pass, matching vanilla `SaveAllCommand`.
#[expect(
    clippy::unnecessary_wraps,
    reason = "command executors return the command result type"
)]
fn save_all(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let server = context.source().server();
    if !server.is_auto_save_enabled() {
        // Vanilla refuses to save while autosaving is off.
        context
            .source()
            .send_failure(TextComponent::plain("Saving is disabled"));
        return Ok(0);
    }

    let source = context.source().clone();
    let task_server = Arc::clone(server);
    tokio::spawn(async move {
        let results = task_server.save_command_data().await;
        // Vanilla `SaveAllCommand` saves the chunks too; reporting "Saved the
        // game" while a chunk write failed would be a lie, so every set counts.
        let failure = results
            .chunks
            .as_ref()
            .err()
            .or(results.scoreboards.as_ref().err())
            .or(results.storage.as_ref().err())
            .map(ToString::to_string);
        if let Some(error) = failure {
            source.send_failure(TextComponent::plain(format!("Save failed: {error}")));
        } else {
            if let Ok(chunks) = results.chunks {
                tracing::debug!(chunks, "Saved chunks");
            }
            source.send_success(&TextComponent::plain("Saved the game"), true);
        }
    });
    Ok(1)
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "command executors return the command result type"
)]
fn save_off(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    context.source().server().set_auto_save_enabled(false);
    context.source().send_success(
        &TextComponent::plain("Automatic saving is now disabled"),
        true,
    );
    Ok(1)
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "command executors return the command result type"
)]
fn save_on(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    context.source().server().set_auto_save_enabled(true);
    context.source().send_success(
        &TextComponent::plain("Automatic saving is now enabled"),
        true,
    );
    Ok(1)
}
