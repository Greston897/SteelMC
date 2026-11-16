//! Store cookie packet (`ClientboundStoreCookiePacket`).
//!
//! Vanilla enforces `MAX_PAYLOAD_SIZE = 5120` bytes via `ByteBufCodecs.byteArray(5120)`.

use std::io::{Error, ErrorKind, Write};

use steel_macros::ClientPacket;
use steel_registry::packets::{config, play};
use steel_utils::Identifier;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;

use crate::packets::login::s_cookie_response::MAX_COOKIE_PAYLOAD_SIZE;

/// Clientbound packet instructing the client to persist a cookie (max 5 KiB).
#[derive(ClientPacket, Clone, Debug, PartialEq, Eq)]
#[packet_id(Config = config::C_STORE_COOKIE, Play = play::C_STORE_COOKIE)]
pub struct CStoreCookie {
    /// Cookie key.
    pub key: Identifier,
    /// Opaque payload (at most [`MAX_COOKIE_PAYLOAD_SIZE`] = 5120 bytes).
    pub payload: Vec<u8>,
}

impl WriteTo for CStoreCookie {
    fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        if self.payload.len() > MAX_COOKIE_PAYLOAD_SIZE {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "cookie payload of {} bytes exceeds {MAX_COOKIE_PAYLOAD_SIZE}",
                    self.payload.len()
                ),
            ));
        }
        self.key.write(writer)?;
        VarInt(self.payload.len() as i32).write(writer)?;
        writer.write_all(&self.payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_valid_cookie_and_rejects_over_five_kib() {
        let ok = CStoreCookie {
            key: Identifier::vanilla_static("token"),
            payload: vec![1, 2, 3],
        };
        let mut bytes = Vec::new();
        ok.write(&mut bytes).unwrap();
        assert_eq!(&bytes[bytes.len() - 4..], &[3, 1, 2, 3]);

        let too_big = CStoreCookie {
            key: Identifier::vanilla_static("token"),
            payload: vec![0; MAX_COOKIE_PAYLOAD_SIZE + 1],
        };
        assert!(too_big.write(&mut Vec::new()).is_err());
    }
}
