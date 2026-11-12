//! Debug event packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_DEBUG_EVENT;

/// Clientbound debug event.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_DEBUG_EVENT)]
pub struct CDebugEvent {
    /// Event kind ordinal.
    #[write(as = VarInt)]
    pub event: i32,
    /// Event payload.
    pub payload: Vec<u8>,
}
