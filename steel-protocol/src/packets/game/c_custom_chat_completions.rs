//! Custom chat completions packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_utils::serial::WriteTo as _;
use steel_registry::packets::play::C_CUSTOM_CHAT_COMPLETIONS;
use steel_utils::codec::VarInt;

/// One chat completion suggestion sent by the server.
#[derive(WriteTo, Clone, Debug)]
pub struct ChatCompletionAction {
    /// Text inserted into the chat field when the entry is clicked.
    pub text: String,
    /// Tooltip shown for the entry.
    pub tooltip: Option<text_components::TextComponent>,
}

/// Clientbound packet that adds server-side chat completion entries.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_CUSTOM_CHAT_COMPLETIONS)]
pub struct CCustomChatCompletions {
    /// Action id: `0` add, `1` remove, `2` set.
    #[write(as = VarInt)]
    pub action: i32,
    /// Entries carried by the action.
    #[write(as = Prefixed(VarInt))]
    pub entries: Vec<ChatCompletionAction>,
}
