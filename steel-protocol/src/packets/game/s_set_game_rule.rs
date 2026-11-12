//! Set game rule packet.

use steel_macros::{ReadFrom, ServerPacket};

/// Serverbound packet that changes one game rule value.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SSetGameRule {
    /// Game rule name.
    pub rule: String,
    /// New value, serialized as a string for every rule type.
    pub value: String,
}
