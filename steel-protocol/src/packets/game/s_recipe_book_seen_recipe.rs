//! Recipe book seen recipe packet.

use steel_macros::{ReadFrom, ServerPacket};
use steel_utils::Identifier;

/// Serverbound packet that marks one recipe as seen.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SRecipeBookSeenRecipe {
    /// Recipe id.
    pub recipe: Identifier,
}
