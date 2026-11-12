//! Login custom query answer packet.

use std::io::Cursor;

use steel_macros::ServerPacket;
use steel_utils::codec::VarInt;
use steel_utils::serial::{PrefixedRead as _, ReadFrom};

/// Serverbound answer to `CCustomQuery`.
#[derive(ServerPacket, Debug, Clone)]
pub struct SCustomQueryAnswer {
    /// Transaction id of the answered query.
    pub transaction_id: i32,
    /// Answer payload, `None` when the client declined the query.
    pub data: Option<Vec<u8>>,
}

impl ReadFrom for SCustomQueryAnswer {
    fn read(data: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let transaction_id = VarInt::read(data)?.0;
        let has_data = bool::read(data)?;
        let data = if has_data {
            Some(Vec::<u8>::read_prefixed::<VarInt>(data)?)
        } else {
            None
        };
        Ok(Self {
            transaction_id,
            data,
        })
    }
}
