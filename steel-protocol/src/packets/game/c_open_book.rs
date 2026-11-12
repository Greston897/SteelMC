//! Clientbound packet that opens the written book screen.
//!
//! Mirrors vanilla `OpenWrittenBookS2CPacket` (`play:open_book`): the only
//! payload is the hand holding the book, so the client can resolve the stack.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_OPEN_BOOK;
use steel_utils::types::InteractionHand;

/// Sent when a written book should be opened on the client.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_OPEN_BOOK)]
pub struct COpenBook {
    /// The hand holding the book.
    pub hand: InteractionHand,
}

impl COpenBook {
    /// Creates the packet for the given hand.
    #[must_use]
    pub const fn new(hand: InteractionHand) -> Self {
        Self { hand }
    }
}

#[cfg(test)]
mod tests {
    use steel_utils::codec::VarInt;
    use steel_utils::serial::WriteTo as _;

    use super::{COpenBook, InteractionHand};

    #[test]
    fn open_book_writes_the_hand_as_a_varint() {
        let mut bytes = Vec::new();
        COpenBook::new(InteractionHand::OffHand)
            .write(&mut bytes)
            .expect("the packet should encode");

        let mut expected = Vec::new();
        VarInt(1)
            .write(&mut expected)
            .expect("the hand id should encode");
        assert_eq!(bytes, expected);
    }
}
