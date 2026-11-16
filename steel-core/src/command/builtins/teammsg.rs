//! Vanilla `/teammsg` (alias `/tm`) team chat command (`TeamMsgCommand`).

use std::sync::Arc;

use steel_registry::vanilla_chat_types;
use steel_utils::{Identifier, translations};
use text_components::TextComponent;

use super::super::{
    brigadier::{ArgumentType, CommandNodeBuilder, CommandSyntaxError},
    execution::{CommandSource, SteelCommandContext, SteelCommandRuntime, argument, literal},
    registration::CommandRegistration,
};
use crate::player::Player;
use crate::scoreboard::ScoreHolder;

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("teammsg"), |_| command())
        .alias("tm")
        .default_access()
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("teammsg").then(
        argument("message", ArgumentType::greedy_string()).executes(send_team_message),
    )
}

fn send_team_message(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let source = context.source();
    let Some(sender) = source.player().cloned() else {
        return Err(CommandSyntaxError::dynamic(
            translations::PERMISSIONS_REQUIRES_PLAYER.msg(),
        ));
    };
    let domain = source.world().domain().to_owned();
    let server = source.server();
    let Some(scoreboard) = server.scoreboards.get(&domain) else {
        return Err(CommandSyntaxError::dynamic(
            translations::COMMANDS_TEAMMSG_FAILED_NOTEAM.msg(),
        ));
    };

    let sender_holder = ScoreHolder::new(sender.gameprofile.name().to_owned());
    let Some(team_name) = scoreboard.team_of(&sender_holder) else {
        return Err(CommandSyntaxError::dynamic(
            translations::COMMANDS_TEAMMSG_FAILED_NOTEAM.msg(),
        ));
    };
    let team_display = TextComponent::plain(team_name.clone());

    let message = TextComponent::plain(context.string("message")?.to_owned());
    let sender_name = source.display_name_component();

    let mut recipients: Vec<Arc<Player>> = Vec::new();
    for world in server.worlds.worlds_in_domain(&domain) {
        world.players.iter_players(|uuid, player| {
            let holder = ScoreHolder::new(player.gameprofile.name().to_owned());
            if *uuid != sender.gameprofile.id()
                && scoreboard.team_of(&holder).as_deref() == Some(team_name.as_str())
            {
                recipients.push(Arc::clone(player));
            }
            true
        });
    }

    if !recipients.is_empty() {
        server.broadcast_decorated_chat_to(
            &vanilla_chat_types::TEAM_MSG_COMMAND_INCOMING,
            &message,
            sender_name.clone(),
            Some(team_display.clone()),
            &recipients,
        );
    }
    server.broadcast_decorated_chat_to(
        &vanilla_chat_types::TEAM_MSG_COMMAND_OUTGOING,
        &message,
        sender_name,
        Some(team_display),
        std::slice::from_ref(&sender),
    );

    Ok((recipients.len() + 1) as i32)
}
