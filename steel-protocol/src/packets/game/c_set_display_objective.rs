//! Scoreboard display-slot packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_utils::serial::WriteTo as _;
use steel_registry::packets::play::C_SET_DISPLAY_OBJECTIVE;
use steel_utils::codec::VarInt;

/// Clientbound packet that binds an objective to a display slot.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_SET_DISPLAY_OBJECTIVE)]
pub struct CSetDisplayObjective {
    /// Vanilla `DisplaySlot` ordinal (`0` list, `1` sidebar, `2..=18` team slots).
    #[write(as = VarInt)]
    pub slot: i32,
    /// Objective name, or an empty string to clear the slot.
    pub objective_name: String,
}
