//! Cookie response packet (`ServerboundCookieResponsePacket`, login/config/play).
//!
//! Vanilla `PAYLOAD_STREAM_CODEC = ByteBufCodecs.byteArray(5120)`: rejects cookie
//! payloads exceeding 5 KiB.

use std::io::{Cursor, Error, ErrorKind, Read as _, Result};

use steel_macros::ServerPacket;
use steel_utils::Identifier;
use steel_utils::codec::VarInt;
use steel_utils::serial::ReadFrom;

/// Vanilla `ClientboundStoreCookiePacket.MAX_PAYLOAD_SIZE` (5 KiB).
pub const MAX_COOKIE_PAYLOAD_SIZE: usize = 5_120;

/// Serverbound answer to a cookie request.
#[derive(ServerPacket, Debug, Clone, PartialEq, Eq)]
pub struct SCookieResponse {
    /// Cookie key.
    pub key: Identifier,
    /// Stored payload (`<= 5120` bytes), or `None` when the client has no such cookie.
    pub payload: Option<Vec<u8>>,
}

impl ReadFrom for SCookieResponse {
    fn read(data: &mut Cursor<&[u8]>) -> Result<Self> {
        let key = Identifier::read(data)?;
        let has_payload = bool::read(data)?;
        let payload = if has_payload {
            let len_raw = VarInt::read(data)?.0;
            let len = usize::try_from(len_raw).map_err(|_| {
                Error::new(ErrorKind::InvalidData, "negative cookie payload length")
            })?;
            if len > MAX_COOKIE_PAYLOAD_SIZE {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("cookie payload of {len} bytes exceeds {MAX_COOKIE_PAYLOAD_SIZE}"),
                ));
            }
            let mut buf = vec![0_u8; len];
            data.read_exact(&mut buf)?;
            Some(buf)
        } else {
            None
        };
        Ok(Self { key, payload })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use steel_utils::serial::WriteTo as _;

    #[test]
    fn enforces_five_kib_cookie_cap() {
        let mut ok_wire = Vec::new();
        "minecraft:session".to_owned().write(&mut ok_wire).unwrap();
        ok_wire.extend_from_slice(&[1, 2, 0xCA, 0xFE]);
        let ok = SCookieResponse::read(&mut Cursor::new(&ok_wire)).unwrap();
        assert_eq!(ok.payload, Some(vec![0xCA, 0xFE]));

        let mut oversized = Vec::new();
        "minecraft:session".to_owned().write(&mut oversized).unwrap();
        oversized.push(1);
        VarInt((MAX_COOKIE_PAYLOAD_SIZE + 1) as i32)
            .write(&mut oversized)
            .unwrap();
        assert!(SCookieResponse::read(&mut Cursor::new(&oversized)).is_err());
    }
}
