//! Debug block-value packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_utils::serial::WriteTo as _;
use steel_registry::packets::play::C_DEBUG_BLOCK_VALUE;
use steel_utils::BlockPos;

/// Clientbound debug payload with the numeric value of one block property.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_DEBUG_BLOCK_VALUE)]
pub struct CDebugBlockValue {
    /// Debug subscription that produced the value.
    pub subscription_id: i64,
    /// Block position.
    pub pos: BlockPos,
    /// Property index inside the block state.
    pub property: i32,
    /// Current value of the property.
    pub value: i32,
}
