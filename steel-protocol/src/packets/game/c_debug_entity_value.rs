//! Debug entity value update packet (`ClientboundDebugEntityValuePacket`).
//!
//! Vanilla `STREAM_CODEC`: `(ByteBufCodecs.VAR_INT entityId, DebugSubscription.Update.STREAM_CODEC)`.

use std::io::Write;

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_DEBUG_ENTITY_VALUE;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;

use super::c_debug_block_value::DebugSubscriptionUpdate;

/// Clientbound packet updating an entity's debug-subscription value.
#[derive(ClientPacket, Clone, Debug, PartialEq, Eq)]
#[packet_id(Play = C_DEBUG_ENTITY_VALUE)]
pub struct CDebugEntityValue {
    /// Target entity id (`VarInt`).
    pub entity_id: i32,
    /// Subscription update.
    pub update: DebugSubscriptionUpdate,
}

impl WriteTo for CDebugEntityValue {
    fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        VarInt(self.entity_id).write(writer)?;
        self.update.write(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_varint_entity_id_then_update() {
        let pkt = CDebugEntityValue {
            entity_id: 42,
            update: DebugSubscriptionUpdate {
                subscription_id: 5,
                encoded_value: Some(vec![1, 2]),
            },
        };
        let mut bytes = Vec::new();
        pkt.write(&mut bytes).unwrap();
        assert_eq!(bytes, vec![42, 5, 1, 1, 2]);
    }
}
