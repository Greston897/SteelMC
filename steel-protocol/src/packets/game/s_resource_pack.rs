//! Resource pack response packet.

use steel_macros::{ReadFrom, ServerPacket};
use uuid::Uuid;

/// Vanilla `ServerboundResourcePackPacket.Action` ordinals.
///
/// Vanilla writes the enum with `writeEnum`, which is a `VarInt`; the order
/// below is the declaration order and therefore the wire order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourcePackAction {
    /// The client applied the pack.
    SuccessfullyLoaded = 0,
    /// The client refused the pack.
    Declined = 1,
    /// The download failed.
    FailedDownload = 2,
    /// The pack was accepted and the download started.
    Accepted = 3,
    /// The downloaded pack is being applied.
    Downloaded = 4,
    /// The pack was invalidated by `CResourcePackPop`.
    InvalidUrl = 5,
    /// The pack failed to reload.
    FailedReload = 6,
    /// The pack was discarded.
    Discarded = 7,
}

impl ResourcePackAction {
    /// Decodes a vanilla action ordinal.
    #[must_use]
    pub const fn from_ordinal(ordinal: i32) -> Option<Self> {
        Some(match ordinal {
            0 => Self::SuccessfullyLoaded,
            1 => Self::Declined,
            2 => Self::FailedDownload,
            3 => Self::Accepted,
            4 => Self::Downloaded,
            5 => Self::InvalidUrl,
            6 => Self::FailedReload,
            7 => Self::Discarded,
            _ => return None,
        })
    }
}

/// Serverbound packet that reports the result of a resource pack push.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SResourcePack {
    /// `UUID` of the pack the result belongs to.
    pub id: Uuid,
    /// Vanilla `ServerboundResourcePackPacket.Action` ordinal.
    #[read(as = VarInt)]
    pub action: i32,
}

impl SResourcePack {
    /// Returns the decoded vanilla action, if the ordinal is known.
    #[must_use]
    pub const fn decoded_action(&self) -> Option<ResourcePackAction> {
        ResourcePackAction::from_ordinal(self.action)
    }
}
