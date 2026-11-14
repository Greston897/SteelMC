use std::io::{Cursor, Read};

use steel_macros::{ReadFrom, ServerPacket};
use steel_utils::Identifier;

use steel_utils::serial::ReadFrom;

#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SCustomPayload {
    pub identifier: Identifier,
    //#[read(as = "vec")]
    pub payload: Payload,
}

/// Vanilla `ClientboundCustomPayloadPacket.MAX_PAYLOAD_SIZE`.
///
/// The frame limit (`MAX_PACKET_SIZE`) is 2 MiB, so without this a client could
/// pin 2 MiB per in-flight packet; vanilla refuses anything past 1 MiB.
pub const MAX_CUSTOM_PAYLOAD_SIZE: usize = 1_048_576;

#[derive(Clone, Debug)]
pub struct Payload(pub Vec<u8>);

impl ReadFrom for Payload {
    fn read(data: &mut Cursor<&[u8]>) -> Result<Self, std::io::Error> {
        let remaining = data.get_ref().len().saturating_sub(data.position() as usize);
        if remaining > MAX_CUSTOM_PAYLOAD_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("custom payload of {remaining} bytes exceeds {MAX_CUSTOM_PAYLOAD_SIZE}"),
            ));
        }
        let mut buf = Vec::with_capacity(remaining);
        data.read_to_end(&mut buf)?;
        Ok(Self(buf))
    }
}
