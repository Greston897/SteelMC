//! Chunk biomes packet (`ClientboundChunksBiomesPacket`).
//!
//! Vanilla `ChunkBiomeData.write`:
//! - `output.writeChunkPos(this.pos)` — big-endian `i64` packed as
//!   `ChunkPos.pack(x, z) = (x as u32 as i64) | ((z as u32 as i64) << 32)`
//! - `output.writeByteArray(this.buffer)` — `VarInt(len)` followed by `buffer` bytes

use std::io::Write;

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_CHUNKS_BIOMES;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo as WriteToTrait;

/// Packs chunk coordinates identically to vanilla `ChunkPos.pack(int x, int z)`.
#[must_use]
pub const fn pack_chunk_pos(x: i32, z: i32) -> i64 {
    ((x as u32) as i64) | (((z as u32) as i64) << 32)
}

/// Biome data for one chunk (`ClientboundChunksBiomesPacket.ChunkBiomeData`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkBiomeData {
    /// Chunk X coordinate.
    pub x: i32,
    /// Chunk Z coordinate.
    pub z: i32,
    /// Concatenated serialized `PalettedContainer<Holder<Biome>>` sections (max 2 MiB in vanilla).
    pub biomes: Vec<u8>,
}

impl WriteToTrait for ChunkBiomeData {
    fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        pack_chunk_pos(self.x, self.z).write(writer)?;
        VarInt(self.biomes.len() as i32).write(writer)?;
        writer.write_all(&self.biomes)
    }
}

/// Clientbound packet that updates biome palettes across one or more loaded chunks.
#[derive(ClientPacket, WriteTo, Debug, Clone, PartialEq, Eq)]
#[packet_id(Play = C_CHUNKS_BIOMES)]
pub struct CChunksBiomes {
    /// Updated chunks.
    pub chunks: Vec<ChunkBiomeData>,
}

#[cfg(test)]
mod tests {
    use super::{CChunksBiomes, ChunkBiomeData, pack_chunk_pos};
    use steel_utils::serial::WriteTo as _;

    #[test]
    fn encodes_packed_chunk_pos_and_single_byte_array_prefix() {
        let pkt = CChunksBiomes {
            chunks: vec![ChunkBiomeData {
                x: -1,
                z: 2,
                biomes: vec![0xAA, 0xBB, 0xCC],
            }],
        };
        let mut bytes = Vec::new();
        pkt.write(&mut bytes).unwrap();

        let packed = pack_chunk_pos(-1, 2);
        assert_eq!(packed as i32, -1);
        assert_eq!((packed >> 32) as i32, 2);

        let mut expected = vec![1]; // 1 chunk
        expected.extend_from_slice(&packed.to_be_bytes());
        expected.push(3); // single VarInt length prefix
        expected.extend_from_slice(&[0xAA, 0xBB, 0xCC]);
        assert_eq!(bytes, expected);
    }
}
