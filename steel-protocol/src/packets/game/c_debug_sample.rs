//! Debug sample packet (`ClientboundDebugSamplePacket`).
//!
//! Vanilla `write` order:
//! 1. `output.writeLongArray(this.sample)` — `VarInt(len)` followed by big-endian `i64`s
//! 2. `output.writeEnum(this.debugSampleType)` — `VarInt` (`0 = TICK_TIME`)

use std::io::Write;

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_DEBUG_SAMPLE;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;

/// Clientbound packet streaming profiler samples (e.g. `F3+2` TPS chart).
#[derive(ClientPacket, Clone, Debug, PartialEq, Eq)]
#[packet_id(Play = C_DEBUG_SAMPLE)]
pub struct CDebugSample {
    /// Raw `i64` sample values (written first).
    pub sample: Vec<i64>,
    /// Vanilla `RemoteDebugSampleType` ordinal (`0 = TICK_TIME`, written second).
    pub debug_sample_type: i32,
}

impl WriteTo for CDebugSample {
    fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        VarInt(self.sample.len() as i32).write(writer)?;
        for &value in &self.sample {
            value.write(writer)?;
        }
        VarInt(self.debug_sample_type).write(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::CDebugSample;
    use steel_utils::serial::WriteTo as _;

    #[test]
    fn writes_long_array_before_sample_type() {
        let pkt = CDebugSample {
            sample: vec![50_000_000],
            debug_sample_type: 0,
        };
        let mut bytes = Vec::new();
        pkt.write(&mut bytes).unwrap();
        let mut expected = vec![1];
        expected.extend_from_slice(&50_000_000_i64.to_be_bytes());
        expected.push(0);
        assert_eq!(bytes, expected);
    }
}
