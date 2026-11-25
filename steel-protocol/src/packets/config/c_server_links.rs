//! Server links packet (`ClientboundServerLinksPacket`), valid in both `Config` and `Play`.
//!
//! Vanilla `ServerLinks.UNTRUSTED_LINKS_STREAM_CODEC`:
//! `VarInt` list of `(Either<KnownLinkType, Component>, String url)`.

use serde::{Deserialize, Serialize};
use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::{config, play};
use steel_utils::codec::Or;
use text_components::TextComponent;

/// Clientbound packet that advertises pause-menu server links.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Config = config::C_SERVER_LINKS, Play = play::C_SERVER_LINKS)]
pub struct CServerLinks {
    /// Advertised links.
    pub links: Vec<Link>,
}

/// Built-in link labels (`ServerLinks.KnownLinkType` ordinals).
#[derive(WriteTo, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[write(as = VarInt)]
#[serde(rename_all = "snake_case")]
pub enum ServerLinksType {
    BugReport = 0,
    CommunityGuidelines = 1,
    Support = 2,
    Status = 3,
    Feedback = 4,
    Community = 5,
    Website = 6,
    Forums = 7,
    News = 8,
    Announcements = 9,
}

/// One link entry (`Either<KnownLinkType, Component>` + target URL).
#[derive(WriteTo, Clone, Debug)]
pub struct Link {
    /// `true` for [`ServerLinksType`] (`Either.left`), `false` for custom [`TextComponent`].
    pub is_built_in: bool,
    /// Link label payload without a duplicate discriminant.
    pub label: Label,
    /// Target URL string.
    #[write(as = Prefixed(VarInt))]
    pub url: String,
}

impl Link {
    /// Constructs a link with a consistent `Either` discriminant.
    #[must_use]
    pub const fn new(label: Label, url: String) -> Self {
        Self {
            is_built_in: label.is_left(),
            label,
            url,
        }
    }
}

/// Either a built-in [`ServerLinksType`] (`Left`) or a custom [`TextComponent`] (`Right`).
pub type Label = Or<ServerLinksType, TextComponent>;

#[cfg(test)]
mod tests {
    use super::{CServerLinks, Link, ServerLinksType};
    use steel_utils::codec::Or;
    use steel_utils::serial::WriteTo as _;

    #[test]
    fn built_in_link_encodes_either_left_then_url() {
        let pkt = CServerLinks {
            links: vec![Link::new(
                Or::Left(ServerLinksType::Website),
                "https://example.com".to_owned(),
            )],
        };
        let mut actual = Vec::new();
        pkt.write(&mut actual).unwrap();

        let mut expected = vec![
            1, // 1 link
            1, // Either.left (built-in)
            6, // ServerLinksType::Website
        ];
        "https://example.com"
            .to_owned()
            .write(&mut expected)
            .unwrap();
        assert_eq!(actual, expected);
    }
}
