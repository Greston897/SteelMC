//! Game test position highlight packet (`ClientboundGameTestHighlightPosPacket`).
//!
//! Vanilla `STREAM_CODEC`: `(BlockPos absolutePos, BlockPos relativePos)` — two
//! non-optional 64-bit packed block positions.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_GAME_TEST_HIGHLIGHT_POS;
use steel_utils::BlockPos;

/// Clientbound packet highlighting a block during a game test.
#[derive(ClientPacket, WriteTo, Debug, Clone, Copy, PartialEq, Eq)]
#[packet_id(Play = C_GAME_TEST_HIGHLIGHT_POS)]
pub struct CGameTestHighlightPos {
    /// World-space block coordinate.
    pub absolute_pos: BlockPos,
    /// Structure-relative block coordinate shown in the label.
    pub relative_pos: BlockPos,
}

#[cfg(test)]
mod tests {
    use super::CGameTestHighlightPos;
    use steel_utils::BlockPos;
    use steel_utils::serial::WriteTo as _;

    #[test]
    fn encodes_exact_sixteen_bytes() {
        let pkt = CGameTestHighlightPos {
            absolute_pos: BlockPos::new(100, 64, -20),
            relative_pos: BlockPos::new(1, 2, 3),
        };
        let mut bytes = Vec::new();
        pkt.write(&mut bytes).unwrap();
        assert_eq!(bytes.len(), 16);
    }
}
