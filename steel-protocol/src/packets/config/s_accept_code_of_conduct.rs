//! Serverbound accept code of conduct packet (`ServerboundAcceptCodeOfConductPacket`).
//!
//! Vanilla uses `StreamCodec.unit(INSTANCE)` — zero payload bytes.

use steel_macros::{ReadFrom, ServerPacket};

/// Sent by the client in the configuration state after accepting `CCodeOfConduct`.
#[derive(ReadFrom, ServerPacket, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SAcceptCodeOfConduct;

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::SAcceptCodeOfConduct;
    use steel_utils::serial::ReadFrom as _;

    #[test]
    fn reads_empty_unit_payload() {
        let mut cursor = Cursor::new(&[][..]);
        let pkt = SAcceptCodeOfConduct::read(&mut cursor).unwrap();
        assert_eq!(pkt, SAcceptCodeOfConduct);
        assert_eq!(cursor.position(), 0);
    }
}
