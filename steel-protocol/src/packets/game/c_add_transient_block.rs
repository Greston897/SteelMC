//! Transient block packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_ADD_TRANSIENT_BLOCK;
use steel_utils::BlockPos;

/// Clientbound packet that shows a temporary block entity (chest preview etc).
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_ADD_TRANSIENT_BLOCK)]
pub struct CAddTransientBlock {
    /// Position of the transient block.
    pub pos: BlockPos,
}
