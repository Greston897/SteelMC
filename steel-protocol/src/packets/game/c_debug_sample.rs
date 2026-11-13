//! Debug sample packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_DEBUG_SAMPLE;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;

/// Clientbound packet that streams profiler samples to the client.
#[derive(ClientPacket, Debug, Clone, WriteTo)]
#[packet_id(Play = C_DEBUG_SAMPLE)]
pub struct CDebugSample {
    /// Sample kind ordinal (`0` tick time, `1` chunk task time).
    #[write(as = VarInt)]
    pub kind: i32,
    /// Sample values in nanoseconds.
    pub samples: Vec<i64>,
}

impl WriteTo for CDebugSample {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        VarInt(self.kind).write(writer)?;
        VarInt(self.samples.len() as i32).write(writer)?;
        for sample in &self.samples {
            sample.write(writer)?;
        }
        Ok(())
    }
}
