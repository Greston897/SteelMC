//! Set test block packet.

use steel_macros::{ReadFrom, ServerPacket};
use steel_utils::BlockPos;
use steel_utils::codec::VarInt;

/// Serverbound packet that edits a test block.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SSetTestBlock {
    /// Test block position.
    pub pos: BlockPos,
    /// Mode ordinal (`start`, `log`, `fail`, `accept`).
    #[read(as = VarInt)]
    pub mode: i32,
    /// Message sent together with the action.
    pub message: String,
}
