//! Pick item from entity packet.

use steel_macros::{ReadFrom, ServerPacket};
use steel_utils::codec::VarInt;

/// Serverbound packet sent when the player middle-clicks an entity.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SPickItemFromEntity {
    /// Entity id.
    #[read(as = VarInt)]
    pub entity_id: i32,
    /// Whether the picked stack includes the entity's custom data.
    pub include_data: bool,
}
