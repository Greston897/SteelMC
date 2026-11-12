//! Place ghost recipe packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_PLACE_GHOST_RECIPE;
use steel_utils::codec::VarInt;

/// Clientbound packet that previews a recipe in the crafting grid.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_PLACE_GHOST_RECIPE)]
pub struct CPlaceGhostRecipe {
    /// Container id of the open crafting screen.
    #[write(as = VarInt)]
    pub container_id: i32,
    /// Recipe id to preview.
    #[write(as = VarInt)]
    pub recipe: i32,
}
