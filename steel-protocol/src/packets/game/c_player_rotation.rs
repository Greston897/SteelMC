//! Player rotation packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_PLAYER_ROTATION;

/// Clientbound packet that forces the player's rotation.
///
/// Vanilla `ClientboundPlayerRotationPacket`: yaw, yaw relative flag, pitch,
/// pitch relative flag. The relative flags are for `Entity.forceSetRotation`
/// callers such as `EntitySelector`; a plain absolute move sends `false, false`.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_PLAYER_ROTATION)]
pub struct CPlayerRotation {
    /// Yaw in degrees.
    pub yaw: f32,
    /// Whether the client should rotate by `yaw` instead of setting it.
    pub relative_yaw: bool,
    /// Pitch in degrees.
    pub pitch: f32,
    /// Whether the client should rotate by `pitch` instead of setting it.
    pub relative_pitch: bool,
}

impl CPlayerRotation {
    /// An absolute rotation, which is what a teleport or `/rotate` sets.
    #[must_use]
    pub const fn absolute(yaw: f32, pitch: f32) -> Self {
        Self {
            yaw,
            relative_yaw: false,
            pitch,
            relative_pitch: false,
        }
    }
}
