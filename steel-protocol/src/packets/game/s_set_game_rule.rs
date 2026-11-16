//! Set game rule packet (`ServerboundSetGameRulePacket`).
//!
//! Vanilla 26.3 wire format (`Entry.STREAM_CODEC.apply(ByteBufCodecs.list())`):
//! a `VarInt`-prefixed list of `(ResourceKey<GameRule> identifier, String value)` entries.

use steel_macros::{ReadFrom, ServerPacket};
use steel_utils::Identifier;
use steel_utils::codec::VarInt;

/// One game rule update requested by an operator (`ServerboundSetGameRulePacket.Entry`).
#[derive(ReadFrom, Clone, Debug, PartialEq, Eq)]
pub struct SetGameRuleEntry {
    /// Game rule registry key (e.g. `minecraft:keep_inventory`).
    pub key: Identifier,
    /// Serialized rule value (`"true"`, `"false"`, or base-10 integer).
    pub value: String,
}

/// Serverbound packet sent by the operator game-rule screen to apply changed rules.
#[derive(ReadFrom, ServerPacket, Clone, Debug, PartialEq, Eq)]
pub struct SSetGameRule {
    /// Updated game rules.
    #[read(as = Prefixed(VarInt))]
    pub entries: Vec<SetGameRuleEntry>,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::{SSetGameRule, SetGameRuleEntry};
    use steel_utils::Identifier;
    use steel_utils::serial::{ReadFrom as _, WriteTo as _};

    #[test]
    fn decodes_list_of_identifier_value_entries() {
        let mut wire = vec![2]; // 2 entries
        "minecraft:keep_inventory"
            .to_owned()
            .write(&mut wire)
            .unwrap();
        "true".to_owned().write(&mut wire).unwrap();
        "minecraft:random_tick_speed"
            .to_owned()
            .write(&mut wire)
            .unwrap();
        "5".to_owned().write(&mut wire).unwrap();

        let decoded = SSetGameRule::read(&mut Cursor::new(&wire)).unwrap();
        assert_eq!(
            decoded.entries,
            vec![
                SetGameRuleEntry {
                    key: Identifier::vanilla_static("keep_inventory"),
                    value: "true".to_owned(),
                },
                SetGameRuleEntry {
                    key: Identifier::vanilla_static("random_tick_speed"),
                    value: "5".to_owned(),
                },
            ]
        );
    }
}
