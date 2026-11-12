//! Pong packet.

use steel_macros::{ReadFrom, ServerPacket};

/// Serverbound answer to a clientbound ping.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SPong {
    /// Opaque id echoed from the ping.
    pub id: i32,
}
