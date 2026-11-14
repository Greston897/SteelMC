//! `/banip` and `/pardonip` over the server's address ban list.
//!
//! Vanilla `BanIpCommands` takes a string that must parse as an address or a
//! CIDR block. Steel parses a plain address; CIDR ranges need an address-range
//! representation the ban list does not have, so a range is rejected with the
//! vanilla error rather than silently banning the wrong thing.

use std::borrow::Cow;
use std::net::IpAddr;

use steel_utils::Identifier;
use text_components::TextComponent;
use text_components::translation::TranslatedMessage;

use super::super::{
    brigadier::{ArgumentType, CommandNodeBuilder, CommandSyntaxError},
    execution::{CommandSource, SteelCommandContext, SteelCommandRuntime, argument, literal},
    registration::CommandRegistration,
};
use crate::server::bans::BanEntry;

const ADDRESS_ARGUMENT: &str = "address";
const REASON_ARGUMENT: &str = "reason";

pub(super) fn banip_registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("banip"), |_| ban_command())
}

pub(super) fn pardonip_registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("pardonip"), |_| pardon_command())
}

fn ban_command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("banip")
        .then(
            argument(ADDRESS_ARGUMENT, ArgumentType::greedy_string())
                .executes(|context| ban(context, None)),
        )
        .then(
            argument(ADDRESS_ARGUMENT, ArgumentType::greedy_string()).then(
                argument(REASON_ARGUMENT, ArgumentType::greedy_string())
                    .executes(|context| ban(context, Some(context.string(REASON_ARGUMENT)?.to_owned()))),
            ),
        )
}

fn pardon_command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("pardonip").then(
        argument(ADDRESS_ARGUMENT, ArgumentType::greedy_string()).executes(pardon),
    )
}

/// Builds a translated component with a literal fallback.
fn translated(key: &'static str, fallback: String, args: Vec<TextComponent>) -> TextComponent {
    TextComponent::translated(TranslatedMessage {
        key: Cow::Borrowed(key),
        fallback: Some(Cow::Owned(fallback)),
        args: Some(args.into_boxed_slice()),
    })
}

fn parse_address(context: &SteelCommandContext<CommandSource>) -> Result<IpAddr, CommandSyntaxError> {
    let text = context.string(ADDRESS_ARGUMENT)?;
    text.parse().map_err(|_| {
        CommandSyntaxError::dynamic(translated(
            "commands.banip.invalid",
            format!("Invalid IP address: {text}"),
            vec![TextComponent::from(text.to_owned())],
        ))
    })
}

fn ban(
    context: &SteelCommandContext<CommandSource>,
    reason: Option<String>,
) -> Result<i32, CommandSyntaxError> {
    let address = parse_address(context)?;
    let reason = reason.unwrap_or_else(|| "Banned by an operator.".to_owned());
    let by = context.source().source_name();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX));

    let server = context.source().server().clone();
    server.bans.ban_address(BanEntry::address(
        address,
        address.to_string(),
        reason.clone(),
        by,
        now,
        None,
    ));
    // Vanilla writes `banned-ips.json` inside the command; the handler is
    // synchronous, so the write is handed to the runtime instead of the tick.
    save_bans_in_background(&server);

    let message = translated(
        "commands.banip.success",
        format!("Banned IP address {address}: {reason}"),
        vec![
            TextComponent::from(address.to_string()),
            TextComponent::from(reason),
        ],
    );
    context.source().send_success(&message, true);
    Ok(1)
}

fn pardon(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let address = parse_address(context)?;
    let server = context.source().server().clone();
    if !server.bans.pardon_address(address) {
        return Err(CommandSyntaxError::dynamic(translated(
            "commands.unbanip.failed",
            format!("{address} is not banned"),
            vec![TextComponent::from(address.to_string())],
        )));
    }
    save_bans_in_background(&server);

    let message = translated(
        "commands.unbanip.success",
        format!("Pardoned IP address {address}"),
        vec![TextComponent::from(address.to_string())],
    );
    context.source().send_success(&message, true);
    Ok(1)
}

/// Writes the ban lists without blocking the synchronous command handler.
///
/// `save_bans` is async because it touches the filesystem; the command itself
/// runs on the tick thread, so it schedules the write and moves on.
fn save_bans_in_background(server: &std::sync::Arc<crate::server::Server>) {
    let server = std::sync::Arc::clone(server);
    tokio::spawn(async move {
        if let Err(error) = server.save_bans().await {
            log::error!("failed to save the ban list: {error}");
        }
    });
}
