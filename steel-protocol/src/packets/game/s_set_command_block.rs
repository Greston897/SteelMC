//! Set command block packet.

use steel_macros::{ReadFrom, ServerPacket};
use steel_utils::BlockPos;
use steel_utils::codec::VarInt;

/// Serverbound packet that edits a command block.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SSetCommandBlock {
    /// Command block position.
    pub pos: BlockPos,
    /// Command text.
    pub command: String,
    /// Vanilla `CommandBlockEntity.Mode` ordinal.
    #[read(as = VarInt)]
    pub mode: i32,
    /// Vanilla flags: `1` track output, `2` conditional, `4` automatic.
    pub flags: u8,
}
