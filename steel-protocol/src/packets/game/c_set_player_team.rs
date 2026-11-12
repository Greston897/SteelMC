//! Scoreboard team packet.

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_SET_PLAYER_TEAM;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;
use text_components::TextComponent;

/// Vanilla `ChatFormatting` ordinal used by teams (`-1` means inherit).
pub const NO_COLOR: i32 = -1;

/// Vanilla `ClientboundSetPlayerTeamPacket` actions.
#[derive(Debug, Clone)]
pub enum TeamAction {
    /// Creates a team (`mode = 0`).
    Add {
        /// Definition of the created team.
        parameters: TeamParameters,
    },
    /// Removes a team (`mode = 1`).
    Remove,
    /// Changes the team parameters (`mode = 2`).
    Update {
        /// New team definition.
        parameters: TeamParameters,
    },
    /// Adds players to the team (`mode = 3`).
    AddPlayers {
        /// Player names to add.
        players: Vec<String>,
    },
    /// Removes players from the team (`mode = 4`).
    RemovePlayers {
        /// Player names to remove.
        players: Vec<String>,
    },
}

/// The mutable part of a scoreboard team.
#[derive(Debug, Clone)]
pub struct TeamParameters {
    /// Displayed team name.
    pub display_name: TextComponent,
    /// Vanilla flags: `1` friendly fire, `2` see friendly invisibles.
    pub flags: u8,
    /// Team name tag visibility (`always`, `hideForOtherTeams`, …).
    pub name_tag_visibility: String,
    /// Team collision rule (`always`, `pushOtherTeams`, …).
    pub collision_rule: String,
    /// Team colour ordinal, or [`NO_COLOR`].
    pub color: i32,
    /// Prefix rendered before the player name.
    pub prefix: TextComponent,
    /// Suffix rendered after the player name.
    pub suffix: TextComponent,
}

/// Clientbound packet that creates, removes or edits a scoreboard team.
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Play = C_SET_PLAYER_TEAM)]
pub struct CSetPlayerTeam {
    /// Team name.
    pub name: String,
    /// Action to apply.
    pub action: TeamAction,
}

impl WriteTo for CSetPlayerTeam {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        self.name.write(writer)?;
        match &self.action {
            TeamAction::Add { parameters } => {
                0_i8.write(writer)?;
                write_parameters(parameters, writer)?;
            }
            TeamAction::Remove => {
                1_i8.write(writer)?;
            }
            TeamAction::Update { parameters } => {
                2_i8.write(writer)?;
                write_parameters(parameters, writer)?;
            }
            TeamAction::AddPlayers { players } => {
                3_i8.write(writer)?;
                players.write_prefixed::<VarInt>(writer)?;
            }
            TeamAction::RemovePlayers { players } => {
                4_i8.write(writer)?;
                players.write_prefixed::<VarInt>(writer)?;
            }
        }
        Ok(())
    }
}

fn write_parameters(
    parameters: &TeamParameters,
    writer: &mut impl std::io::Write,
) -> std::io::Result<()> {
    parameters.display_name.write(writer)?;
    (parameters.flags as i8).write(writer)?;
    parameters.name_tag_visibility.write(writer)?;
    parameters.collision_rule.write(writer)?;
    VarInt(parameters.color).write(writer)?;
    parameters.prefix.write(writer)?;
    parameters.suffix.write(writer)?;
    Ok(())
}
