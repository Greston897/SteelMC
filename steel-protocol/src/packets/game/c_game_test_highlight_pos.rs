//! Game-test highlight packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_GAME_TEST_HIGHLIGHT_POS;
use steel_utils::BlockPos;

/// Clientbound packet that highlights a position for the game-test framework.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_GAME_TEST_HIGHLIGHT_POS)]
pub struct CGameTestHighlightPos {
    /// First corner, or `None` when the highlight is cleared.
    pub from: Option<BlockPos>,
    /// Second corner.
    pub to: Option<BlockPos>,
}
