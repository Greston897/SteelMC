//! `/ban`, `/pardon` and `/banlist` over the server's ban list.
//!
//! Vanilla resolves the target through the profile cache, kicks them if they are
//! online and records the reason and issuer. Steel does the same, except that
//! bans are not persisted yet (see [`crate::server::bans`]), so they last until
//! the server stops.
//!
//! `/banip` and `/pardonip` live in `super::banip`.

use std::borrow::Cow;

use steel_utils::Identifier;
use text_components::TextComponent;
use text_components::translation::TranslatedMessage;
use tokio::{sync::oneshot, task::JoinHandle};

use super::super::{
    brigadier::{ArgumentType, CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandResultSuspension, CommandResultSuspensionPoll, CommandSource,
        CommandSuspensionOrder, GameProfileArgument, SteelArgumentType, SteelCommandContext,
        SteelCommandRuntime, argument, literal,
    },
    registration::CommandRegistration,
};
use crate::server::bans::BanEntry;

const TARGETS_ARGUMENT: &str = "targets";
const REASON_ARGUMENT: &str = "reason";

pub(super) fn ban_registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("ban"), |_| ban_command())
}

pub(super) fn pardon_registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("pardon"), |_| pardon_command())
}

pub(super) fn banlist_registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("banlist"), |_| banlist_command())
}

fn ban_command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    let targets = argument(TARGETS_ARGUMENT, SteelArgumentType::game_profile());
    literal("ban")
        .then(
            targets
                .clone()
                .executes_suspended(|context| start_ban(context, None)),
        )
        .then(
            targets.then(
                argument(REASON_ARGUMENT, ArgumentType::greedy_string())
                    .executes_suspended(|context| start_ban(context, Some(context.string(REASON_ARGUMENT)?.to_owned()))),
            ),
        )
}

fn pardon_command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("pardon").then(
        argument(TARGETS_ARGUMENT, SteelArgumentType::operator_profile())
            .executes_suspended(start_pardon),
    )
}

fn banlist_command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("banlist")
        .executes(|context| list(context, Listing::Players))
        .then(literal("ips").executes(|context| list(context, Listing::Addresses)))
}

/// Builds a translated component with a literal fallback.
fn translated(key: &'static str, fallback: String, args: Vec<TextComponent>) -> TextComponent {
    TextComponent::translated(TranslatedMessage {
        key: Cow::Borrowed(key),
        fallback: Some(fallback),
        args: Some(args.into_boxed_slice()),
    })
}

/// The current wall-clock second, which is what ban expiry compares against.
fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX))
}

fn start_ban(
    context: &SteelCommandContext<CommandSource>,
    reason: Option<String>,
) -> Result<BanSuspension, CommandSyntaxError> {
    let argument = context.game_profile_argument(TARGETS_ARGUMENT).cloned()?;
    let source = context.source().clone();
    let source_name = context.source().source_name();
    let reason = reason.unwrap_or_else(|| "Banned by an operator.".to_owned());

    let task_source = source.clone();
    let (sender, receiver) = oneshot::channel();
    let task = tokio::spawn(async move {
        let result = run_ban(&task_source, argument, reason, source_name).await;
        let _ = sender.send(result);
    });
    Ok(BanSuspension {
        receiver,
        task: Some(task),
    })
}

async fn run_ban(
    source: &CommandSource,
    argument: GameProfileArgument,
    reason: String,
    by: String,
) -> Result<i32, CommandSyntaxError> {
    let targets = argument.resolve(source).await?;
    let server = source.server();
    let now = now();
    let mut banned = 0;

    for target in &targets {
        let entry = BanEntry::player(
            target.uuid,
            target.name.clone(),
            reason.clone(),
            by.clone(),
            now,
            None,
        );
        server.bans.ban_player(entry);

        // Vanilla kicks the player if they are online, with the ban reason.
        if let Some(player) = server.player_by_uuid(target.uuid) {
            player.disconnect(format!("You are banned: {reason}"));
        }
        banned += 1;
    }

    let message = translated(
        "commands.ban.success",
        format!("Banned {banned} player(s): {reason}"),
        vec![
            TextComponent::from(banned.to_string()),
            TextComponent::from(reason),
        ],
    );
    source.send_success(&message, true);
    Ok(banned)
}

fn start_pardon(
    context: &SteelCommandContext<CommandSource>,
) -> Result<BanSuspension, CommandSyntaxError> {
    let argument = context.game_profile_argument(TARGETS_ARGUMENT).cloned()?;
    let source = context.source().clone();
    let (sender, receiver) = oneshot::channel();
    let task_source = source.clone();
    let task = tokio::spawn(async move {
        let result = run_pardon(&task_source, argument).await;
        let _ = sender.send(result);
    });
    Ok(BanSuspension {
        receiver,
        task: Some(task),
    })
}

async fn run_pardon(
    source: &CommandSource,
    argument: GameProfileArgument,
) -> Result<i32, CommandSyntaxError> {
    let targets = argument.resolve(source).await?;
    let bans = &source.server().bans;
    let mut pardoned = 0;
    for target in &targets {
        if bans.pardon(target.uuid) {
            pardoned += 1;
        }
    }

    if pardoned == 0 {
        return Err(CommandSyntaxError::dynamic(translated(
            "commands.unban.failed",
            "That player is not banned".to_owned(),
            Vec::new(),
        )));
    }

    let message = translated(
        "commands.unban.success",
        format!("Pardoned {pardoned} player(s)"),
        vec![TextComponent::from(pardoned.to_string())],
    );
    source.send_success(&message, true);
    Ok(pardoned)
}

#[derive(Clone, Copy)]
enum Listing {
    Players,
    Addresses,
}

fn list(
    context: &SteelCommandContext<CommandSource>,
    listing: Listing,
) -> Result<i32, CommandSyntaxError> {
    let bans = &context.source().server().bans;
    let now = now();
    let (entries, key, fallback) = match listing {
        Listing::Players => (
            bans.player_bans(),
            "commands.banlist.players",
            "There are banned players:",
        ),
        Listing::Addresses => (
            bans.address_bans(),
            "commands.banlist.ips",
            "There are banned IP addresses:",
        ),
    };

    let entries: Vec<_> = entries
        .into_iter()
        .filter(|entry| !entry.is_expired(now))
        .collect();

    let header = translated(
        key,
        format!("{fallback} {}", entries.len()),
        vec![TextComponent::from(entries.len().to_string())],
    );
    context.source().send_success(&header, false);

    for entry in &entries {
        let line = translated(
            "commands.banlist.entry",
            format!("- {}: {}", entry.name, entry.display_reason()),
            vec![
                TextComponent::from(entry.name.clone()),
                TextComponent::from(entry.display_reason().to_owned()),
            ],
        );
        context.source().send_success(&line, false);
    }

    Ok(entries.len() as i32)
}

/// Suspension shared by the ban and pardon commands: both resolve a profile
/// through the cache, which may need the network.
struct BanSuspension {
    receiver: oneshot::Receiver<Result<i32, CommandSyntaxError>>,
    task: Option<JoinHandle<()>>,
}

impl CommandResultSuspension for BanSuspension {
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
                        "commands.ban.failed",
                        "Profile lookup was cancelled".to_owned(),
                        Vec::new(),
                    ),
                )))
            }
        }
    }
}
