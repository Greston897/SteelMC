//! Mount screen packet (`ClientboundMountScreenOpenPacket`).
//!
//! Vanilla writes `writeContainerId(containerId)` (`VarInt`),
//! `writeVarInt(inventoryColumns)`, then `writeInt(entityId)` (fixed 4-byte `i32`).

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_MOUNT_SCREEN_OPEN;

/// Clientbound packet that opens the inventory of a ridden entity
/// (horse, donkey, mule, llama, camel, nautilus, …).
#[derive(ClientPacket, WriteTo, Clone, Debug, PartialEq, Eq)]
#[packet_id(Play = C_MOUNT_SCREEN_OPEN)]
pub struct CMountScreenOpen {
    /// Container id the client uses for the screen (`VarInt`).
    #[write(as = VarInt)]
    pub container_id: i32,
    /// Number of inventory columns (`VarInt`).
    #[write(as = VarInt)]
    pub inventory_columns: i32,
    /// Entity id of the mount (big-endian `i32`, NOT `VarInt`).
    pub entity_id: i32,
}

#[cfg(test)]
mod tests {
    use super::CMountScreenOpen;
    use steel_utils::serial::WriteTo as _;

    #[test]
    fn encodes_entity_id_as_big_endian_i32() {
        let pkt = CMountScreenOpen {
            container_id: 2,
            inventory_columns: 5,
            entity_id: 0x0102_0304,
        };
        let mut bytes = Vec::new();
        pkt.write(&mut bytes).unwrap();
        assert_eq!(bytes, vec![2, 5, 0x01, 0x02, 0x03, 0x04]);
    }
}
