//! Clientbound packet that plays a sound bound to an entity.

use steel_macros::{ClientPacket, WriteTo};
use steel_utils::serial::WriteTo as _;
use steel_registry::packets::play::C_SOUND_ENTITY;
use steel_utils::codec::VarInt;

/// Clientbound packet that plays a sound at an entity's position.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_SOUND_ENTITY)]
pub struct CSoundEntity {
    /// Sound registry ID.
    #[write(as = VarInt)]
    pub sound_id: i32,
    /// Vanilla `SoundSource` ordinal.
    #[write(as = VarInt)]
    pub source: i32,
    /// Emitting entity ID.
    #[write(as = VarInt)]
    pub entity_id: i32,
    /// Volume multiplier.
    pub volume: f32,
    /// Pitch multiplier.
    pub pitch: f32,
    /// Random seed used by the client to pick a variant.
    pub seed: i64,
}
