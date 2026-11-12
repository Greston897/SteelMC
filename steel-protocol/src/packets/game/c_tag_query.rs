//! Tag query response packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_TAG_QUERY;
use steel_utils::codec::VarInt;

/// One tag entry of a tag-query response.
#[derive(WriteTo, Clone, Debug)]
pub struct TagEntry {
    /// Tag identifier.
    pub tag: steel_utils::Identifier,
    /// Registry ids carrying this tag.
    #[write(as = Prefixed(VarInt))]
    pub entries: Vec<i32>,
}

/// Clientbound response to a `tag_query` request.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_TAG_QUERY)]
pub struct CTagQuery {
    /// Transaction id of the request this answers.
    #[write(as = VarInt)]
    pub transaction_id: i32,
    /// Tags of the requested registry.
    #[write(as = Prefixed(VarInt))]
    pub tags: Vec<TagEntry>,
}
