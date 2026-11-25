//! Debug subscription request packet.

use std::io::{Cursor, Result};

use steel_macros::ServerPacket;
use steel_utils::codec::VarInt;
use steel_utils::serial::ReadFrom;

/// Serverbound packet that (un)subscribes to debug data streams.
///
/// Vanilla `ServerboundDebugSubscriptionRequestPacket(Set<DebugSubscription<?>>)`:
/// the wire form is a VarInt-prefixed list of `minecraft:debug_subscription`
/// registry ids. It is not a `(kind, subscribe)` pair — that shape would desync
/// the packet stream because vanilla sends a length prefix first.
#[derive(ServerPacket, Clone, Debug)]
pub struct SDebugSubscriptionRequest {
    /// Registry ids of the subscriptions the client now wants.
    pub subscriptions: Vec<i32>,
}

impl ReadFrom for SDebugSubscriptionRequest {
    fn read(data: &mut Cursor<&[u8]>) -> Result<Self> {
        // `ByteBufCodecs.collection(ReferenceOpenHashSet::new)` — a VarInt
        // element count, then every `ByteBufCodecs.registry` id.
        let len = VarInt::read(data)?.0.max(0) as usize;
        let mut subscriptions = Vec::with_capacity(len);
        for _ in 0..len {
            subscriptions.push(VarInt::read(data)?.0);
        }
        Ok(Self { subscriptions })
    }
}
