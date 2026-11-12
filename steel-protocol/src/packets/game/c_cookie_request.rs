//! Cookie request packet.

use steel_macros::ClientPacket;
use steel_registry::packets::{config, play};
use steel_utils::Identifier;
use steel_utils::serial::WriteTo;

/// Clientbound packet that asks the client for a stored cookie.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Config = config::C_COOKIE_REQUEST, Play = play::C_COOKIE_REQUEST)]
pub struct CCookieRequest {
    /// Cookie key.
    pub key: Identifier,
}
