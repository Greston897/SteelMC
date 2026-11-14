//! Configuration-state ping packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_utils::serial::WriteTo as _;
use steel_registry::packets::config::C_PING;

/// Clientbound packet that ping-pongs with the client during configuration.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Config = C_PING)]
pub struct CConfigPing {
    /// Opaque id echoed back in the pong.
    pub id: i32,
}
