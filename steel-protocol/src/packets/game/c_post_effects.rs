//! Post-processing effects packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_utils::serial::WriteTo as _;
use steel_registry::packets::{config, play};
use steel_utils::Identifier;

/// A post-processing shader to enable.
#[derive(Clone, Debug)]
pub struct PostEffect {
    /// Effect id from the `minecraft:post_effect` registry.
    pub id: Identifier,
}

/// Clientbound packet that replaces the active post-processing pipeline.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Config = config::C_POST_EFFECTS, Play = play::C_POST_EFFECTS)]
pub struct CPostEffects {
    /// Effects applied in order.
    #[write(as = Prefixed(VarInt))]
    pub effects: Vec<PostEffect>,
}
