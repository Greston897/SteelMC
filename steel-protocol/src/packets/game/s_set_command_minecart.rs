//! Set command minecart packet.

use steel_macros::{ReadFrom, ServerPacket};

/// Serverbound packet that edits a command block minecart.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SSetCommandMinecart {
    /// Command text.
    pub command: String,
    /// Whether the command block tracks its output.
    pub track_output: bool,
}
