//! Vanilla `/save-all`, `/save-off` and `/save-on`.
//!
//! Vanilla stops the automatic `MinecraftServer.saveEverything` loop with
//! `/save-off` and resumes it with `/save-on`, while `/save-all` forces one save
//! pass. Steel owns the same switch: the server keeps an autosave flag, the run
//! loop consults it before scheduling an autosave, and `/save-all` writes every
//! command-owned data set immediately.

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

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("save-all")
        .executes(|context| save_all(context))
        .then(literal("flush").executes(|context| save_all(context)))
        .then(literal("save-off").executes(save_off))
        .then(literal("save-on").executes(save_on))
}

/// Runs one save pass, matching vanilla `SaveAllCommand`.
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
    let task_server = std::sync::Arc::clone(server);
    tokio::spawn(async move {
        let results = task_server.save_command_data().await;
        let failure = results
            .scoreboards
            .as_ref()
            .err()
            .or(results.storage.as_ref().err())
            .map(ToString::to_string);
        match failure {
            Some(error) => {
                source.send_failure(TextComponent::plain(format!("Save failed: {error}")));
            }
            None => {
                source.send_success(&TextComponent::plain("Saved the game"), true);
            }
        }
    });
    Ok(1)
}

fn save_off(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    context.source().server().set_auto_save_enabled(false);
    context
        .source()
        .send_success(&TextComponent::plain("Automatic saving is now disabled"), true);
    Ok(1)
}

fn save_on(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    context.source().server().set_auto_save_enabled(true);
    context
        .source()
        .send_success(&TextComponent::plain("Automatic saving is now enabled"), true);
    Ok(1)
}
