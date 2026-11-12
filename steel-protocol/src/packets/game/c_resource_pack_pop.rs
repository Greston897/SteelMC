//! Resource pack pop packet.

use steel_macros::ClientPacket;
use steel_registry::packets::{config, play};
use steel_utils::serial::WriteTo;
use uuid::Uuid;

/// Clientbound packet that removes previously pushed resource packs.
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Config = config::C_RESOURCE_PACK_POP, Play = play::C_RESOURCE_PACK_POP)]
pub struct CResourcePackPop {
    /// `UUID` of the previously pushed pack, or `None` to clear every pack.
    pub id: Option<Uuid>,
}

impl WriteTo for CResourcePackPop {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        match &self.id {
            Some(id) => {
                true.write(writer)?;
                id.write(writer)
            }
            None => false.write(writer),
        }
    }
}
