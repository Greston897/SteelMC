//! Recipe book settings packet.

use steel_macros::{ReadFrom, ServerPacket};
use steel_utils::codec::VarInt;

/// Serverbound packet that stores the recipe book open/filter state.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SRecipeBookChangeSettings {
    /// Recipe book type ordinal.
    #[read(as = VarInt)]
    pub book_type: i32,
    /// Whether the book is open.
    pub open: bool,
    /// Whether only craftable recipes are shown.
    pub filtering: bool,
}
