//! Lock difficulty packet.

use steel_macros::{ReadFrom, ServerPacket};

/// Serverbound packet used in single player to lock the difficulty.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SLockDifficulty {
    /// Whether the difficulty becomes locked.
    pub locked: bool,
}
