//! Show dialog packet.

use steel_macros::ClientPacket;
use steel_registry::packets::{config, play};
use steel_utils::Identifier;
use steel_utils::serial::WriteTo;

/// Where the dialog is anchored when shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogAnchor {
    /// Server-defined screen (`0`).
    Server = 0,
    /// Shown above the player's hotbar (`1`).
    Hanging = 1,
}

/// Clientbound packet that opens a dialog shipped by the server.
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Config = config::C_SHOW_DIALOG, Play = play::C_SHOW_DIALOG)]
pub struct CShowDialog {
    /// The dialog registry entry to show.
    pub dialog: Identifier,
    /// Anchor of the dialog.
    pub anchor: DialogAnchor,
}

impl WriteTo for CShowDialog {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        self.dialog.write(writer)?;
        match self.anchor {
            DialogAnchor::Server => false.write(writer),
            DialogAnchor::Hanging => true.write(writer),
        }
    }
}
