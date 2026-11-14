//! Select trade packet.

use steel_macros::{ReadFrom, ServerPacket};
use steel_utils::codec::VarInt;

/// Serverbound packet sent when the player selects a merchant offer.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SSelectTrade {
    /// Index of the selected offer in the merchant list.
    #[read(as = VarInt)]
    pub item: i32,
}
