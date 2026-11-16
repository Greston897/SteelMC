//! Clientbound ping packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_PING;

/// Clientbound packet that asks the client to answer with a pong.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_PING)]
pub struct CPing {
    /// Opaque id echoed back in the pong.
    pub id: i32,
}
