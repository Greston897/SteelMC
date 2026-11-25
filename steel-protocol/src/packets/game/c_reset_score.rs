//! Scoreboard score reset packet (`ClientboundResetScorePacket`).
//!
//! Vanilla writes `owner: Utf` followed by `writeNullable(objectiveName, writeUtf)`
//! (`None` clears the holder from all objectives).

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_RESET_SCORE;

/// Clientbound packet that removes a score entry (or all scores for `owner` when `None`).
#[derive(ClientPacket, WriteTo, Clone, Debug, PartialEq, Eq)]
#[packet_id(Play = C_RESET_SCORE)]
pub struct CResetScore {
    /// Score holder name.
    pub owner: String,
    /// Objective name, or `None` to clear all objectives for `owner`.
    pub objective_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::CResetScore;
    use steel_utils::serial::WriteTo as _;

    #[test]
    fn encodes_nullable_objective_name() {
        let some_pkt = CResetScore {
            owner: "Alex".to_owned(),
            objective_name: Some("deaths".to_owned()),
        };
        let mut bytes = Vec::new();
        some_pkt.write(&mut bytes).unwrap();
        let mut expected = Vec::new();
        "Alex".to_owned().write(&mut expected).unwrap();
        expected.push(1);
        "deaths".to_owned().write(&mut expected).unwrap();
        assert_eq!(bytes, expected);

        let none_pkt = CResetScore {
            owner: "Alex".to_owned(),
            objective_name: None,
        };
        let mut none_bytes = Vec::new();
        none_pkt.write(&mut none_bytes).unwrap();
        assert_eq!(none_bytes, vec![4, b'A', b'l', b'e', b'x', 0]);
    }
}
