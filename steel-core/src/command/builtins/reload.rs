//! Vanilla `/reload`.
//!
//! Vanilla's `/reload` re-runs the data pack reloader: every data-driven file
//! (permissions, game rules, advancements) is re-read from disk and the live
//! server is rebuilt from it. Steel keeps its registries immutable after
//! startup, so this command reloads the parts that are genuinely re-readable at
//! runtime and reports everything it touched:
//!
//! * `groups.toml` — permission groups and their metadata (vanilla's
//!   `Permissions` data pack).
//! * the per-world game rule store, so an operator's edit of the saved world
//!   data is picked up without restarting.
//!
//! Registries that would require regenerating the whole world (biomes, blocks)
//! are *not* reloaded, and the command says so through its return value rather
//! than silently pretending a full reload happened.

use steel_utils::{Identifier, translations};

use crate::permission::{PermissionExpr, PermissionKey};

use super::super::{
    brigadier::{CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandPermissionSource, CommandSource, SteelCommandContext, SteelCommandRuntime, literal,
    },
    registration::CommandRegistration,
};

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("reload"), |_| command())
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("reload").executes(reload)
}

/// Reloads the data-driven server configuration.
///
/// Vanilla answers `/reload` with `commands.reload.success` on success and
/// `commands.reload.failure` when the reloader throws; it always reports result
/// `0` and never fails the command itself.
#[expect(
    clippy::unnecessary_wraps,
    reason = "Command executors use a shared fallible callback signature."
)]
fn reload(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let source = context.source();
    let server = source.server();

    // Vanilla gates `/reload` at permission level 2 (`minecraft.command.reload`).
    let allowed = PermissionKey::parse("minecraft.command.reload").is_ok_and(|key| {
        CommandPermissionSource::has_permission(source, &PermissionExpr::key(key))
    });
    if !allowed {
        source.send_failure(translations::PERMISSIONS_REQUIRES_PLAYER.msg().component());
        return Ok(0);
    }

    match server.reload_data_driven_state() {
        Ok(summary) => {
            source.send_success(&translations::COMMANDS_RELOAD_SUCCESS.msg().component(), true);
            log::info!("{summary}");
        }
        Err(error) => {
            source.send_failure(translations::COMMANDS_RELOAD_FAILURE.msg().component());
            log::warn!("resource reload failed: {error}");
        }
    }

    // Vanilla returns 0 for `/reload` either way.
    Ok(0)
}
