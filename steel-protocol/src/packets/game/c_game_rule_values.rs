//! Game rule values snapshot packet (`ClientboundGameRuleValuesPacket`).
//!
//! Vanilla `STREAM_CODEC`: `ByteBufCodecs.map(ResourceKey.streamCodec(Registries.GAME_RULE),
//! ByteBufCodecs.STRING_UTF8)` — a `VarInt` count followed by `(Identifier, String)` pairs.

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_GAME_RULE_VALUES;
use steel_utils::Identifier;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;

/// Clientbound packet answering `REQUEST_GAME_RULE_VALUES` with every game rule and value.
#[derive(ClientPacket, Debug, Clone, PartialEq, Eq)]
#[packet_id(Play = C_GAME_RULE_VALUES)]
pub struct CGameRuleValues {
    /// `(minecraft:<rule>, value)` entries.
    pub rules: Vec<(Identifier, String)>,
}

impl WriteTo for CGameRuleValues {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        VarInt(self.rules.len() as i32).write(writer)?;
        for (key, value) in &self.rules {
            key.write(writer)?;
            value.write(writer)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::CGameRuleValues;
    use steel_utils::Identifier;
    use steel_utils::serial::WriteTo as _;

    #[test]
    fn encodes_namespaced_game_rule_keys() {
        let pkt = CGameRuleValues {
            rules: vec![(
                Identifier::vanilla_static("keep_inventory"),
                "true".to_owned(),
            )],
        };
        let mut actual = Vec::new();
        pkt.write(&mut actual).unwrap();

        let mut expected = vec![1];
        "minecraft:keep_inventory"
            .to_owned()
            .write(&mut expected)
            .unwrap();
        "true".to_owned().write(&mut expected).unwrap();
        assert_eq!(actual, expected);
    }
}
