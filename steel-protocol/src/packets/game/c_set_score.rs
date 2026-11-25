//! Scoreboard score packet (`ClientboundSetScorePacket`).
//!
//! Vanilla sends one score per packet (`StreamCodec.composite`):
//! 1. `owner`: UTF-8 string
//! 2. `objectiveName`: UTF-8 string
//! 3. `score`: `VarInt`
//! 4. `display`: `Optional<Component>`
//! 5. `numberFormat`: `Optional<NumberFormat>`

use std::io::Write;

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_SET_SCORE;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;
use text_components::TextComponent;

/// Clientbound packet that sets one score on one objective.
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Play = C_SET_SCORE)]
pub struct CSetScore {
    /// Score holder name.
    pub owner: String,
    /// Objective name.
    pub objective_name: String,
    /// Current score value (`VarInt`).
    pub score: i32,
    /// Optional custom display name for the holder.
    pub display: Option<TextComponent>,
    /// Optional fixed-component number format (`NumberFormatTypes` ordinal `2 = fixed`;
    /// `None` writes `false` for `NumberFormatTypes.OPTIONAL_STREAM_CODEC`).
    pub fixed_format: Option<TextComponent>,
}

impl CSetScore {
    /// Creates a plain integer score update with default formatting.
    #[must_use]
    pub const fn new(owner: String, objective_name: String, score: i32) -> Self {
        Self {
            owner,
            objective_name,
            score,
            display: None,
            fixed_format: None,
        }
    }
}

impl WriteTo for CSetScore {
    fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.owner.write(writer)?;
        self.objective_name.write(writer)?;
        VarInt(self.score).write(writer)?;
        self.display.write(writer)?;
        match &self.fixed_format {
            Some(text) => {
                true.write(writer)?;
                // `NumberFormatTypes.bootstrap`: 0 = blank, 1 = styled, 2 = fixed.
                VarInt(2).write(writer)?;
                text.write(writer)
            }
            None => false.write(writer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CSetScore;
    use steel_utils::codec::VarInt;
    use steel_utils::serial::WriteTo as _;

    #[test]
    fn encodes_single_score_with_optional_trailers() {
        let pkt = CSetScore::new("Steve".to_owned(), "kills".to_owned(), 42);
        let mut actual = Vec::new();
        pkt.write(&mut actual).unwrap();

        let mut expected = Vec::new();
        "Steve".to_owned().write(&mut expected).unwrap();
        "kills".to_owned().write(&mut expected).unwrap();
        VarInt(42).write(&mut expected).unwrap();
        expected.push(0); // display = None
        expected.push(0); // numberFormat = None
        assert_eq!(actual, expected);
    }
}
