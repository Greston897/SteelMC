//! Debug event packet (`ClientboundDebugEventPacket`).
//!
//! Vanilla `STREAM_CODEC`: `DebugSubscription.Event.STREAM_CODEC` — writes
//! `VarInt(subscription_registry_id)` followed directly by the encoded event value
//! (non-optional, unlike `DebugSubscription.Update`).

use std::io::Write;

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_DEBUG_EVENT;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;

/// Clientbound packet broadcasting a one-shot debug-subscription event.
#[derive(ClientPacket, Clone, Debug, PartialEq, Eq)]
#[packet_id(Play = C_DEBUG_EVENT)]
pub struct CDebugEvent {
    /// Registry id in `minecraft:debug_subscription`.
    pub subscription_id: i32,
    /// Pre-encoded event value for `subscription_id`.
    pub encoded_value: Vec<u8>,
}

impl WriteTo for CDebugEvent {
    fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        VarInt(self.subscription_id).write(writer)?;
        writer.write_all(&self.encoded_value)
    }
}

#[cfg(test)]
mod tests {
    use super::CDebugEvent;
    use steel_utils::serial::WriteTo as _;

    #[test]
    fn encodes_subscription_id_and_direct_event_payload() {
        let pkt = CDebugEvent {
            subscription_id: 1,
            encoded_value: vec![0xDE, 0xAD],
        };
        let mut bytes = Vec::new();
        pkt.write(&mut bytes).unwrap();
        assert_eq!(bytes, vec![1, 0xDE, 0xAD]);
    }
}
