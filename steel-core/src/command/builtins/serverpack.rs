//! Vanilla MC 26.3 `/serverpack` + `/fetchprofile` operator commands.

use steel_protocol::packets::game::{
    CResourcePackPop, CResourcePackPush, MAX_RESOURCE_PACK_HASH_LENGTH,
};
use steel_utils::Identifier;
use text_components::TextComponent;
use uuid::Uuid;

use super::super::{
    brigadier::{ArgumentType, CommandNodeBuilder, CommandSyntaxError},
    execution::{CommandSource, SteelCommandContext, SteelCommandRuntime, argument, literal},
    registration::CommandRegistration,
};

pub(super) fn serverpack_registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("serverpack"), |_| {
        serverpack_command()
    })
}

pub(super) fn fetchprofile_registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("fetchprofile"), |_| {
        fetchprofile_command()
    })
}

fn serverpack_command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("serverpack")
        .then(
            literal("push").then(
                argument("url", ArgumentType::string())
                    .executes(|ctx| push_pack(ctx, false, false))
                    .then(
                        literal("uuid").then(
                            argument("uuid", ArgumentType::string())
                                .executes(|ctx| push_pack(ctx, true, false))
                                .then(
                                    literal("hash").then(
                                        argument("hash", ArgumentType::word())
                                            .executes(|ctx| push_pack(ctx, true, true)),
                                    ),
                                ),
                        ),
                    )
                    .then(
                        literal("hash").then(
                            argument("hash", ArgumentType::word())
                                .executes(|ctx| push_pack(ctx, false, true)),
                        ),
                    ),
            ),
        )
        .then(literal("pop").then(argument("uuid", ArgumentType::string()).executes(pop_pack)))
}

fn push_pack(
    context: &SteelCommandContext<CommandSource>,
    has_uuid: bool,
    has_hash: bool,
) -> Result<i32, CommandSyntaxError> {
    let url = context.string("url")?.to_owned();
    let id = if has_uuid {
        let raw = context.string("uuid")?;
        Uuid::parse_str(raw)
            .map_err(|_| CommandSyntaxError::dynamic(format!("Invalid UUID '{raw}'")))?
    } else {
        // Vanilla `UUID.nameUUIDFromBytes(url.getBytes(UTF_8))` (version 3 / deterministic).
        Uuid::new_v5(&Uuid::NAMESPACE_URL, url.as_bytes())
    };
    let hash = if has_hash {
        let h = context.string("hash")?;
        if h.len() > MAX_RESOURCE_PACK_HASH_LENGTH || !h.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(CommandSyntaxError::dynamic(format!(
                "Invalid SHA-1 resource pack hash '{h}'"
            )));
        }
        h.to_ascii_lowercase()
    } else {
        String::new()
    };

    let packet = CResourcePackPush {
        id,
        url,
        hash,
        required: false,
        prompt: None,
    };
    context.source().server().broadcast_to_online(packet);
    Ok(1)
}

fn pop_pack(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let raw = context.string("uuid")?;
    let id = Uuid::parse_str(raw)
        .map_err(|_| CommandSyntaxError::dynamic(format!("Invalid UUID '{raw}'")))?;
    context
        .source()
        .server()
        .broadcast_to_online(CResourcePackPop { id: Some(id) });
    Ok(1)
}

fn fetchprofile_command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("fetchprofile")
        .then(
            literal("name")
                .then(argument("name", ArgumentType::word()).executes(fetch_profile_by_name)),
        )
        .then(
            literal("id")
                .then(argument("id", ArgumentType::string()).executes(fetch_profile_by_id)),
        )
}

fn fetch_profile_by_name(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let name = context.string("name")?;
    let server = context.source().server();
    let known = server.known_players();
    let Some(entry) = known
        .entries()
        .iter()
        .find(|entry| entry.last_known_name().eq_ignore_ascii_case(name))
    else {
        return Err(CommandSyntaxError::dynamic(format!(
            "Failed to find a cached profile with name {name}"
        )));
    };
    context.source().send_success(
        &TextComponent::plain(format!(
            "Successfully resolved profile {} ({})",
            entry.last_known_name(),
            entry.uuid()
        )),
        false,
    );
    Ok(1)
}

fn fetch_profile_by_id(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let raw = context.string("id")?;
    let id = Uuid::parse_str(raw)
        .map_err(|_| CommandSyntaxError::dynamic(format!("Invalid UUID '{raw}'")))?;
    let server = context.source().server();
    let known = server.known_players();
    let Some(entry) = known.entries().iter().find(|entry| entry.uuid() == id) else {
        return Err(CommandSyntaxError::dynamic(format!(
            "Failed to find a cached profile with id {id}"
        )));
    };
    context.source().send_success(
        &TextComponent::plain(format!(
            "Successfully resolved profile {} ({})",
            entry.last_known_name(),
            entry.uuid()
        )),
        false,
    );
    Ok(1)
}
