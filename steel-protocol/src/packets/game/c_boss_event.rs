//! Boss bar packet.
//!
//! Vanilla splits boss-bar changes into six actions carried by one packet
//! (`ClientboundBossEventPacket`). The action id and its payload are written in
//! [`CBossEvent::write`].

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_BOSS_EVENT;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;
use text_components::TextComponent;
use uuid::Uuid;

/// Vanilla `BossEvent.BossBarColor` ordinals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossBarColor {
    /// Pink bar.
    Pink = 0,
    /// Blue bar.
    Blue = 1,
    /// Red bar.
    Red = 2,
    /// Green bar.
    Green = 3,
    /// Yellow bar.
    Yellow = 4,
    /// Purple bar.
    Purple = 5,
    /// White bar.
    White = 6,
}

impl BossBarColor {
    /// Wire ordinal of the colour.
    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }
}

/// Vanilla `BossEvent.BossBarOverlay` ordinals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossBarOverlay {
    /// Continuous progress bar.
    Progress = 0,
    /// Bar split into six segments.
    Notched6 = 1,
    /// Bar split into ten segments.
    Notched10 = 2,
    /// Bar split into twelve segments.
    Notched12 = 3,
    /// Bar split into twenty segments.
    Notched20 = 4,
}

impl BossBarOverlay {
    /// Wire ordinal of the overlay.
    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }
}

/// Payload of one boss-bar action.
#[derive(Debug, Clone)]
pub enum BossEventAction {
    /// Creates a new boss bar on the client.
    Add {
        /// Displayed title.
        name: TextComponent,
        /// Progress in `0.0..=1.0`.
        progress: f32,
        /// Bar colour.
        color: BossBarColor,
        /// Bar overlay style.
        overlay: BossBarOverlay,
        /// Vanilla `BossEvent` flags (`1` darken screen, `2` boss music, `4` world fog).
        flags: u8,
    },
    /// Removes the boss bar.
    Remove,
    /// Updates the progress only.
    UpdateProgress {
        /// Progress in `0.0..=1.0`.
        progress: f32,
    },
    /// Updates the title only.
    UpdateName {
        /// Displayed title.
        name: TextComponent,
    },
    /// Updates the colour and overlay.
    UpdateStyle {
        /// Bar colour.
        color: BossBarColor,
        /// Bar overlay style.
        overlay: BossBarOverlay,
    },
    /// Updates the `BossEvent` flags.
    UpdateProperties {
        /// See [`BossEventAction::Add`].
        flags: u8,
    },
}

impl BossEventAction {
    /// Vanilla action id written in front of the payload.
    #[must_use]
    const fn id(&self) -> i32 {
        match self {
            Self::Add { .. } => 0,
            Self::Remove => 1,
            Self::UpdateProgress { .. } => 2,
            Self::UpdateName { .. } => 3,
            Self::UpdateStyle { .. } => 4,
            Self::UpdateProperties { .. } => 5,
        }
    }
}

/// Updates one boss bar on the client.
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Play = C_BOSS_EVENT)]
pub struct CBossEvent {
    /// Identity of the boss bar.
    pub uuid: Uuid,
    /// Action to apply.
    pub action: BossEventAction,
}

impl CBossEvent {
    /// Builds an `add` action.
    #[must_use]
    pub const fn add(
        uuid: Uuid,
        name: TextComponent,
        progress: f32,
        color: BossBarColor,
        overlay: BossBarOverlay,
        flags: u8,
    ) -> Self {
        Self {
            uuid,
            action: BossEventAction::Add {
                name,
                progress,
                color,
                overlay,
                flags,
            },
        }
    }

    /// Builds a `remove` action.
    #[must_use]
    pub const fn remove(uuid: Uuid) -> Self {
        Self {
            uuid,
            action: BossEventAction::Remove,
        }
    }

    /// Builds an `update_progress` action.
    #[must_use]
    pub const fn update_progress(uuid: Uuid, progress: f32) -> Self {
        Self {
            uuid,
            action: BossEventAction::UpdateProgress { progress },
        }
    }

    /// Builds an `update_name` action.
    #[must_use]
    pub const fn update_name(uuid: Uuid, name: TextComponent) -> Self {
        Self {
            uuid,
            action: BossEventAction::UpdateName { name },
        }
    }
}

impl WriteTo for CBossEvent {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        self.uuid.write(writer)?;
        VarInt(self.action.id()).write(writer)?;
        match &self.action {
            BossEventAction::Add {
                name,
                progress,
                color,
                overlay,
                flags,
            } => {
                name.write(writer)?;
                writer.write_all(&progress.to_be_bytes())?;
                VarInt(color.id()).write(writer)?;
                VarInt(overlay.id()).write(writer)?;
                writer.write_all(&[*flags])
            }
            BossEventAction::Remove => Ok(()),
            BossEventAction::UpdateProgress { progress } => {
                writer.write_all(&progress.to_be_bytes())
            }
            BossEventAction::UpdateName { name } => name.write(writer),
            BossEventAction::UpdateStyle { color, overlay } => {
                VarInt(color.id()).write(writer)?;
                VarInt(overlay.id()).write(writer)
            }
            BossEventAction::UpdateProperties { flags } => writer.write_all(&[*flags]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(packet: &CBossEvent) -> Vec<u8> {
        let mut bytes = Vec::new();
        packet
            .write(&mut bytes)
            .unwrap_or_else(|error| panic!("boss event encoding failed: {error}"));
        bytes
    }

    #[test]
    fn remove_writes_uuid_then_action_id() {
        let uuid = Uuid::from_u128(0x0011_2233_4455_6677_8899_aabb_ccdd_eeff);
        let bytes = encode(&CBossEvent::remove(uuid));
        assert_eq!(bytes.len(), 17, "uuid plus a one byte action id");
        assert_eq!(&bytes[..16], uuid.as_bytes());
        assert_eq!(bytes[16], 1);
    }

    #[test]
    fn update_progress_writes_big_endian_float() {
        let uuid = Uuid::from_u128(7);
        let bytes = encode(&CBossEvent::update_progress(uuid, 0.5));
        assert_eq!(bytes.len(), 16 + 1 + 4);
        assert_eq!(bytes[16], 2);
        assert_eq!(&bytes[17..], &0.5f32.to_be_bytes());
    }

    #[test]
    fn add_writes_colour_and_overlay_ordinals() {
        let uuid = Uuid::from_u128(9);
        let packet = CBossEvent::add(
            uuid,
            TextComponent::plain("TPS"),
            1.0,
            BossBarColor::Green,
            BossBarOverlay::Progress,
            0,
        );
        let bytes = encode(&packet);
        assert_eq!(bytes[16], 0, "add action id");
        assert_eq!(BossBarColor::Green.id(), 3);
        assert_eq!(BossBarOverlay::Notched20.id(), 4);
        assert_eq!(bytes[bytes.len() - 3], 3, "green colour ordinal");
        assert_eq!(bytes[bytes.len() - 2], 0, "progress overlay ordinal");
        assert_eq!(bytes[bytes.len() - 1], 0, "no flags");
    }
}
