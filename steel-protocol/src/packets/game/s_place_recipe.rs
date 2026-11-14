//! Place recipe packet.

use steel_macros::{ReadFrom, ServerPacket};
use steel_utils::codec::VarInt;

/// Serverbound packet that fills the crafting grid from a recipe.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SPlaceRecipe {
    /// Container id of the open screen.
    #[read(as = VarInt)]
    pub container_id: i32,
    /// Recipe id.
    #[read(as = VarInt)]
    pub recipe: i32,
    /// Whether the grid is filled with the maximum possible amount.
    pub make_all: bool,
}
