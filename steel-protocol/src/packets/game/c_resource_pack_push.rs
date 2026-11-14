//! Resource pack push packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::{config, play};
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo as _;
use uuid::Uuid;

/// One resource pack entry pushed to the client.
#[derive(WriteTo, Debug, Clone)]
pub struct ResourcePackEntry {
    /// Pack `UUID` used by the client to report the result.
    pub id: Uuid,
    /// Download URL.
    pub url: String,
    /// Expected SHA-1 hash, or `None` when not enforced.
    pub hash: Option<String>,
}

/// Clientbound packet that asks the client to download and apply packs.
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Config = config::C_RESOURCE_PACK_PUSH, Play = play::C_RESOURCE_PACK_PUSH)]
pub struct CResourcePackPush {
    /// Packs to apply, in order.
    pub packs: Vec<ResourcePackEntry>,
    /// Text shown while the client prompts the player, if any.
    pub prompt: Option<text_components::TextComponent>,
    /// Whether the packs are mandatory.
    pub required: bool,
}

impl steel_utils::serial::WriteTo for CResourcePackPush {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        VarInt(self.packs.len() as i32).write(writer)?;
        for pack in &self.packs {
            pack.id.write(writer)?;
            pack.url.write(writer)?;
            pack.hash.write(writer)?;
        }
        self.prompt.write(writer)?;
        self.required.write(writer)
    }
}
