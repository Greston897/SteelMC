//! Login custom query answer packet (`ServerboundCustomQueryAnswerPacket`).
//!
//! Vanilla `read`: `readVarInt(transactionId)`, `readBoolean(hasPayload)`, and when
//! `true`, reads all remaining bytes in the packet directly (`<= 1_048_576`, NO
//! inner length prefix).

use std::io::{Cursor, Error, ErrorKind, Read as _, Result};

use steel_macros::ServerPacket;
use steel_utils::codec::VarInt;
use steel_utils::serial::ReadFrom;

use super::c_custom_query::MAX_CUSTOM_QUERY_PAYLOAD_SIZE;

/// Serverbound answer to [`super::CCustomQuery`].
#[derive(ServerPacket, Debug, Clone, PartialEq, Eq)]
pub struct SCustomQueryAnswer {
    /// Transaction id of the answered query.
    pub transaction_id: i32,
    /// Raw answer payload, or `None` when the client does not understand the query.
    pub data: Option<Vec<u8>>,
}

impl ReadFrom for SCustomQueryAnswer {
    fn read(data: &mut Cursor<&[u8]>) -> Result<Self> {
        let transaction_id = VarInt::read(data)?.0;
        let has_data = bool::read(data)?;
        let payload = if has_data {
            let pos = usize::try_from(data.position()).unwrap_or(usize::MAX);
            let remaining = data.get_ref().len().saturating_sub(pos);
            if remaining > MAX_CUSTOM_QUERY_PAYLOAD_SIZE {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Custom query answer payload may not be larger than 1048576 bytes",
                ));
            }
            let mut buf = vec![0_u8; remaining];
            data.read_exact(&mut buf)?;
            Some(buf)
        } else {
            None
        };
        Ok(Self {
            transaction_id,
            data: payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_declined_and_raw_trailing_answers() {
        let declined = SCustomQueryAnswer::read(&mut Cursor::new(&[7, 0])).unwrap();
        assert_eq!(declined.transaction_id, 7);
        assert_eq!(declined.data, None);

        let answered = SCustomQueryAnswer::read(&mut Cursor::new(&[7, 1, 0xAA, 0xBB])).unwrap();
        assert_eq!(answered.transaction_id, 7);
        assert_eq!(answered.data, Some(vec![0xAA, 0xBB]));
    }
}
