//! Low disk space warning packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_LOW_DISK_SPACE_WARNING;

/// Clientbound packet that warns the client about low disk space.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_LOW_DISK_SPACE_WARNING)]
pub struct CLowDiskSpaceWarning {}
