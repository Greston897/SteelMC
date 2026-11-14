//! Teleport to entity packet.

use steel_macros::{ReadFrom, ServerPacket};
use uuid::Uuid;

/// Serverbound spectator teleport request.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct STeleportToEntity {
    /// `UUID` of the target player.
    pub target: Uuid,
}
