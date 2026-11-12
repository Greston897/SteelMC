//! Store cookie packet.

use steel_macros::ClientPacket;
use steel_registry::packets::{config, play};
use steel_utils::Identifier;
use steel_utils::serial::WriteTo;

/// Clientbound packet that asks the client to persist a cookie.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Config = config::C_STORE_COOKIE, Play = play::C_STORE_COOKIE)]
pub struct CStoreCookie {
    /// Cookie key.
    pub key: Identifier,
    /// Opaque payload (at most 5 KiB in vanilla).
    pub payload: Vec<u8>,
}
