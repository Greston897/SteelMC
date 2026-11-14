//! Tag query response packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_TAG_QUERY;
use steel_utils::codec::VarInt;
use steel_utils::serial::OptionalNbt;

/// Clientbound response to a `block_entity_tag_query` or `entity_tag_query`.
///
/// Vanilla `ClientboundTagQueryPacket` answers the `/data get` transaction with
/// the queried object's NBT, or a `null` tag when nothing was found.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_TAG_QUERY)]
pub struct CTagQuery {
    /// Transaction id of the request this answers.
    #[write(as = VarInt)]
    pub transaction_id: i32,
    /// The queried NBT, or `None` when the object does not exist.
    pub tag: OptionalNbt,
}
