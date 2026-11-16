//! Start configuration packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_START_CONFIGURATION;

/// Clientbound packet that asks the client to re-enter the configuration state.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_START_CONFIGURATION)]
pub struct CStartConfiguration {}
