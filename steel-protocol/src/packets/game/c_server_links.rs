//! Server links packet.

use steel_macros::ClientPacket;
use steel_registry::packets::{config, play};
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;
use text_components::TextComponent;

/// One link shown in the in-game pause menu.
#[derive(Debug, Clone)]
pub struct ServerLink {
    /// Built-in label, `None` when the link uses a custom label.
    pub builtin_type: Option<VarInt>,
    /// Custom label shown when [`ServerLink::builtin_type`] is `None`.
    pub label: TextComponent,
    /// Target URL.
    pub url: String,
}

/// Clientbound packet that advertises the server links.
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Config = config::C_SERVER_LINKS, Play = play::C_SERVER_LINKS)]
pub struct CServerLinks {
    /// Advertised links.
    pub links: Vec<ServerLink>,
}

impl WriteTo for CServerLinks {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        VarInt(self.links.len() as i32).write(writer)?;
        for link in &self.links {
            match link.builtin_type {
                Some(id) => id.write(writer)?,
                None => VarInt(0).write(writer)?,
            }
            link.label.write(writer)?;
            link.url.write(writer)?;
        }
        Ok(())
    }
}
