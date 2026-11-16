//! Combat end packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_PLAYER_COMBAT_END;
use steel_utils::codec::VarInt;

/// Clientbound packet that ends combat after the cooldown.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_PLAYER_COMBAT_END)]
pub struct CPlayerCombatEnd {
    /// Ticks the client still has to wait before leaving combat.
    #[write(as = VarInt)]
    pub combat_time: i32,
}
