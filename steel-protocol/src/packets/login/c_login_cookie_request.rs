//! Login cookie request packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_utils::serial::WriteTo as _;
use steel_registry::packets::login::C_COOKIE_REQUEST;
use steel_utils::Identifier;

/// Clientbound login-state packet that asks for a stored cookie.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Login = C_COOKIE_REQUEST)]
pub struct CLoginCookieRequest {
    /// Cookie key.
    pub key: Identifier,
}
