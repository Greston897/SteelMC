//! Chunk biome array packet.

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_CHUNKS_BIOMES;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;

/// Biomes of one chunk section column.
#[derive(Debug, Clone)]
pub struct ChunkBiomeData {
    /// Chunk x coordinate.
    pub x: i32,
    /// Chunk z coordinate.
    pub z: i32,
    /// Packed biome container for every section of the chunk.
    pub biomes: Vec<u8>,
}

/// Clientbound packet that sends only the biome palette of loaded chunks.
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Play = C_CHUNKS_BIOMES)]
pub struct CChunksBiomes {
    /// Chunks whose biomes changed.
    pub chunks: Vec<ChunkBiomeData>,
}

impl WriteTo for CChunksBiomes {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        VarInt(self.chunks.len() as i32).write(writer)?;
        for chunk in &self.chunks {
            VarInt(chunk.x).write(writer)?;
            VarInt(chunk.z).write(writer)?;
            VarInt(chunk.biomes.len() as i32).write(writer)?;
            chunk.biomes.write(writer)?;
        }
        Ok(())
    }
}
