//! Debug chunk-value packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_utils::serial::WriteTo as _;
use steel_registry::packets::play::C_DEBUG_CHUNK_VALUE;
use steel_utils::codec::VarInt;

/// Clientbound debug payload with one chunk value.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_DEBUG_CHUNK_VALUE)]
pub struct CDebugChunkValue {
    /// Debug subscription that produced the value.
    pub subscription_id: i64,
    /// Chunk x coordinate.
    #[write(as = VarInt)]
    pub x: i32,
    /// Chunk z coordinate.
    #[write(as = VarInt)]
    pub z: i32,
    /// Value kind ordinal.
    #[write(as = VarInt)]
    pub kind: i32,
    /// Value payload.
    pub value: i64,
}
