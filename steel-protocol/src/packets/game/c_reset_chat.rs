//! Reset chat packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_utils::serial::WriteTo as _;
use steel_registry::packets::config::C_RESET_CHAT;

/// Clientbound packet that clears the client chat history.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Config = C_RESET_CHAT)]
pub struct CResetChat {}
