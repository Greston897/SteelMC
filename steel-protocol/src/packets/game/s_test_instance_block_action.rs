//! Test instance block action packet.

use steel_macros::{ReadFrom, ServerPacket};
use steel_utils::BlockPos;
use steel_utils::codec::VarInt;

/// Serverbound packet that drives a test instance block.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct STestInstanceBlockAction {
    /// Test instance block position.
    pub pos: BlockPos,
    /// Action ordinal (`start`, `clear`, `reset`, `save`, `export`).
    #[read(as = VarInt)]
    pub action: i32,
    /// Payload of the action.
    pub test: String,
}
