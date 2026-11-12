//! Cookie response packet (login and configuration states).

use std::io::Cursor;

use steel_macros::ServerPacket;
use steel_utils::Identifier;
use steel_utils::codec::VarInt;
use steel_utils::serial::{PrefixedRead as _, ReadFrom};

/// Serverbound answer to a cookie request.
#[derive(ServerPacket, Debug, Clone)]
pub struct SCookieResponse {
    /// Cookie key.
    pub key: Identifier,
    /// Stored payload, `None` when the client has no such cookie.
    pub payload: Option<Vec<u8>>,
}

impl ReadFrom for SCookieResponse {
    fn read(data: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let key = Identifier::read(data)?;
        let has_payload = bool::read(data)?;
        let payload = if has_payload {
            Some(Vec::<u8>::read_prefixed::<VarInt>(data)?)
        } else {
            None
        };
        Ok(Self { key, payload })
    }
}
