//! Configuration acknowledged packet.

use steel_macros::{ReadFrom, ServerPacket};

/// Serverbound packet sent after the client re-entered the configuration state.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SConfigurationAcknowledged {}
