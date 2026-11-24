//! `/datapack` — vanilla `DataPackCommand` selection over Steel's runtime packs.
//!
//! Implements `enable <name> [first|last|after <existing>|before <existing>]`,
//! `disable <name>`, `list [available|enabled]`, `create <name>`, and `reload`.
//! Selection order, placement rules, refusal messages, and the chat links follow
//! vanilla's `DataPackCommand`/`Pack.Position`/`PackLocationInfo`. `reload`
//! delegates to the server's data-driven state reload (functions, function tags,
//! game rules, permission groups); recipes, loot tables, advancements and
//! registry tags are not yet reloaded at runtime.

use std::{borrow::Cow, sync::Arc};

use steel_utils::Identifier;
use text_components::{
    Modifier, TextComponent, format::Color, interactivity::HoverEvent,
    translation::TranslatedMessage,
};
use tokio::{sync::oneshot, task::JoinHandle};

use super::super::{
    brigadier::{ArgumentType, CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandPermissionSource, CommandResultSuspension, CommandResultSuspensionPoll,
        CommandSource, CommandSuspensionOrder, SteelCommandContext, SteelCommandRuntime, argument,
        literal,
    },
    registration::CommandRegistration,
};
use crate::permission::{PermissionExpr, PermissionKey};
use crate::server::datapacks::{PackCreationError, PackInfo, PackPlacement, PackSelectionError};

/// `/datapack` command registration: vanilla `DataPackCommand` selection over
/// Steel's runtime function packs. Implements `enable`, `disable`, and `list`.
pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("datapack"), |_| command())
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("datapack")
        .then(literal("enable").then(enable_target()))
        .then(literal("disable").then(argument("name", ArgumentType::string()).executes(disable)))
        .then(literal("create").then(argument("name", ArgumentType::string()).executes(create)))
        .then(literal("reload").executes_suspended(datapack_reload))
        .then(
            literal("list")
                .executes(|context| list(context, Listing::All))
                .then(literal("enabled").executes(|context| list(context, Listing::Enabled)))
                .then(literal("available").executes(|context| list(context, Listing::Available))),
        )
}

/// `enable <name> [first|last|after <existing>|before <existing>]`.
fn enable_target() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    argument("name", ArgumentType::string())
        .executes(|context| enable(context, &PackPlacement::Default))
        .then(literal("first").executes(|context| enable(context, &PackPlacement::First)))
        .then(literal("last").executes(|context| enable(context, &PackPlacement::Last)))
        .then(
            literal("after").then(argument("existing", ArgumentType::string()).executes(
                |context| {
                    let existing = context.string("existing")?.to_owned();
                    enable(context, &PackPlacement::After(existing))
                },
            )),
        )
        .then(
            literal("before").then(argument("existing", ArgumentType::string()).executes(
                |context| {
                    let existing = context.string("existing")?.to_owned();
                    enable(context, &PackPlacement::Before(existing))
                },
            )),
        )
}

#[derive(Clone, Copy)]
enum Listing {
    All,
    Enabled,
    Available,
}

/// Builds a translated component with a literal fallback.
fn translated(key: &'static str, fallback: String, args: Vec<TextComponent>) -> TextComponent {
    TextComponent::translated(TranslatedMessage {
        key: Cow::Borrowed(key),
        fallback: Some(Cow::Owned(fallback)),
        args: Some(args.into_boxed_slice()),
    })
}

/// Resolves a discovered pack's location info, falling back to the id itself.
fn pack_info(infos: &[PackInfo], id: &str) -> PackInfo {
    infos
        .iter()
        .find(|info| info.id == id)
        .cloned()
        .unwrap_or_else(|| PackInfo {
            id: id.to_owned(),
            title: id.to_owned(),
            description: String::new(),
            source_label: "world",
            pack_format: None,
            incompatible: false,
        })
}

/// Vanilla `PackLocationInfo.createChatLink`: `[id (source)]` with the id in
/// green when enabled and red when not, the insertion set to the raw id, and a
/// hover tooltip of the pack title and description.
fn chat_link(info: &PackInfo, enabled: bool) -> TextComponent {
    let color = if enabled { Color::Green } else { Color::Red };
    TextComponent::plain("[")
        .color(color.clone())
        .add_child(TextComponent::plain(info.decorated_id()).color(Color::Gray))
        .add_child(TextComponent::plain("]").color(color))
        .insertion(info.id.clone())
        .hover_event(HoverEvent::show_text(TextComponent::plain(format!(
            "{}\n{}",
            info.title, info.description
        ))))
}

