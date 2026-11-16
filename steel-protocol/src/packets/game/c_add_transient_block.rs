//! Transient block placement packet (`ClientboundAddTransientBlockPacket`).
//!
//! Vanilla 26.3 wire format (`StreamCodec.composite`):
//! 1. `pos`: `BlockPos` (`i64`)
//! 2. `blockState`: `Block.BLOCK_STATE_REGISTRY` (`VarInt` block state id)
//!
//! Sent when a `FallingBlockEntity` lands and turns back into a solid block so
//! the client avoids a 1-tick gap between entity removal and block placement.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_ADD_TRANSIENT_BLOCK;
use steel_utils::{BlockPos, BlockStateId};

/// Clientbound packet that places a transient landing block at `pos`.
#[derive(ClientPacket, WriteTo, Clone, Copy, Debug, PartialEq, Eq)]
#[packet_id(Play = C_ADD_TRANSIENT_BLOCK)]
pub struct CAddTransientBlock {
    /// Block position of the newly placed block.
    pub pos: BlockPos,
    /// Placed block state id (`VarInt`).
    pub block_state: BlockStateId,
}

#[cfg(test)]
mod tests {
    use super::CAddTransientBlock;
    use steel_utils::codec::VarInt;
    use steel_utils::serial::WriteTo as _;
    use steel_utils::{BlockPos, BlockStateId};

    #[test]
    fn encodes_block_pos_then_varint_state() {
        let pos = BlockPos::new(12, 65, -4);
        let pkt = CAddTransientBlock {
            pos,
            block_state: BlockStateId(300),
        };
        let mut actual = Vec::new();
        pkt.write(&mut actual).unwrap();

        let mut expected = Vec::new();
        pos.write(&mut expected).unwrap();
        VarInt(300).write(&mut expected).unwrap();
        assert_eq!(actual, expected);
    }
}
