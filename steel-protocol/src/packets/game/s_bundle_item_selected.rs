//! Bundle item selection packet.

use steel_macros::{ReadFrom, ServerPacket};
use steel_utils::codec::VarInt;

/// Serverbound packet sent when the player scrolls inside a bundle.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SBundleItemSelected {
    /// Inventory slot of the bundle.
    #[read(as = VarInt)]
    pub slot_id: i32,
    /// Index of the selected item inside the bundle.
    #[read(as = VarInt)]
    pub selected_item_index: i32,
}
