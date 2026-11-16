//! Login custom query packet (`ClientboundCustomQueryPacket`).
//!
//! Vanilla `write`: `writeVarInt(transactionId)`, `writeIdentifier(id)`, then
//! raw payload bytes directly to the end of the packet (`MAX_PAYLOAD_SIZE = 1_048_576`,
//! NO inner length prefix).

use std::io::{Error, ErrorKind, Write};

use steel_macros::ClientPacket;
use steel_registry::packets::login::C_CUSTOM_QUERY;
use steel_utils::Identifier;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;

/// Vanilla `ClientboundCustomQueryPacket.MAX_PAYLOAD_SIZE` (1 MiB).
pub const MAX_CUSTOM_QUERY_PAYLOAD_SIZE: usize = 1_048_576;

/// Clientbound login-state packet asking the client a custom login query.
#[derive(ClientPacket, Debug, Clone, PartialEq, Eq)]
#[packet_id(Login = C_CUSTOM_QUERY)]
pub struct CCustomQuery {
    /// Transaction id echoed in [`super::SCustomQueryAnswer`].
    pub transaction_id: i32,
    /// Query channel identifier.
    pub channel: Identifier,
    /// Raw query payload bytes (up to [`MAX_CUSTOM_QUERY_PAYLOAD_SIZE`]).
    pub data: Vec<u8>,
}

impl WriteTo for CCustomQuery {
    fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        if self.data.len() > MAX_CUSTOM_QUERY_PAYLOAD_SIZE {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "custom query payload exceeds 1 MiB",
            ));
        }
        VarInt(self.transaction_id).write(writer)?;
        self.channel.write(writer)?;
        writer.write_all(&self.data)
    }
}

#[cfg(test)]
mod tests {
    use super::CCustomQuery;
    use steel_utils::Identifier;
    use steel_utils::serial::WriteTo as _;

    #[test]
    fn writes_unprefixed_trailing_payload() {
        let pkt = CCustomQuery {
            transaction_id: 9,
            channel: Identifier::vanilla_static("velocity"),
            data: vec![1, 2, 3],
        };
        let mut actual = Vec::new();
        pkt.write(&mut actual).unwrap();

        let mut expected = vec![9];
        "minecraft:velocity".to_owned().write(&mut expected).unwrap();
        expected.extend_from_slice(&[1, 2, 3]);
        assert_eq!(actual, expected);
    }
}
