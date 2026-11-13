//! `/whitelist` — the login whitelist.
//!
//! Vanilla `WhitelistCommand` has `add`, `remove`, `list`, `on` and `off`. The
//! names in `add` and `remove` are resolved through the game profile cache, so a
//! player who has never joined can still be added by name; Steel resolves them
//! through `GameProfileArgument`, which accepts a real name or a UUID.

use std::borrow::Cow;

use steel_utils::Identifier;
use text_components::TextComponent;
use text_components::translation::TranslatedMessage;

use tokio::{sync::oneshot, task::JoinHandle};

use super::super::{
    brigadier::{CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandResultSuspension, CommandResultSuspensionPoll, CommandSource,
        CommandSuspensionOrder, GameProfileArgument, SteelArgumentType, SteelCommandContext,
        SteelCommandRuntime, argument, literal,
    },
    registration::CommandRegistration,
};
use crate::server::whitelist::Whitelist;

const TARGETS_ARGUMENT: &str = "targets";

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("whitelist"), |_| command())
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    let targets = argument(TARGETS_ARGUMENT, SteelArgumentType::operator_profile());

    literal("whitelist")
        .then(literal("add").then(
            targets
                .clone()
                .executes_suspended(|context| start_operation(context, WhitelistAction::Add)),
        ))
        .then(literal("remove").then(
            targets.executes_suspended(|context| start_operation(context, WhitelistAction::Remove)),
        ))
        .then(literal("list").executes(list))
        .then(literal("on").executes(|context| set_enabled(context, true)))
        .then(literal("off").executes(|context| set_enabled(context, false)))
}

/// Builds a translated component with a literal fallback.
fn translated(key: &'static str, fallback: String, args: Vec<TextComponent>) -> TextComponent {
    TextComponent::translated(TranslatedMessage {
        key: Cow::Borrowed(key),
        fallback: Some(fallback),
        args: Some(args.into_boxed_slice()),
    })
}

fn whitelist_of(context: &SteelCommandContext<CommandSource>) -> std::sync::Arc<Whitelist> {
    std::sync::Arc::clone(&context.source().server().whitelist)
}

/// Which way to edit the list.
#[derive(Clone, Copy)]
enum WhitelistAction {
    Add,
    Remove,
}

impl WhitelistAction {
    const fn success_key(self) -> &'static str {
        match self {
            Self::Add => "commands.whitelist.add.success",
            Self::Remove => "commands.whitelist.remove.success",
        }
    }

    const fn fallback(self, count: usize) -> &'static str {
        match self {
            Self::Add => "Added players to the whitelist",
            Self::Remove => "Removed players from the whitelist",
        }
    }
}

/// Starts the name lookup. Profile resolution may hit the Mojang API, so it runs
/// off the command thread and the command suspends until it finishes, exactly
/// like `/op` does.
fn start_operation(
    context: &SteelCommandContext<CommandSource>,
    action: WhitelistAction,
) -> Result<WhitelistCommandSuspension, CommandSyntaxError> {
    let argument = context.game_profile_argument(TARGETS_ARGUMENT).cloned()?;
    let task_source = context.source().clone();
    let (sender, receiver) = oneshot::channel();
    let task = tokio::spawn(async move {
        let result = run_operation(&task_source, argument, action).await;
        let _ = sender.send(result);
    });
    Ok(WhitelistCommandSuspension {
        receiver,
        task: Some(task),
    })
}

async fn run_operation(
    source: &CommandSource,
    argument: GameProfileArgument,
    action: WhitelistAction,
) -> Result<i32, CommandSyntaxError> {
    let targets = argument.resolve(source).await?;
    let whitelist = source.server().whitelist.clone();

    let mut changed: i32 = 0;
    for target in &targets {
        let touched = match action {
            WhitelistAction::Add => !whitelist.add(target.uuid, target.name.clone()),
            WhitelistAction::Remove => whitelist.remove(target.uuid),
        };
        if touched {
            changed += 1;
        }
    }

    if changed == 0 {
        return Err(CommandSyntaxError::dynamic(translated(
            "commands.whitelist.none",
            "Nothing changed".to_owned(),
            Vec::new(),
        )));
    }

    let message = translated(
        action.success_key(),
        action.fallback(changed).to_owned(),
        vec![TextComponent::from(changed.to_string())],
    );
    source.send_success(&message, true);
    Ok(changed)
}

struct WhitelistCommandSuspension {
    receiver: oneshot::Receiver<Result<i32, CommandSyntaxError>>,
    task: Option<JoinHandle<()>>,
}

impl CommandResultSuspension for WhitelistCommandSuspension {
    fn order(&self) -> CommandSuspensionOrder {
        CommandSuspensionOrder::Global
    }

    fn poll(&mut self) -> CommandResultSuspensionPoll {
        match self.receiver.try_recv() {
            Ok(result) => {
                self.task = None;
                CommandResultSuspensionPoll::Ready(result)
            }
            Err(oneshot::error::TryRecvError::Empty) => CommandResultSuspensionPoll::Pending,
            Err(oneshot::error::TryRecvError::Closed) => {
                self.task = None;
                CommandResultSuspensionPoll::Ready(Err(CommandSyntaxError::dynamic(
                    translated(
                        "commands.whitelist.none",
                        "Profile lookup was cancelled".to_owned(),
                        Vec::new(),
                    ),
                )))
            }
        }
    }
}

fn list(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let whitelist = whitelist_of(context);
    let entries = whitelist.entries();

    let header = translated(
        "commands.whitelist.list",
        format!(
            "There are {} whitelisted player(s):",
            entries.len()
        ),
        vec![TextComponent::from(entries.len().to_string())],
    );
    context.source().send_success(&header, false);

    for entry in &entries {
        let line = translated(
            "commands.whitelist.list.entry",
            format!("- {} ({})", entry.name, entry.uuid),
            vec![
                TextComponent::from(entry.name.clone()),
                TextComponent::from(entry.uuid.to_string()),
            ],
        );
        context.source().send_success(&line, false);
    }

    Ok(entries.len() as i32)
}

fn set_enabled(
    context: &SteelCommandContext<CommandSource>,
    enabled: bool,
) -> Result<i32, CommandSyntaxError> {
    let whitelist = whitelist_of(context);
    let was_enabled = whitelist.set_enabled(enabled);

    let key = if enabled {
        "commands.whitelist.enabled"
    } else {
        "commands.whitelist.disabled"
    };
    let fallback = if enabled {
        "Whitelist is now enabled".to_owned()
    } else {
        "Whitelist is now disabled".to_owned()
    };
    let message = translated(key, fallback, Vec::new());
    if was_enabled == enabled {
        // Vanilla reports the no-op case separately so a scripted `/whitelist
        // on` twice can tell whether it changed anything.
        let already = translated(
            "commands.whitelist.none",
            "The whitelist was already in that state".to_owned(),
            Vec::new(),
        );
        context.source().send_success(&already, true);
        return Ok(0);
    }

    context.source().send_success(&message, true);
    Ok(1)
}
