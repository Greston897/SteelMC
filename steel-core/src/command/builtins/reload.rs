//! Vanilla `/reload`.
//!
//! Vanilla's `/reload` rebuilds its reloadable resource layer and updates
//! connected players. Steel keeps its registries immutable after startup, so
//! this command only reloads the parts that can currently be re-read:
//!
//! * directory-pack functions and function tags;
//! * `groups.toml` — Steel permission groups and their metadata, re-read and
//!   validated before being swapped in.
//! * each world's `level.toml` game rules, so an operator's edit is picked up
//!   without restarting while seeds, clocks, and generation stay untouched.
//!
//! Recipes, loot, advancements and registry tags are not reloaded. The command
//! reports this limitation to the operator instead of claiming a full reload.
//!
//! The reload is `async`, so it runs on a Tokio task and the command resumes
//! through [`ReloadSuspension`] instead of blocking the tick-owned queue.

use std::sync::Arc;

use steel_utils::{Identifier, translations};
use text_components::TextComponent;
use tokio::{sync::oneshot, task::JoinHandle};

use crate::permission::{PermissionExpr, PermissionKey};

use super::super::{
    brigadier::{CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandPermissionSource, CommandResultSuspension, CommandResultSuspensionPoll,
        CommandSource, CommandSuspensionOrder, SteelCommandContext, SteelCommandRuntime, literal,
    },
    registration::CommandRegistration,
};

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("reload"), |_| command())
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("reload").executes_suspended(start_reload)
}

/// Suspension returned by `/reload`.
///
/// The reload itself is `async` (it re-reads files), which cannot run inside the
/// tick-owned command queue, so it is handed to a Tokio task and the command
/// resumes when the task reports back. `Global` ordering keeps every other
/// command behind it while the data-driven state is being swapped.
struct ReloadSuspension {
    /// `None` for a suspension that already finished, e.g. after a refused
    /// permission check.
    receiver: Option<oneshot::Receiver<Result<String, String>>>,
    task: Option<JoinHandle<()>>,
}

impl ReloadSuspension {
    /// A suspension that completes on its first poll.
    const fn finished() -> Self {
        Self {
            receiver: None,
            task: None,
        }
    }
}

impl CommandResultSuspension for ReloadSuspension {
    fn order(&self) -> CommandSuspensionOrder {
        CommandSuspensionOrder::Global
    }

    fn poll(&mut self) -> CommandResultSuspensionPoll {
        let Some(receiver) = self.receiver.as_mut() else {
            return CommandResultSuspensionPoll::Ready(Ok(SELF_RESULT));
        };
        match receiver.try_recv() {
            // The task already reported success/failure to the source; vanilla
            // reports result `0` either way and never fails the command.
            Ok(_) => {
                self.task = None;
                CommandResultSuspensionPoll::Ready(Ok(SELF_RESULT))
            }
            Err(oneshot::error::TryRecvError::Empty) => CommandResultSuspensionPoll::Pending,
            Err(oneshot::error::TryRecvError::Closed) => {
                self.task = None;
                log::warn!("resource reload was cancelled");
                CommandResultSuspensionPoll::Ready(Ok(SELF_RESULT))
            }
        }
    }

    /// Aborts the reload task when the owning execution is cancelled, so a
    /// cancelled command cannot keep mutating permission state behind it.
    fn cancel(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}

/// Vanilla reports result `0` for `/reload` whether or not the reload succeeded.
const SELF_RESULT: i32 = 0;

/// Starts the data-driven reload and reports success/failure to the source.
///
/// The result remains `0`, but the operator is told that Steel's reload is
/// incomplete and may have applied some changes even when a later step fails.
#[expect(
    clippy::unnecessary_wraps,
    reason = "command executors return the command result type"
)]
fn start_reload(
    context: &SteelCommandContext<CommandSource>,
) -> Result<ReloadSuspension, CommandSyntaxError> {
    let source = context.source();
    let server = Arc::clone(source.server());

    // Vanilla gates `/reload` at permission level 2 (`minecraft.command.reload`).
    let allowed = PermissionKey::parse("minecraft.command.reload").is_ok_and(|key| {
        CommandPermissionSource::has_permission(source, &PermissionExpr::key(key))
    });
    if !allowed {
        source.send_failure(translations::PERMISSIONS_REQUIRES_PLAYER.msg().component());
        return Ok(ReloadSuspension::finished());
    }

    let reply_source = source.clone();
    let (sender, receiver) = oneshot::channel();
    let task = tokio::spawn(async move {
        let result = server.reload_data_driven_state().await;
        match &result {
            Ok(summary) => {
                log::info!("{summary}");
                reply_source.send_success(&TextComponent::plain(summary.clone()), true);
            }
            Err(error) => {
                log::warn!("resource reload failed: {error}");
                reply_source.send_failure(TextComponent::plain(format!(
                    "Reload failed: {error}. Some earlier reload steps may already have applied."
                )));
            }
        }
        let _ = sender.send(result);
    });

    Ok(ReloadSuspension {
        receiver: Some(receiver),
        task: Some(task),
    })
}
