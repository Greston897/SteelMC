//! Code of conduct packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::config::C_CODE_OF_CONDUCT;

/// Clientbound packet that asks the player to accept the code of conduct
/// before the server lets them join.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Config = C_CODE_OF_CONDUCT)]
pub struct CCodeOfConduct {
    /// Hyperlink opening the full code of conduct.
    pub link: Option<String>,
}
