use steel_macros::{ReadFrom, ServerPacket};

/// Serverbound packet sent by the client while riding a boat.
///
/// Mirrors vanilla `BoatPaddleStateC2SPacket`: two booleans, left paddle first.
/// The server copies the flags into the boat's synchronized data, which drives
/// the paddle animation and sounds.
#[derive(ReadFrom, ServerPacket, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SPaddleBoat {
    /// Whether the left paddle is moving.
    pub left_paddling: bool,
    /// Whether the right paddle is moving.
    pub right_paddling: bool,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::packet_traits::ServerPacket;

    use super::SPaddleBoat;

    #[test]
    fn flags_decode_in_wire_order() {
        let bytes = [1_u8, 0_u8];
        let packet = SPaddleBoat::read_packet(&mut Cursor::new(&bytes[..]))
            .expect("paddle packet should decode");
        assert!(packet.left_paddling);
        assert!(!packet.right_paddling);
    }
}
