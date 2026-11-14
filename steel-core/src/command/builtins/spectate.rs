//! `/spectate [target] [player]` — moves a spectator's camera onto an entity.
//!
//! Vanilla `SpectateCommand` is a wrapper around the same `setCamera` path the
//! spectator menu uses. Steel already has that path (the entity-interaction
//! range check, the camera slot and the `set_camera` packet), so this command
//! only resolves the target and calls into it.
//!
//! Omitting the target returns the camera to the player, mirroring vanilla's
//! `spectate` with no arguments.

use std::borrow::Cow;

use steel_utils::Identifier;
use text_components::TextComponent;
use text_components::translation::TranslatedMessage;

use super::super::{
    brigadier::{CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandSource, SteelArgumentType, SteelCommandContext, SteelCommandRuntime, argument,
        literal,
    },
    registration::CommandRegistration,
};
use steel_utils::types::GameType;

use crate::entity::Entity as _;

const TARGET_ARGUMENT: &str = "target";
const PLAYER_ARGUMENT: &str = "player";

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("spectate"), |_| command())
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("spectate")
        .executes(|context| spectate(context, None))
        .then(argument(TARGET_ARGUMENT, SteelArgumentType::entity()).executes(|context| {
            let target = context.entity(TARGET_ARGUMENT)?;
            spectate(context, Some(target))
        }))
        .then(
            argument(TARGET_ARGUMENT, SteelArgumentType::entity()).then(
                argument(PLAYER_ARGUMENT, SteelArgumentType::player()).executes(|context| {
                    let target = context.entity(TARGET_ARGUMENT)?;
                    spectate(context, Some(target))
                }),
            ),
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

fn spectate(
    context: &SteelCommandContext<CommandSource>,
    target: Option<crate::entity::SharedEntity>,
) -> Result<i32, CommandSyntaxError> {
    // The camera belongs to the player the command runs as.
    let Some(viewer) = context.source().player().cloned() else {
        return Err(CommandSyntaxError::dynamic(translations_requires_player()));
    };

    if viewer.game_mode() != GameType::Spectator {
        return Err(CommandSyntaxError::dynamic(translated(
            "commands.spectate.self",
            "You must be a spectator to use this command".to_owned(),
            Vec::new(),
        )));
    }

    match target {
        Some(target) => {
            let entity_id = target.id();
            viewer.set_camera_entity(Some(target));
            viewer.send_packet(steel_protocol::packets::game::CSetCamera {
                camera_id: entity_id,
            });
            let message = translated(
                "commands.spectate.success",
                format!("Now spectating entity {entity_id}"),
                vec![TextComponent::from(entity_id.to_string())],
            );
            context.source().send_success(&message, true);
            Ok(1)
        }
        None => {
            // Vanilla clears the camera when the target is omitted.
            viewer.set_camera_entity(None);
            if let Some(player) = context.source().player() {
                viewer.send_packet(steel_protocol::packets::game::CSetCamera {
                    camera_id: player.base().id(),
                });
            }
            context.source().send_success(
                &translated(
                    "commands.spectate.self",
                    "Stopped spectating".to_owned(),
                    Vec::new(),
                ),
                true,
            );
            Ok(1)
        }
    }
}

fn translations_requires_player() -> TextComponent {
    translated(
        "permissions.requires.player",
        "This command requires a player".to_owned(),
        Vec::new(),
    )
}
