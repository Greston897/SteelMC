//! Place ghost recipe packet.

use std::io::{Result, Write};

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_PLACE_GHOST_RECIPE;
use steel_registry::slot_display::RecipeDisplay;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;

/// Clientbound packet that previews a recipe in the crafting grid.
///
/// Vanilla `ClientboundPlaceGhostRecipePacket(int containerId, RecipeDisplay
/// recipeDisplay)`. The display is sent in full — it is *not* a recipe-display
/// id — so the client can render the ghost without a second lookup.
#[derive(ClientPacket, Clone, Debug)]
#[packet_id(Play = C_PLACE_GHOST_RECIPE)]
pub struct CPlaceGhostRecipe {
    /// Container id of the open crafting screen.
    pub container_id: i32,
    /// The recipe display to preview.
    pub display: RecipeDisplay,
}

impl WriteTo for CPlaceGhostRecipe {
    fn write(&self, writer: &mut impl Write) -> Result<()> {
        // `ByteBufCodecs.CONTAINER_ID` is a VarInt.
        VarInt(self.container_id).write(writer)?;
        // `RecipeDisplay.STREAM_CODEC`: the registry id of the display type,
        // then the type's own body.
        self.display.write(writer)
    }
}
