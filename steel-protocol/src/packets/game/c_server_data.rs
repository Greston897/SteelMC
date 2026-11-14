//! Server data packet.

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_SERVER_DATA;
use steel_utils::serial::WriteTo;
use text_components::TextComponent;

/// Clientbound packet that carries the MOTD and the server icon.
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Play = C_SERVER_DATA)]
pub struct CServerData {
    /// Server MOTD shown in the F3 screen and the tab list header.
    pub motd: TextComponent,
    /// Base64 PNG icon bytes, or `None` when the server has no icon.
    pub icon: Option<Vec<u8>>,
}

impl WriteTo for CServerData {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        self.motd.write(writer)?;
        match &self.icon {
            Some(icon) => {
                true.write(writer)?;
                icon.write(writer)
            }
            None => false.write(writer),
        }
    }
}
