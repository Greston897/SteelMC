//! Jigsaw generate packet.

use steel_macros::{ReadFrom, ServerPacket};
use steel_utils::BlockPos;
use steel_utils::codec::VarInt;

/// Serverbound packet that regenerates a jigsaw block's structure.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SJigsawGenerate {
    /// Jigsaw block position.
    pub pos: BlockPos,
    /// Number of generation levels.
    #[read(as = VarInt)]
    pub levels: i32,
    /// Whether the existing structure is kept.
    pub keep_jigsaws: bool,
}
