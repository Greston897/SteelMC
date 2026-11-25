//! Clear dialog packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::{config, play};

/// Clientbound packet that closes the currently open dialog.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Config = config::C_CLEAR_DIALOG, Play = play::C_CLEAR_DIALOG)]
pub struct CClearDialog {}
