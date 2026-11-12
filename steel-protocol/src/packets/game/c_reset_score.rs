//! Scoreboard score reset packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_RESET_SCORE;

/// Clientbound packet that clears a score from one objective.
///
/// Vanilla encodes "every objective" as an empty objective name, so the packet
/// carries a plain `String` instead of an optional component.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_RESET_SCORE)]
pub struct CResetScore {
    /// Target owner name.
    pub owner: String,
    /// Objective name; an empty string clears the score in every objective.
    pub objective_name: String,
}
