//! Entity tag query packet.

use steel_macros::{ReadFrom, ServerPacket};
use steel_utils::codec::VarInt;

/// Serverbound request for the tags of an entity.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SEntityTagQuery {
    /// Transaction id echoed in the response.
    #[read(as = VarInt)]
    pub transaction_id: i32,
    /// Entity id.
    #[read(as = VarInt)]
    pub entity_id: i32,
}
