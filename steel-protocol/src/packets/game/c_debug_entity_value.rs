//! Debug entity-value packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_utils::serial::WriteTo as _;
use steel_registry::packets::play::C_DEBUG_ENTITY_VALUE;
use steel_utils::codec::VarInt;

/// Clientbound debug payload with one entity value.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_DEBUG_ENTITY_VALUE)]
pub struct CDebugEntityValue {
    /// Debug subscription that produced the value.
    pub subscription_id: i64,
    /// Entity id.
    #[write(as = VarInt)]
    pub entity_id: i32,
    /// Value kind ordinal.
    #[write(as = VarInt)]
    pub kind: i32,
    /// Value payload.
    pub value: i64,
}
