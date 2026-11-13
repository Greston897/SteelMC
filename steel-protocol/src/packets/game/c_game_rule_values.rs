//! Game rule values packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_utils::serial::WriteTo as _;
use steel_registry::packets::play::C_GAME_RULE_VALUES;

/// Clientbound packet that sends the current value of every game rule.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_GAME_RULE_VALUES)]
pub struct CGameRuleValues {
    /// Rule name to value pairs.
    #[write(as = Prefixed(VarInt))]
    pub rules: Vec<(String, String)>,
}
