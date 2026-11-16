//! Debug block value update packet (`ClientboundDebugBlockValuePacket`).
//!
//! Vanilla `STREAM_CODEC`: `(BlockPos, DebugSubscription.Update.STREAM_CODEC)` where
//! `DebugSubscription.Update` writes `VarInt(subscription_registry_id)` followed by
//! `bool` (`has_value`) and the subscription's raw encoded value bytes when present.

use std::io::Write;

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_DEBUG_BLOCK_VALUE;
use steel_utils::BlockPos;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;

/// Vanilla `DebugSubscription.Update<?>` wire payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebugSubscriptionUpdate {
    /// Registry id in `minecraft:debug_subscription`.
    pub subscription_id: i32,
    /// Pre-encoded subscription value (`None` clears the tracked debug value).
    pub encoded_value: Option<Vec<u8>>,
}

impl WriteTo for DebugSubscriptionUpdate {
    fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        VarInt(self.subscription_id).write(writer)?;
        match &self.encoded_value {
            Some(bytes) => {
                true.write(writer)?;
                writer.write_all(bytes)
            }
            None => false.write(writer),
        }
    }
}

/// Clientbound packet updating a block's debug-subscription value.
#[derive(ClientPacket, Clone, Debug, PartialEq, Eq)]
#[packet_id(Play = C_DEBUG_BLOCK_VALUE)]
pub struct CDebugBlockValue {
    /// Block coordinate.
    pub block_pos: BlockPos,
    /// Subscription update.
    pub update: DebugSubscriptionUpdate,
}

impl WriteTo for CDebugBlockValue {
    fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.block_pos.write(writer)?;
        self.update.write(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_block_pos_subscription_id_and_optional_value() {
        let pos = BlockPos::new(4, 64, 8);
        let pkt = CDebugBlockValue {
            block_pos: pos,
            update: DebugSubscriptionUpdate {
                subscription_id: 3,
                encoded_value: None,
            },
        };
        let mut bytes = Vec::new();
        pkt.write(&mut bytes).unwrap();
        assert_eq!(bytes.len(), 8 + 1 + 1);
        assert_eq!(&bytes[8..], &[3, 0]);
    }
}
