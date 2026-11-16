//! Debug chunk value update packet (`ClientboundDebugChunkValuePacket`).
//!
//! Vanilla `STREAM_CODEC`: `(ChunkPos.STREAM_CODEC, DebugSubscription.Update.STREAM_CODEC)`
//! — packed `i64` chunk pos (`ChunkPos.pack(x, z)`) followed by `DebugSubscriptionUpdate`.

use std::io::Write;

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_DEBUG_CHUNK_VALUE;
use steel_utils::serial::WriteTo;

use super::c_chunks_biomes::pack_chunk_pos;
use super::c_debug_block_value::DebugSubscriptionUpdate;

/// Clientbound packet updating a chunk's debug-subscription value.
#[derive(ClientPacket, Clone, Debug, PartialEq, Eq)]
#[packet_id(Play = C_DEBUG_CHUNK_VALUE)]
pub struct CDebugChunkValue {
    /// Chunk X coordinate.
    pub chunk_x: i32,
    /// Chunk Z coordinate.
    pub chunk_z: i32,
    /// Subscription update.
    pub update: DebugSubscriptionUpdate,
}

impl WriteTo for CDebugChunkValue {
    fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        pack_chunk_pos(self.chunk_x, self.chunk_z).write(writer)?;
        self.update.write(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_packed_chunk_pos_then_subscription_update() {
        let pkt = CDebugChunkValue {
            chunk_x: -3,
            chunk_z: 7,
            update: DebugSubscriptionUpdate {
                subscription_id: 2,
                encoded_value: Some(vec![0x42]),
            },
        };
        let mut bytes = Vec::new();
        pkt.write(&mut bytes).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&pack_chunk_pos(-3, 7).to_be_bytes());
        expected.extend_from_slice(&[2, 1, 0x42]);
        assert_eq!(bytes, expected);
    }
}
