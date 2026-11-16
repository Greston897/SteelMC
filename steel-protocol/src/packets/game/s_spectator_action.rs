//! Spectator camera action (`ServerboundSpectatorActionPacket`).
//!
//! Vanilla 26.3 wire format (`ByteBufCodecs.OPTIONAL_VAR_INT`):
//! - `VarInt(0)` → `None` (`OptionalInt.empty()`, e.g. stopping spectate)
//! - `VarInt(n)` → `Some(n.wrapping_sub(1))` (target entity id `n - 1`)

use std::io::{Cursor, Result};

use steel_macros::ServerPacket;
use steel_utils::codec::VarInt;
use steel_utils::serial::ReadFrom;

/// Serverbound packet requesting spectator camera attachment or detachment.
#[derive(ServerPacket, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SSpectatorAction {
    /// Target entity id, or `None` when encoded as `0` (`OptionalInt.empty()`).
    pub spectate_entity_id: Option<i32>,
}

impl ReadFrom for SSpectatorAction {
    fn read(data: &mut Cursor<&[u8]>) -> Result<Self> {
        let raw = VarInt::read(data)?.0;
        let spectate_entity_id = if raw == 0 {
            None
        } else {
            Some(raw.wrapping_sub(1))
        };
        Ok(Self { spectate_entity_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_absent_entity_id_from_zero_varint() {
        let mut data = Cursor::new([0].as_slice());
        let packet = SSpectatorAction::read(&mut data).expect("packet should parse");

        assert_eq!(packet.spectate_entity_id, None);
    }

    #[test]
    fn reads_present_entity_id_as_optional_varint_minus_one() {
        // Vanilla encodes entity id 62 as VarInt(63), and entity id 0 as VarInt(1).
        let mut data = Cursor::new([63].as_slice());
        let packet = SSpectatorAction::read(&mut data).expect("packet should parse");
        assert_eq!(packet.spectate_entity_id, Some(62));

        let mut zero_id = Cursor::new([1].as_slice());
        let packet_zero = SSpectatorAction::read(&mut zero_id).expect("packet should parse");
        assert_eq!(packet_zero.spectate_entity_id, Some(0));
    }
}
