//! Delete signed chat message packet (`ClientboundDeleteChatPacket`).
//!
//! Vanilla `MessageSignature.Packed.write`:
//! - `VarInt(id + 1)`
//! - when `fullSignature` is sent (`id == -1`, so `VarInt(0)`), writes the raw
//!   256-byte signature (`output.writeBytes(fullSignature.bytes())`, NO length prefix).

use std::io::Write;

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_DELETE_CHAT;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;

/// Vanilla `MessageSignature.Packed`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackedMessageSignature {
    /// Index into the client's seen-signature cache (`id >= 0`, encoded as `VarInt(id + 1)`).
    CachedId(i32),
    /// Full 256-byte RSA signature (encoded as `VarInt(0)` followed by 256 raw bytes).
    Full(Box<[u8; 256]>),
}

/// Clientbound packet instructing clients to hide/delete a signed chat message.
#[derive(ClientPacket, Clone, Debug, PartialEq, Eq)]
#[packet_id(Play = C_DELETE_CHAT)]
pub struct CDeleteChat {
    /// Packed signature identifying the message to remove.
    pub message_signature: PackedMessageSignature,
}

impl WriteTo for CDeleteChat {
    fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match &self.message_signature {
            PackedMessageSignature::CachedId(id) => VarInt(id.saturating_add(1)).write(writer),
            PackedMessageSignature::Full(sig) => {
                VarInt(0).write(writer)?;
                writer.write_all(sig.as_slice())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CDeleteChat, PackedMessageSignature};
    use steel_utils::serial::WriteTo as _;

    #[test]
    fn encodes_cached_and_full_message_signatures() {
        let cached = CDeleteChat {
            message_signature: PackedMessageSignature::CachedId(7),
        };
        let mut bytes = Vec::new();
        cached.write(&mut bytes).unwrap();
        assert_eq!(bytes, vec![8]);

        let full = CDeleteChat {
            message_signature: PackedMessageSignature::Full(Box::new([0xAB; 256])),
        };
        let mut full_bytes = Vec::new();
        full.write(&mut full_bytes).unwrap();
        assert_eq!(full_bytes.len(), 257);
        assert_eq!(full_bytes[0], 0);
        assert!(full_bytes[1..].iter().all(|&b| b == 0xAB));
    }
}
