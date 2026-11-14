//! Set jigsaw block packet.

use steel_macros::{ReadFrom, ServerPacket};
use steel_utils::{BlockPos, Identifier};
use steel_utils::codec::VarInt;

/// Serverbound packet that edits a jigsaw block.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SSetJigsawBlock {
    /// Jigsaw block position.
    pub pos: BlockPos,
    /// Name of the generated feature.
    pub name: Identifier,
    /// Target pool to connect with.
    pub target: Identifier,
    /// Pool used for the attached piece.
    pub pool: Identifier,
    /// Piece to generate when the jigsaw is broken.
    pub final_state: String,
    /// Joint type ordinal (`rollable` or `aligned`).
    #[read(as = VarInt)]
    pub joint: i32,
    /// Generation priority.
    #[read(as = VarInt)]
    pub selection_priority: i32,
    /// Placement priority.
    #[read(as = VarInt)]
    pub placement_priority: i32,
}
