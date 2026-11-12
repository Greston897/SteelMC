//! Projectile power packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_PROJECTILE_POWER;
use steel_utils::codec::VarInt;

/// Clientbound packet that changes the air drag of a projectile.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_PROJECTILE_POWER)]
pub struct CProjectilePower {
    /// Projectile entity id.
    #[write(as = VarInt)]
    pub entity_id: i32,
    /// Per-tick drag multiplier applied by the client.
    pub acceleration_power: f64,
}
