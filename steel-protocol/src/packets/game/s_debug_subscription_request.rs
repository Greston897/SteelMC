//! Debug subscription request packet.

use steel_macros::{ReadFrom, ServerPacket};
use steel_utils::codec::VarInt;

/// Serverbound packet that (un)subscribes to a debug data stream.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SDebugSubscriptionRequest {
    /// Subscription kind ordinal.
    #[read(as = VarInt)]
    pub kind: i32,
    /// Whether the subscription is enabled.
    pub subscribe: bool,
}