/// Joins chat links the way vanilla's `ComponentUtils.formatList` does.
fn joined_links(infos: &[PackInfo], enabled: bool) -> TextComponent {
    let Some((first, rest)) = infos.split_first() else {
        return TextComponent::plain(String::new());
    };
    let mut list = chat_link(first, enabled);
    for info in rest {
        list = list
            .add_child(TextComponent::plain(", "))
            .add_child(chat_link(info, enabled));
    }
    list
}

fn enable(
    context: &SteelCommandContext<CommandSource>,
    placement: &PackPlacement,
) -> Result<i32, CommandSyntaxError> {
    let id = context.string("name")?.to_owned();
    let server = context.source().server();
    let selected = server
        .function_packs
        .enable(&id, placement)
        .map_err(|error| selection_error(&error))?;
    server.persist_pack_selection(&server.function_packs.selection());
    let info = pack_info(&server.function_packs.pack_infos(), &id);
    let link = chat_link(&info, true);
    let message = translated(
        "commands.datapack.modify.enable",
        format!("Enabling data pack [{}]", info.decorated_id()),
        vec![link],
    );
    context.source().send_success(&message, true);
    Ok(selected.len() as i32)
}

fn disable(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let id = context.string("name")?.to_owned();
    let server = context.source().server();
    let selected = server
        .function_packs
        .disable(&id)
        .map_err(|error| selection_error(&error))?;
    server.persist_pack_selection(&server.function_packs.selection());
    let info = pack_info(&server.function_packs.pack_infos(), &id);
    let link = chat_link(&info, false);
    let message = translated(
        "commands.datapack.modify.disable",
        format!("Disabling data pack [{}]", info.decorated_id()),
        vec![link],
    );
    context.source().send_success(&message, true);
    Ok(selected.len() as i32)
}

fn create(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let name = context.string("name")?.to_owned();
    let server = context.source().server();
    let available = server
        .function_packs
        .create(&name)
        .map_err(|error| creation_error(&error))?;
    server.persist_pack_selection(&server.function_packs.selection());
    let id = format!("file/{name}");
    let info = pack_info(&server.function_packs.pack_infos(), &id);
    let link = chat_link(&info, false);
    let message = translated(
        "commands.datapack.create.success",
        format!("Created data pack [{}]", info.decorated_id()),
        vec![link],
    );
    context.source().send_success(&message, true);
    Ok(available.len() as i32)
}

/// `/datapack reload` — delegates to the server's data-driven state reload.
///
/// Mirrors vanilla `/reload` through the datapack command namespace: reloads
/// function packs, function tags, game rules and permission groups. Recipes,
/// loot tables, advancements and registry tags are not reloaded at runtime.
struct DatapackReloadSuspension {
    receiver: Option<oneshot::Receiver<Result<String, String>>>,
    task: Option<JoinHandle<()>>,
}

impl DatapackReloadSuspension {
    const fn finished() -> Self {
        Self {
            receiver: None,
            task: None,
        }
    }
}

impl CommandResultSuspension for DatapackReloadSuspension {
    fn order(&self) -> CommandSuspensionOrder {
        CommandSuspensionOrder::Global
    }

    fn poll(&mut self) -> CommandResultSuspensionPoll {
        let Some(receiver) = self.receiver.as_mut() else {
            return CommandResultSuspensionPoll::Ready(Ok(SELF_RESULT));
        };
        match receiver.try_recv() {
            Ok(_) => {
                self.task = None;
                CommandResultSuspensionPoll::Ready(Ok(SELF_RESULT))
            }
            Err(oneshot::error::TryRecvError::Empty) => CommandResultSuspensionPoll::Pending,
            Err(oneshot::error::TryRecvError::Closed) => {
                self.task = None;
                log::warn!("datapack reload was cancelled");
                CommandResultSuspensionPoll::Ready(Ok(SELF_RESULT))
            }
        }
    }

