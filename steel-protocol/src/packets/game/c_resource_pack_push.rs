//! Resource pack push packet.

use std::io;

use steel_macros::ClientPacket;
use steel_registry::packets::{config, play};
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo as _;
use text_components::TextComponent;
use uuid::Uuid;

/// Vanilla `ClientboundResourcePackPushPacket.MAX_HASH_LENGTH`.
pub const MAX_RESOURCE_PACK_HASH_LENGTH: usize = 40;

/// Clientbound packet that asks the client to download and apply one pack.
///
/// Vanilla pushes packs one per packet (`ClientboundResourcePackPushPacket`),
/// not as a list: `ServerCommonPacketListenerImpl.sendResourcePackPush` builds
/// a packet per URL and the client answers each one with `SResourcePack`.
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Config = config::C_RESOURCE_PACK_PUSH, Play = play::C_RESOURCE_PACK_PUSH)]
pub struct CResourcePackPush {
    /// Pack `UUID` used by the client to report the result.
    pub id: Uuid,
    /// Download URL.
    pub url: String,
    /// Lowercase hex SHA-1, at most [`MAX_RESOURCE_PACK_HASH_LENGTH`] chars.
    ///
    /// Vanilla sends an empty string when the hash is not enforced.
    pub hash: String,
    /// Whether the pack is mandatory: declining it disconnects the player.
    pub required: bool,
    /// Text shown while the client prompts the player, if any.
    pub prompt: Option<TextComponent>,
}

impl steel_utils::serial::WriteTo for CResourcePackPush {
    fn write(&self, writer: &mut impl io::Write) -> io::Result<()> {
        debug_assert!(
            self.hash.len() <= MAX_RESOURCE_PACK_HASH_LENGTH,
            "resource pack hash is longer than vanilla allows"
        );
        self.id.write(writer)?;
        self.url.write(writer)?;
        self.hash.write(writer)?;
        self.required.write(writer)?;
        self.prompt.write(writer)
    }
}
