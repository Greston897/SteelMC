//! Custom click action packet (dialog buttons).

use steel_macros::{ReadFrom, ServerPacket};
use steel_utils::Identifier;
use steel_utils::codec::VarInt;
use steel_utils::serial::{PrefixedRead as _, ReadFrom};

/// Serverbound packet sent when the player clicks a custom dialog action.
#[derive(ServerPacket, Clone, Debug)]
pub struct SCustomClickAction {
    /// Click action id.
    pub id: Identifier,
}

impl ReadFrom for SCustomClickAction {
    fn read(
        data: &mut std::io::Cursor<&[u8]>,
    ) -> std::io::Result<Self> {
        let id = Identifier::read(data)?;
        // Vanilla sends an optional payload compound after the id.
        let has_payload = bool::read(data)?;
        if has_payload {
            let _payload: Vec<u8> = <Vec<u8>>::read_prefixed::<VarInt>(data)?;
        }
        Ok(Self { id })
    }
}
