//! Custom click action packet (`ServerboundCustomClickActionPacket`).
//!
//! Vanilla wire format (shared by `Config` and `Play`):
//! 1. `id`: `Identifier`
//! 2. `payload` (`UNTRUSTED_TAG_CODEC`): `ByteBufCodecs.lengthPrefixed(65536)` wrapping
//!    `ByteBufCodecs.optional(ByteBufCodecs.tagCodec(limit = 32768, depth = 16))`:
//!    - `VarInt` byte length (`0..=65536`)
//!    - inside that slice: `bool` presence flag, followed when `true` by an unnamed
//!      network NBT tag (e.g. `0x0A ... 0x00`).

use std::io::{Cursor, Error, ErrorKind, Read as _, Result};

use steel_macros::ServerPacket;
use steel_utils::Identifier;
use steel_utils::codec::VarInt;
use steel_utils::serial::ReadFrom;

/// Vanilla `ByteBufCodecs.lengthPrefixed(65536)` outer cap.
pub const MAX_CUSTOM_CLICK_PAYLOAD_LENGTH: usize = 65_536;

/// Serverbound packet emitted by a `custom` click event or dialog action.
#[derive(ServerPacket, Clone, Debug, PartialEq, Eq)]
pub struct SCustomClickAction {
    /// Custom action identifier.
    pub id: Identifier,
    /// Raw unnamed network NBT tag bytes when the client supplied a payload (`None` when absent).
    pub payload: Option<Vec<u8>>,
}

impl ReadFrom for SCustomClickAction {
    fn read(data: &mut Cursor<&[u8]>) -> Result<Self> {
        let id = Identifier::read(data)?;
        let framed_len_raw = VarInt::read(data)?.0;
        let framed_len = usize::try_from(framed_len_raw).map_err(|_| {
            Error::new(
                ErrorKind::InvalidData,
                "negative custom click payload length",
            )
        })?;
        if framed_len == 0 || framed_len > MAX_CUSTOM_CLICK_PAYLOAD_LENGTH {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("custom click payload length {framed_len} out of 1..=65536 range"),
            ));
        }
        let mut framed = vec![0_u8; framed_len];
        data.read_exact(&mut framed)?;
        let has_tag = match framed[0] {
            0 => false,
            1 => true,
            other => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("invalid optional NBT flag {other}"),
                ));
            }
        };
        let payload = if has_tag {
            if framed.len() < 2 {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "missing NBT tag id in custom click payload",
                ));
            }
            Some(framed[1..].to_vec())
        } else {
            if framed.len() != 1 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "unexpected trailing bytes after empty custom click payload",
                ));
            }
            None
        };
        Ok(Self { id, payload })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use steel_utils::serial::WriteTo as _;

    #[test]
    fn decodes_absent_and_present_length_prefixed_optional_tag() {
        // Absent payload: length = 1, [0x00]
        let mut empty_wire = Vec::new();
        "minecraft:accept".to_owned().write(&mut empty_wire).unwrap();
        empty_wire.extend_from_slice(&[1, 0]);
        let empty = SCustomClickAction::read(&mut Cursor::new(&empty_wire)).unwrap();
        assert_eq!(empty.id, Identifier::vanilla_static("accept"));
        assert_eq!(empty.payload, None);

        // Present empty compound payload: length = 3, [0x01, 0x0A, 0x00]
        let mut nbt_wire = Vec::new();
        "minecraft:submit".to_owned().write(&mut nbt_wire).unwrap();
        nbt_wire.extend_from_slice(&[3, 1, 0x0A, 0x00]);
        let with_nbt = SCustomClickAction::read(&mut Cursor::new(&nbt_wire)).unwrap();
        assert_eq!(with_nbt.payload, Some(vec![0x0A, 0x00]));
    }
}
