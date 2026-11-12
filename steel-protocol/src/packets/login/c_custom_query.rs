//! Login custom query packet.

use steel_macros::ClientPacket;
use steel_registry::packets::login::C_CUSTOM_QUERY;
use steel_utils::Identifier;
use steel_utils::codec::VarInt;
use steel_utils::serial::{PrefixedWrite as _, WriteTo};

/// Clientbound login-state packet that asks the client a custom question.
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Login = C_CUSTOM_QUERY)]
pub struct CCustomQuery {
    /// Transaction id echoed in the answer.
    pub transaction_id: i32,
    /// Query channel.
    pub channel: Identifier,
    /// Query payload.
    pub data: Vec<u8>,
}

impl WriteTo for CCustomQuery {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        VarInt(self.transaction_id).write(writer)?;
        self.channel.write(writer)?;
        self.data.write_prefixed::<VarInt>(writer)
    }
}
