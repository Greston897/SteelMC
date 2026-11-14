//! Block entity tag query packet.

use steel_macros::{ReadFrom, ServerPacket};
use steel_utils::BlockPos;
use steel_utils::codec::VarInt;

/// Serverbound request for the block entity tags at a position.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SBlockEntityTagQuery {
    /// Transaction id echoed in the response.
    #[read(as = VarInt)]
    pub transaction_id: i32,
    /// Position of the queried block entity.
    pub pos: BlockPos,
}
