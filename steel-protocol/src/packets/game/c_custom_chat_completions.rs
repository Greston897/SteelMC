//! Custom chat completions packet (`ClientboundCustomChatCompletionsPacket`).
//!
//! Vanilla `write`: `output.writeEnum(this.action)` (`VarInt`: `0 = ADD`, `1 = REMOVE`, `2 = SET`)
//! followed by `output.writeCollection(this.entries, FriendlyByteBuf::writeUtf)`.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_CUSTOM_CHAT_COMPLETIONS;

/// Action applied to the client's custom tab-completion list.
#[derive(WriteTo, Clone, Copy, Debug, PartialEq, Eq)]
#[write(as = VarInt)]
pub enum CustomChatCompletionAction {
    /// Append `entries` to the current suggestions.
    Add = 0,
    /// Remove `entries` from the current suggestions.
    Remove = 1,
    /// Replace all custom suggestions with `entries`.
    Set = 2,
}

/// Clientbound packet adding, removing, or replacing custom chat completions.
#[derive(ClientPacket, WriteTo, Clone, Debug, PartialEq, Eq)]
#[packet_id(Play = C_CUSTOM_CHAT_COMPLETIONS)]
pub struct CCustomChatCompletions {
    /// Mutation action (`VarInt` `0..=2`).
    pub action: CustomChatCompletionAction,
    /// Completion strings (`Vec<String>`).
    pub entries: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::{CCustomChatCompletions, CustomChatCompletionAction};
    use steel_utils::serial::WriteTo as _;

    #[test]
    fn encodes_action_then_string_list() {
        let pkt = CCustomChatCompletions {
            action: CustomChatCompletionAction::Set,
            entries: vec!["foo".to_owned(), "bar".to_owned()],
        };
        let mut bytes = Vec::new();
        pkt.write(&mut bytes).unwrap();
        assert_eq!(bytes, b"\x02\x02\x03foo\x03bar");
    }
}
