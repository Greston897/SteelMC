//! Mount screen packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_MOUNT_SCREEN_OPEN;
use steel_utils::codec::VarInt;

/// Clientbound packet that opens the inventory of a ridden entity
/// (horse, llama, camel, pig with saddle, …).
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_MOUNT_SCREEN_OPEN)]
pub struct CMountScreenOpen {
    /// Container id the client uses for the screen.
    #[write(as = VarInt)]
    pub container_id: i32,
    /// Size of the entity inventory in slots.
    #[write(as = VarInt)]
    pub inventory_columns: i32,
    /// Entity id of the mount.
    #[write(as = VarInt)]
    pub entity_id: i32,
}
