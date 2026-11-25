//! Scoreboard team management packet (`ClientboundSetPlayerTeamPacket`).
//!
//! Vanilla 26.3 wire format:
//! 1. `name`: UTF-8 string
//! 2. `method`: `i8` (`0 = ADD`, `1 = REMOVE`, `2 = CHANGE`, `3 = JOIN`, `4 = LEAVE`)
//! 3. If `method == 0 || method == 2`, `Parameters.STREAM_CODEC`:
//!    - `displayName`: `Component`
//!    - `playerPrefix`: `Component`
//!    - `playerSuffix`: `Component`
//!    - `nameTagVisibility`: `VarInt` (`0 = always`, `1 = never`, `2 = hideForOtherTeams`, `3 = hideForOwnTeam`)
//!    - `collisionRule`: `VarInt` (`0 = always`, `1 = never`, `2 = pushOtherTeams`, `3 = pushOwnTeam`)
//!    - `color`: `Optional<TeamColor>` (`bool` + `VarInt` `0..=15`)
//!    - `options`: `u8` (bit `0x01` friendly fire, bit `0x02` see friendly invisibles)
//! 4. If `method == 0 || method == 3 || method == 4`, `players`: `VarInt`-prefixed list of UTF-8 strings.

use std::io::Write;

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_SET_PLAYER_TEAM;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;
use text_components::TextComponent;

/// Sentinel retained for callers converting optional team colors (`None` on wire).
pub const NO_COLOR: i32 = -1;

/// Vanilla `ClientboundSetPlayerTeamPacket.Parameters`.
#[derive(Debug, Clone)]
pub struct TeamParameters {
    /// Team display name.
    pub display_name: TextComponent,
    /// Prefix rendered before each member's name.
    pub prefix: TextComponent,
    /// Suffix rendered after each member's name.
    pub suffix: TextComponent,
    /// Vanilla `Team.Visibility` id (`0=always, 1=never, 2=hideForOtherTeams, 3=hideForOwnTeam`).
    pub name_tag_visibility: i32,
    /// Vanilla `Team.CollisionRule` id (`0=always, 1=never, 2=pushOtherTeams, 3=pushOwnTeam`).
    pub collision_rule: i32,
    /// Vanilla `Optional<TeamColor>` (`Some(0..=15)` or `None`).
    pub color: Option<i32>,
    /// Option bitfield (`0x01` friendly fire, `0x02` see friendly invisibles).
    pub options: u8,
}

impl WriteTo for TeamParameters {
    fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.display_name.write(writer)?;
        self.prefix.write(writer)?;
        self.suffix.write(writer)?;
        VarInt(self.name_tag_visibility).write(writer)?;
        VarInt(self.collision_rule).write(writer)?;
        match self.color {
            Some(color_id @ 0..=15) => {
                true.write(writer)?;
                VarInt(color_id).write(writer)?;
            }
            _ => {
                false.write(writer)?;
            }
        }
        self.options.write(writer)
    }
}

/// Vanilla `ClientboundSetPlayerTeamPacket` action (`method` byte).
#[derive(Debug, Clone)]
pub enum TeamAction {
    /// Creates the team with parameters and initial member list (`method = 0`).
    Add {
        /// Team parameters.
        parameters: TeamParameters,
        /// Initial members on the team.
        players: Vec<String>,
    },
    /// Deletes the team (`method = 1`).
    Remove,
    /// Updates team parameters (`method = 2`).
    Update {
        /// Updated team parameters.
        parameters: TeamParameters,
    },
    /// Adds players to the team (`method = 3`).
    AddPlayers {
        /// Player / score-holder names to add.
        players: Vec<String>,
    },
    /// Removes players from the team (`method = 4`).
    RemovePlayers {
        /// Player / score-holder names to remove.
        players: Vec<String>,
    },
}

/// Clientbound packet that manages a scoreboard team.
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Play = C_SET_PLAYER_TEAM)]
pub struct CSetPlayerTeam {
    /// Team identifier.
    pub name: String,
    /// Team action.
    pub action: TeamAction,
}

impl WriteTo for CSetPlayerTeam {
    fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.name.write(writer)?;
        match &self.action {
            TeamAction::Add {
                parameters,
                players,
            } => {
                0_i8.write(writer)?;
                parameters.write(writer)?;
                players.write(writer)
            }
            TeamAction::Remove => 1_i8.write(writer),
            TeamAction::Update { parameters } => {
                2_i8.write(writer)?;
                parameters.write(writer)
            }
            TeamAction::AddPlayers { players } => {
                3_i8.write(writer)?;
                players.write(writer)
            }
            TeamAction::RemovePlayers { players } => {
                4_i8.write(writer)?;
                players.write(writer)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_team_encodes_26_3_parameters_order_and_player_list() {
        let pkt = CSetPlayerTeam {
            name: "red".to_owned(),
            action: TeamAction::Add {
                parameters: TeamParameters {
                    display_name: TextComponent::plain("Red"),
                    prefix: TextComponent::plain("[R] "),
                    suffix: TextComponent::plain(""),
                    name_tag_visibility: 2,
                    collision_rule: 3,
                    color: Some(12),
                    options: 0x03,
                },
                players: vec!["Steve".to_owned()],
            },
        };
        let mut actual = Vec::new();
        pkt.write(&mut actual).unwrap();

        let mut expected = Vec::new();
        "red".to_owned().write(&mut expected).unwrap();
        0_i8.write(&mut expected).unwrap();
        TextComponent::plain("Red").write(&mut expected).unwrap();
        TextComponent::plain("[R] ").write(&mut expected).unwrap();
        TextComponent::plain("").write(&mut expected).unwrap();
        VarInt(2).write(&mut expected).unwrap(); // visibility
        VarInt(3).write(&mut expected).unwrap(); // collision
        true.write(&mut expected).unwrap(); // color present
        VarInt(12).write(&mut expected).unwrap(); // TeamColor.RED
        0x03_u8.write(&mut expected).unwrap(); // options byte
        vec!["Steve".to_owned()].write(&mut expected).unwrap(); // players list!
        assert_eq!(actual, expected);
    }
}
