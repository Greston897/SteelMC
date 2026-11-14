//! Delete chat packet.

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_DELETE_CHAT;
use steel_utils::serial::WriteTo;

/// Vanilla `ChatType` signature marker used to build the message signature.
#[derive(Debug, Clone)]
pub struct MessageSignature {
    /// 256-byte signature payload.
    pub bytes: Vec<u8>,
}

/// Clientbound packet that removes one chat message from the client history.
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Play = C_DELETE_CHAT)]
pub struct CDeleteChat {
    /// The signature of the message to delete.
    pub signature: Option<MessageSignature>,
}

impl WriteTo for CDeleteChat {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        match &self.signature {
            Some(signature) => {
                true.write(writer)?;
                signature.bytes.write(writer)
            }
            None => false.write(writer),
        }
    }
}
