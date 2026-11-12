//! Scoreboard score update packet.

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_SET_SCORE;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;

/// One score entry update.
#[derive(Debug, Clone)]
pub struct ScoreEntry {
    /// Target owner name (player name or an entity `UUID` string).
    pub owner: String,
    /// Objective the score belongs to.
    pub objective_name: String,
    /// New score value.
    pub value: i32,
}

/// Clientbound packet that writes one score.
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Play = C_SET_SCORE)]
pub struct CSetScore {
    /// Scores to update, written one after another with no length prefix.
    pub entries: Vec<ScoreEntry>,
}

impl WriteTo for CSetScore {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        for entry in &self.entries {
            entry.owner.write(writer)?;
            entry.objective_name.write(writer)?;
            VarInt(entry.value).write(writer)?;
        }
        Ok(())
    }
}
