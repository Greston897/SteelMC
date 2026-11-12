//! Resource pack response packet.

use steel_macros::{ReadFrom, ServerPacket};
use steel_utils::codec::VarInt;
use uuid::Uuid;

/// Serverbound packet that reports the result of a resource pack push.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SResourcePack {
    /// `UUID` of the pack the result belongs to.
    pub id: Uuid,
    /// Vanilla `ServerboundResourcePackPacket.Action` ordinal.
    #[read(as = VarInt)]
    pub action: i32,
}
