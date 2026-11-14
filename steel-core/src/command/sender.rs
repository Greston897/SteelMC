//! Module defining the sender of a command.
use std::{fmt, sync::Arc};
use text_components::TextComponent;
use uuid::Uuid;

use crate::{
    player::{DomainResidenceToken, Player},
    server::Server,
};

/// The sender of a command.
#[derive(Clone)]
pub enum CommandSender {
    /// The command was sent by a player via the chat.
    Player(Arc<Player>),
    /// The command was sent via the server's console.
    Console,
    /// The command was sent via Rcon.
    ///
    /// Carries the reply channel of the connection so the console output of the
    /// command reaches the RCON client, exactly like vanilla
    /// `RconConsoleSource.sendSystemMessage`.
    Rcon(RconOutput),
}

/// Sink for the text a command sends to an RCON client.
#[derive(Clone)]
pub struct RconOutput {
    sender: tokio::sync::mpsc::UnboundedSender<String>,
}

impl RconOutput {
    /// Creates an output sink backed by `sender`.
    #[must_use]
    pub fn new(sender: tokio::sync::mpsc::UnboundedSender<String>) -> Self {
        Self { sender }
    }

    /// Appends one line to the client's reply.
    ///
    /// A closed receiver means the client disconnected mid-command, which is
    /// normal and therefore ignored.
    pub fn send(&self, line: String) {
        let _ = self.sender.send(line);
    }
}

impl fmt::Debug for RconOutput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("RconOutput").finish_non_exhaustive()
    }
}

/// Stable identity used to preserve top-level command ordering while work is suspended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum CommandSenderKey {
    Player(Uuid),
    Console,
    Rcon,
}

/// Exact key used to coalesce suggestions from one live player residence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CommandSuggestionKey {
    Player {
        uuid: Uuid,
        session_address: usize,
        residence: Option<DomainResidenceToken>,
    },
    Console,
    Rcon,
}

/// Exact runtime owner of queued command work.
///
/// UUIDs remain the ordering key, while the player `Arc` and residence token
/// prevent an old session or an earlier domain stay from resuming work.
#[derive(Clone)]
pub(crate) struct CommandExecutionOwner {
    sender: CommandSender,
    player_residence: Option<DomainResidenceToken>,
}

impl CommandExecutionOwner {
    pub(crate) fn capture(sender: CommandSender, server: &Server) -> Self {
        let player_residence = sender.get_player().and_then(|player| {
            // Snapshot first so a concurrent detach cannot pair source
            // availability with the next domain residence.
            let residence = player.domain_residence_token();
            server.command_world_for_player(player).map(|_| residence)
        });
        Self {
            sender,
            player_residence,
        }
    }

    #[cfg(test)]
    pub(crate) fn non_player_for_test(sender: CommandSender) -> Self {
        assert!(
            sender.get_player().is_none(),
            "test helper only constructs non-player command owners"
        );
        Self {
            sender,
            player_residence: None,
        }
    }

    pub(crate) fn key(&self) -> CommandSenderKey {
        self.sender.key()
    }

    pub(crate) fn suggestion_key(&self) -> CommandSuggestionKey {
        match &self.sender {
            CommandSender::Player(player) => CommandSuggestionKey::Player {
                uuid: player.gameprofile.id(),
                // The owner retains this Arc while queued, so its allocation
                // address cannot be reused by a replacement session.
                session_address: Arc::as_ptr(player) as usize,
                residence: self.player_residence,
            },
            CommandSender::Console => CommandSuggestionKey::Console,
            CommandSender::Rcon(_) => CommandSuggestionKey::Rcon,
        }
    }

    pub(crate) const fn sender(&self) -> &CommandSender {
        &self.sender
    }

    pub(crate) fn is_current(&self, server: &Server) -> bool {
        let CommandSender::Player(player) = &self.sender else {
            return true;
        };
        let Some(residence) = self.player_residence else {
            return false;
        };
        server.command_world_for_player(player).is_some()
            && player.is_domain_residence_current(residence)
    }
}

impl CommandSender {
    pub(crate) fn key(&self) -> CommandSenderKey {
        match self {
            Self::Player(player) => CommandSenderKey::Player(player.gameprofile.id()),
            Self::Console => CommandSenderKey::Console,
            Self::Rcon(_) => CommandSenderKey::Rcon,
        }
    }

    /// Returns the player if the sender is a player.
    #[must_use]
    pub const fn get_player(&self) -> Option<&Arc<Player>> {
        match self {
            Self::Player(player) => Some(player),
            _ => None,
        }
    }

    /// Sends a system message to the command sender.
    pub fn send_message(&self, text: &TextComponent) {
        match self {
            Self::Player(player) => player.send_message(text),
            Self::Console => log::info!("{text}"),
            Self::Rcon(output) => {
                // Vanilla's RCON source keeps every message in a buffer that is
                // packetized into one reply once the command finishes.
                let plain = text.to_string();
                output.send(plain);
            }
        }
    }
}

impl fmt::Display for CommandSender {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Player(p) => &p.gameprofile.name(),
                Self::Console => "Server",
                Self::Rcon(_) => "Rcon",
            }
        )
    }
}