    fn cancel(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}

const SELF_RESULT: i32 = 0;

#[expect(
    clippy::unnecessary_wraps,
    reason = "command executors return the command result type"
)]
fn datapack_reload(
    context: &SteelCommandContext<CommandSource>,
) -> Result<DatapackReloadSuspension, CommandSyntaxError> {
    let source = context.source();
    let server = Arc::clone(source.server());

    let allowed = PermissionKey::parse("minecraft.command.reload").is_ok_and(|key| {
        CommandPermissionSource::has_permission(source, &PermissionExpr::key(key))
    });
    if !allowed {
        source.send_failure(
            steel_utils::translations::PERMISSIONS_REQUIRES_PLAYER
                .msg()
                .component(),
        );
        return Ok(DatapackReloadSuspension::finished());
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
                log::warn!("datapack reload failed: {error}");
                reply_source.send_failure(TextComponent::plain(format!(
                    "Reload failed: {error}. Some earlier reload steps may already have applied."
                )));
            }
        }
        let _ = sender.send(result);
    });

    Ok(DatapackReloadSuspension {
        receiver: Some(receiver),
        task: Some(task),
    })
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "command executors return the command result type"
)]
fn list(
    context: &SteelCommandContext<CommandSource>,
    listing: Listing,
) -> Result<i32, CommandSyntaxError> {
    let server = context.source().server();
    // Vanilla's `listPacks` re-discovers the repository first, so a pack added
    // since startup appears without an explicit `/reload`; it does not reload
    // the function resources.
    let _ = server.function_packs.refresh_repository();
    let infos = server.function_packs.pack_infos();
    let selected = server.function_packs.selected_packs();
    let mut total = 0;
    if matches!(listing, Listing::All | Listing::Enabled) {
        total += list_enabled(context, &infos, &selected);
    }
    if matches!(listing, Listing::All | Listing::Available) {
        total += list_available(context, &infos, &selected);
    }
    Ok(total)
}

fn list_enabled(
    context: &SteelCommandContext<CommandSource>,
    infos: &[PackInfo],
    selected: &[String],
) -> i32 {
    let count = selected.len();
    if selected.is_empty() {
        context.source().send_success(
            &translated(
                "commands.datapack.list.enabled.none",
                "There are no data packs enabled".to_owned(),
                Vec::new(),
            ),
            false,
        );
    } else {
        let links = selected
            .iter()
            .map(|id| pack_info(infos, id))
            .collect::<Vec<_>>();
        let message = translated(
            "commands.datapack.list.enabled.success",
            format!(
                "There are {count} data pack(s) enabled: {}",
                selected.join(", ")
            ),
            vec![
                TextComponent::plain(count.to_string()),
                joined_links(&links, true),
            ],
        );
        context.source().send_success(&message, false);
    }
    count as i32
}

fn list_available(
    context: &SteelCommandContext<CommandSource>,
    infos: &[PackInfo],
    selected: &[String],
) -> i32 {
    let unselected = infos
        .iter()
        .filter(|info| !selected.contains(&info.id))
        .cloned()
        .collect::<Vec<_>>();
    let count = unselected.len();
    if unselected.is_empty() {
        context.source().send_success(
            &translated(
                "commands.datapack.list.available.none",
                "There are no more data packs available".to_owned(),
                Vec::new(),
            ),
            false,
        );
    } else {
        let ids = unselected
            .iter()
            .map(|info| info.id.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let message = translated(
            "commands.datapack.list.available.success",
            format!("There are {count} data pack(s) available: {ids}"),
            vec![
                TextComponent::plain(count.to_string()),
                joined_links(&unselected, false),
            ],
        );
        context.source().send_success(&message, false);
    }
    count as i32
}

/// Maps a refused selection change onto vanilla's `commands.datapack.*` feedback.
fn selection_error(error: &PackSelectionError) -> CommandSyntaxError {
    let (key, fallback, args) = match error {
        PackSelectionError::UnknownPack(id) => (
            "commands.datapack.unknown",
            format!("Unknown data pack '{id}'"),
            vec![TextComponent::plain(id.clone())],
        ),
        PackSelectionError::AlreadyEnabled(id) => (
            "commands.datapack.enable.failed",
            format!("Pack '{id}' is already enabled!"),
            vec![TextComponent::plain(id.clone())],
        ),
        PackSelectionError::AlreadyDisabled(id) => (
            "commands.datapack.disable.failed",
            format!("Pack '{id}' is not enabled!"),
            vec![TextComponent::plain(id.clone())],
        ),
    };
    CommandSyntaxError::dynamic(translated(key, fallback, args))
}

/// Maps a refused pack creation onto vanilla's `commands.datapack.create.*`
/// feedback.
fn creation_error(error: &PackCreationError) -> CommandSyntaxError {
    let (key, fallback, args) = match error {
        PackCreationError::AlreadyExists(id) => (
            "commands.datapack.create.failed.already_exists",
            format!("Data pack '{id}' already exists"),
            vec![TextComponent::plain(id.clone())],
        ),
        PackCreationError::InvalidName(name) => (
            "commands.datapack.create.failed.invalid_name",
            format!("Invalid data pack name '{name}'"),
            vec![TextComponent::plain(name.clone())],
        ),
    };
    CommandSyntaxError::dynamic(translated(key, fallback, args))
}
