//! Simulation distance packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_utils::serial::WriteTo as _;
use steel_registry::packets::play::C_SET_SIMULATION_DISTANCE;
use steel_utils::codec::VarInt;

/// Clientbound packet that changes the client's simulation distance.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_SET_SIMULATION_DISTANCE)]
pub struct CSetSimulationDistance {
    /// New simulation distance in chunks.
    #[write(as = VarInt)]
    pub distance: i32,
}
