//! Play-state ping packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_PING;

/// Clientbound packet that pings the client during play.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_PING)]
pub struct CPingPlay {
    /// Opaque id echoed back in the pong.
    pub id: i32,
}
