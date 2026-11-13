//! Player rotation packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_utils::serial::WriteTo as _;
use steel_registry::packets::play::C_PLAYER_ROTATION;

/// Clientbound packet that forces the player's body rotation.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_PLAYER_ROTATION)]
pub struct CPlayerRotation {
    /// Body yaw in degrees.
    pub yaw: f32,
    /// Head pitch in degrees.
    pub pitch: f32,
    /// Whether the head rotates relative to the body.
    pub relative: bool,
}
