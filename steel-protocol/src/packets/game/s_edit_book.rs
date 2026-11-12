//! Serverbound packet carrying edited book pages and an optional title.
//!
//! Mirrors vanilla `BookUpdateC2SPacket` (`play:edit_book`): the packet holds
//! the inventory slot that was edited, the page texts, and — when the player
//! signed the book — the chosen title. Limits come straight from the vanilla
//! codec: 100 pages of at most 1024 characters and a 32 character title.

use std::io::{Cursor, Error, ErrorKind as IoErrorKind, Result};

use steel_macros::ServerPacket;
use steel_utils::codec::VarInt;
use steel_utils::serial::{ReadFrom, prefixed_read::read_utf};

/// Vanilla `BookUpdateC2SPacket` page list limit.
pub const MAX_PAGES: usize = 100;

/// Vanilla per-page character limit (`PacketCodecs.string(1024)`).
pub const MAX_PAGE_LENGTH: usize = 1024;

/// Vanilla title character limit (`PacketCodecs.string(32)`).
pub const MAX_TITLE_LENGTH: usize = 32;

/// The client's edited book contents.
#[derive(ServerPacket, Clone, Debug)]
pub struct SEditBook {
    /// Inventory slot (hotbar index or the offhand slot) that was edited.
    pub slot: i32,
    /// Page texts in order.
    pub pages: Vec<String>,
    /// The title when the player signed the book, otherwise `None`.
    pub title: Option<String>,
}

impl ReadFrom for SEditBook {
    fn read(data: &mut Cursor<&[u8]>) -> Result<Self> {
        let slot = VarInt::read(data)?.0;

        let page_count = VarInt::read(data)?.0;
        if page_count < 0 || page_count as usize > MAX_PAGES {
            return Err(Error::new(
                IoErrorKind::InvalidData,
                format!("book page count {page_count} exceeds {MAX_PAGES}"),
            ));
        }
        let mut pages = Vec::with_capacity(page_count as usize);
        for _ in 0..page_count {
            pages.push(read_utf(data, MAX_PAGE_LENGTH)?);
        }

        let has_title = u8::read(data)? != 0;
        let title = if has_title {
            Some(read_utf(data, MAX_TITLE_LENGTH)?)
        } else {
            None
        };

        Ok(Self { slot, pages, title })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use steel_utils::codec::VarInt;
    use steel_utils::serial::{ReadFrom as _, WriteTo as _};

    use super::{MAX_PAGE_LENGTH, MAX_PAGES, MAX_TITLE_LENGTH, SEditBook};

    fn encode_string(bytes: &[u8], out: &mut Vec<u8>) {
        VarInt(bytes.len() as i32)
            .write(out)
            .expect("test string length should encode");
        out.extend_from_slice(bytes);
    }

    fn encode(slot: i32, pages: &[&str], title: Option<&str>) -> Vec<u8> {
        let mut out = Vec::new();
        VarInt(slot).write(&mut out).expect("slot should encode");
        VarInt(pages.len() as i32)
            .write(&mut out)
            .expect("page count should encode");
        for page in pages {
            encode_string(page.as_bytes(), &mut out);
        }
        out.push(u8::from(title.is_some()));
        if let Some(title) = title {
            encode_string(title.as_bytes(), &mut out);
        }
        out
    }

    fn decode(bytes: &[u8]) -> std::io::Result<SEditBook> {
        SEditBook::read(&mut Cursor::new(bytes))
    }

    #[test]
    fn reads_pages_without_a_title() {
        let decoded = decode(&encode(3, &["one", "two"], None)).expect("pages should decode");
        assert_eq!(decoded.slot, 3);
        assert_eq!(decoded.pages, ["one", "two"]);
        assert_eq!(decoded.title, None);
    }

    #[test]
    fn reads_signing_title_and_pages() {
        let bytes = encode(40, &["page"], Some("My Book"));
        let decoded = decode(&bytes).expect("signing should decode");
        assert_eq!(decoded.slot, 40);
        assert_eq!(decoded.pages, ["page"]);
        assert_eq!(decoded.title.as_deref(), Some("My Book"));
    }

    #[test]
    fn rejects_more_pages_than_vanilla_allows() {
        let pages = vec![""; MAX_PAGES + 1];
        assert!(decode(&encode(0, &pages, None)).is_err());
    }

    #[test]
    fn rejects_pages_longer_than_vanilla_allows() {
        let page = "a".repeat(MAX_PAGE_LENGTH + 1);
        assert!(decode(&encode(0, &[&page], None)).is_err());
    }

    #[test]
    fn rejects_titles_longer_than_vanilla_allows() {
        let title = "a".repeat(MAX_TITLE_LENGTH + 1);
        assert!(decode(&encode(0, &["page"], Some(&title))).is_err());
    }

    #[test]
    fn accepts_the_vanilla_limits_at_the_boundary() {
        let page = "a".repeat(MAX_PAGE_LENGTH);
        let title = "a".repeat(MAX_TITLE_LENGTH);
        let decoded =
            decode(&encode(0, &[&page], Some(&title))).expect("boundary values should decode");
        assert_eq!(decoded.pages[0].len(), MAX_PAGE_LENGTH);
        assert_eq!(
            decoded.title.map(|title| title.len()),
            Some(MAX_TITLE_LENGTH)
        );
    }
}
