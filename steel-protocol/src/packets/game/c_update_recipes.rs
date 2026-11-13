//! Update recipes packet.

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_UPDATE_RECIPES;
use steel_utils::codec::VarInt;
use steel_utils::serial::PrefixedWrite as _;
use steel_utils::serial::WriteTo;

/// One recipe entry: its id and the network form of the recipe body.
#[derive(Debug, Clone)]
pub struct RecipeEntry {
    /// Recipe registry id.
    pub id: i32,
    /// Serialized recipe body (the vanilla `RecipeSerializer` payload).
    pub data: Vec<u8>,
}

/// Clientbound packet that replaces the client recipe book contents.
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Play = C_UPDATE_RECIPES)]
pub struct CUpdateRecipes {
    /// All recipes known to the server.
    pub recipes: Vec<RecipeEntry>,
    /// Stonecutter recipe ids shown in the stonecutter screen.
    pub stonecutter_recipes: Vec<i32>,
}

impl WriteTo for CUpdateRecipes {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        VarInt(self.recipes.len() as i32).write(writer)?;
        for recipe in &self.recipes {
            VarInt(recipe.id).write(writer)?;
            recipe.data.write_prefixed::<VarInt>(writer)?;
        }
        self.stonecutter_recipes.write_prefixed::<VarInt>(writer)
    }
}
