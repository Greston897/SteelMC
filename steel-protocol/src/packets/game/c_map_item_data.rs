//! Clientbound map item data packet (`ClientboundMapItemDataPacket`).
//!
//! Wire format (vanilla `MapItemSavedData.MapPatch.STREAM_CODEC`):
//! 1. `map_id` — `VarInt`
//! 2. `scale` — byte
//! 3. `locked` — bool
//! 4. decorations — optional list: presence bool, then `VarInt` count of
//!    `(type: VarInt, x: byte, z: byte, rot: byte, name: optional component)`
//! 5. color patch — `0_u8` when absent; otherwise `width: u8 (> 0), height: u8,
//!    start_x: u8, start_z: u8` followed by a `VarInt`-prefixed color byte array
//!    (NO separate boolean presence flag before `width`).

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_MAP_ITEM_DATA;
use steel_utils::codec::VarInt;
use text_components::TextComponent;

/// One icon drawn on a map.
#[derive(WriteTo, Clone, Debug, PartialEq, Eq)]
pub struct MapDecorationEntry {
    /// Registry id of the decoration type (vanilla `MapDecorationType`).
    #[write(as = VarInt)]
    pub type_id: i32,
    /// Icon X in map pixel coordinates (`-128..=127`).
    pub x: i8,
    /// Icon Z in map pixel coordinates (`-128..=127`).
    pub z: i8,
    /// Icon rotation in units of 360/16 degrees.
    pub rotation: i8,
    /// Custom display name, e.g. a named banner.
    pub name: Option<TextComponent>,
}

impl MapDecorationEntry {
    /// Creates a decoration without a custom name.
    #[must_use]
    pub const fn new(type_id: i32, x: i8, z: i8, rotation: i8) -> Self {
        Self {
            type_id,
            x,
            z,
            rotation,
            name: None,
        }
    }
}

/// Optional map patch wrapper with vanilla's nonzero-width discriminator
/// (`MapItemSavedData.MapPatch.STREAM_CODEC`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OptionalMapPatch(pub Option<MapPatch>);

impl OptionalMapPatch {
    /// Wraps an optional patch (zero-width patches normalize to `None` as in vanilla).
    #[must_use]
    pub fn new(patch: Option<MapPatch>) -> Self {
        Self(patch.filter(|p| p.width > 0))
    }

    /// Returns the patch, if any.
    #[must_use]
    pub const fn patch(&self) -> Option<&MapPatch> {
        self.0.as_ref()
    }

    /// Returns whether there is no patch to send.
    #[must_use]
    pub const fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl steel_utils::serial::WriteTo for OptionalMapPatch {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        match &self.0 {
            Some(patch) if patch.width > 0 => patch.write(writer),
            _ => 0_u8.write(writer),
        }
    }
}

/// A rectangular patch of changed map colors (`MapItemSavedData.MapPatch`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapPatch {
    /// Patch width in pixels (`1..=128`).
    pub width: u8,
    /// Patch height in pixels (`1..=128`).
    pub height: u8,
    /// X of the first patch pixel.
    pub start_x: u8,
    /// Z of the first patch pixel.
    pub start_z: u8,
    /// `width * height` color bytes, row-major.
    pub colors: Vec<u8>,
}

impl steel_utils::serial::WriteTo for MapPatch {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        self.width.write(writer)?;
        self.height.write(writer)?;
        self.start_x.write(writer)?;
        self.start_z.write(writer)?;
        VarInt(self.colors.len() as i32).write(writer)?;
        writer.write_all(&self.colors)
    }
}

/// Sends map state to the client.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_MAP_ITEM_DATA)]
pub struct CMapItemData {
    /// Map saved-data id (`MapId`).
    #[write(as = VarInt)]
    pub map_id: i32,
    /// Map scale (`0..=4`).
    pub scale: i8,
    /// Whether the map is locked (copied with glass pane in a cartography table).
    pub locked: bool,
    /// Replacement decoration list, when the decoration set changed.
    pub decorations: Option<Vec<MapDecorationEntry>>,
    /// Changed color rectangle, when pixels changed.
    pub patch: OptionalMapPatch,
}

