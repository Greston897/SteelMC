//! Combat enter packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_utils::serial::WriteTo as _;
use steel_registry::packets::play::C_PLAYER_COMBAT_ENTER;

/// Clientbound packet that starts the combat cooldown timer.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_PLAYER_COMBAT_ENTER)]
pub struct CPlayerCombatEnter {}
