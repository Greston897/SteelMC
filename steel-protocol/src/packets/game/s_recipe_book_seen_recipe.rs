//! Seen recipe notification (`ServerboundRecipeBookSeenRecipePacket`).
//!
//! Vanilla `STREAM_CODEC`: `RecipeDisplayId.STREAM_CODEC` (`ByteBufCodecs.VAR_INT`) —
//! a single `VarInt` display index matching `ClientboundRecipeBookAddPacket`.

use steel_macros::{ReadFrom, ServerPacket};

/// Serverbound packet reporting that the player viewed one newly unlocked recipe.
#[derive(ReadFrom, ServerPacket, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SRecipeBookSeenRecipe {
    /// `RecipeDisplayId` (`VarInt`).
    #[read(as = VarInt)]
    pub recipe: i32,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::SRecipeBookSeenRecipe;
    use steel_utils::serial::ReadFrom as _;

    #[test]
    fn reads_varint_recipe_display_id() {
        let pkt = SRecipeBookSeenRecipe::read(&mut Cursor::new(&[0xAC, 0x02])).unwrap();
        assert_eq!(pkt.recipe, 300);
    }
}