impl CMapItemData {
    /// Creates a map update packet.
    #[must_use]
    pub const fn new(
        map_id: i32,
        scale: i8,
        locked: bool,
        decorations: Option<Vec<MapDecorationEntry>>,
        patch: OptionalMapPatch,
    ) -> Self {
        Self {
            map_id,
            scale,
            locked,
            decorations,
            patch,
        }
    }

    /// Whether this packet carries nothing the client would apply.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.decorations.is_none() && self.patch.is_none()
    }

    /// Builds a decorations-only update.
    #[must_use]
    pub const fn decorations_only(
        map_id: i32,
        scale: i8,
        locked: bool,
        decorations: Vec<MapDecorationEntry>,
    ) -> Self {
        Self::new(
            map_id,
            scale,
            locked,
            Some(decorations),
            OptionalMapPatch(None),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{CMapItemData, MapDecorationEntry, MapPatch, OptionalMapPatch};
    use steel_utils::serial::WriteTo as _;
    use text_components::TextComponent;

    fn encode(packet: &CMapItemData) -> Vec<u8> {
        let mut bytes = Vec::new();
        packet.write(&mut bytes).expect("packet should encode");
        bytes
    }

    #[test]
    fn full_packet_encodes_vanilla_field_order_with_width_discriminator() {
        let packet = CMapItemData::new(
            300,
            1,
            true,
            Some(vec![MapDecorationEntry::new(9, -1, 5, 12)]),
            OptionalMapPatch::new(Some(MapPatch {
                width: 2,
                height: 1,
                start_x: 3,
                start_z: 4,
                colors: vec![0x2A, 0x0B],
            })),
        );

        let bytes = encode(&packet);
        assert_eq!(
            bytes,
            vec![
                // VarInt 300 = 0xAC 0x02
                0xAC, 0x02, // map id
                1,    // scale
                1,    // locked
                1,    // decorations present
                1,    // one decoration
                9,    // type id
                0xFF, // x = -1
                5,    // z
                12,   // rotation
                0,    // no display name
                2, 1, 3, 4, // MapPatch: width (>0 acts as discriminator), height, startX, startY
                2, // VarInt color count
                0x2A, 0x0B,
            ]
        );
    }

    #[test]
    fn empty_update_skips_both_optional_sections() {
        let packet = CMapItemData::new(7, 0, false, None, OptionalMapPatch::default());
        assert!(packet.is_empty());
        assert_eq!(encode(&packet), vec![7, 0, 0, 0, 0]);
    }

    #[test]
    fn decoration_name_is_optional_component() {
        let mut entry = MapDecorationEntry::new(3, 64, -64, 0);
        entry.name = Some(TextComponent::plain("Base"));
        let packet = CMapItemData::new(1, 4, false, Some(vec![entry]), OptionalMapPatch::default());

        let bytes = encode(&packet);
        assert_eq!(&bytes[..10], &[1, 4, 0, 1, 1, 3, 64, 0xC0, 0, 1]);
        assert!(bytes.len() > 10);
        assert_eq!(*bytes.last().unwrap(), 0, "patch width 0 marks absent");
    }

    #[test]
    fn patch_without_decorations_writes_width_directly() {
        let packet = CMapItemData::new(
            1_000,
            2,
            false,
            None,
            OptionalMapPatch::new(Some(MapPatch {
                width: 1,
                height: 1,
                start_x: 0,
                start_z: 0,
                colors: vec![0xFF],
            })),
        );

        assert_eq!(
            encode(&packet),
            vec![0xE8, 0x07, 2, 0, 0, 1, 1, 0, 0, 1, 0xFF]
        );
    }
}
