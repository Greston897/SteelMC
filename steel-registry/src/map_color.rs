//! Vanilla `MapColor` palette.
//!
//! Maps store one byte per pixel: the upper six bits are the palette entry and
//! the lower two bits are a brightness modifier (vanilla
//! `MapColor.getRenderColorByte`). Steel keeps the same encoding so that the
//! bytes persisted in `MapState` are bit-compatible with vanilla saved data.

/// Vanilla `MapColor.Brightness`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Brightness {
    /// Vanilla `MapColor.Brightness.LOW`, id 0, multiplier 180/255.
    Low = 0,
    /// Vanilla `MapColor.Brightness.NORMAL`, id 1, multiplier 220/255.
    Normal = 1,
    /// Vanilla `MapColor.Brightness.HIGH`, id 2, multiplier 255/255.
    High = 2,
    /// Vanilla `MapColor.Brightness.LOWEST`, id 3, multiplier 135/255.
    Lowest = 3,
}

impl Brightness {
    /// The two-bit brightness id stored in a map color byte.
    #[must_use]
    pub const fn id(self) -> u8 {
        self as u8
    }

    /// The RGB scale factor in `0..=255`, matching vanilla `Brightness.brightness`.
    #[must_use]
    pub const fn scale(self) -> u8 {
        match self {
            Self::Low => 180,
            Self::Normal => 220,
            Self::High => 255,
            Self::Lowest => 135,
        }
    }

    /// Looks up a brightness by its stored value.
    #[must_use]
    pub const fn from_id(id: u8) -> Self {
        match id & 3 {
            0 => Self::Low,
            1 => Self::Normal,
            2 => Self::High,
            _ => Self::Lowest,
        }
    }
}

/// One vanilla `MapColor` palette entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapColor {
    id: u8,
    rgb: u32,
}

impl MapColor {
    /// Palette id in `0..64`.
    #[must_use]
    pub const fn id(self) -> u8 {
        self.id
    }

    /// Raw RGB value, matching vanilla `MapColor.color`.
    #[must_use]
    pub const fn rgb(self) -> u32 {
        self.rgb
    }

    /// Encodes this color plus a brightness into the byte stored in `MapState.colors`.
    #[must_use]
    pub const fn render_byte(self, brightness: Brightness) -> u8 {
        (self.id << 2) | (brightness.id() & 3)
    }
}

const fn color(id: u8, rgb: u32) -> MapColor {
    MapColor { id, rgb }
}

/// Vanilla `MapColor` palette, indexed by palette id.
///
/// Entry 0 is `MapColor.CLEAR`, which means "no color" and makes the map
/// renderer skip the block (`BlockState.getMapColor` returning `CLEAR`).
pub const MAP_COLORS: [MapColor; 64] = [
    color(0, 0),
    color(1, 8_368_696),
    color(2, 16_247_203),
    color(3, 13_092_807),
    color(4, 16_711_680),
    color(5, 10_526_975),
    color(6, 10_987_431),
    color(7, 31_744),
    color(8, 16_777_215),
    color(9, 10_791_096),
    color(10, 9_923_917),
    color(11, 7_368_816),
    color(12, 4_210_943),
    color(13, 9_402_184),
    color(14, 16_776_437),
    color(15, 14_188_339),
    color(16, 11_685_080),
    color(17, 6_724_056),
    color(18, 15_066_419),
    color(19, 8_375_321),
    color(20, 15_892_389),
    color(21, 5_000_268),
    color(22, 10_066_329),
    color(23, 5_013_401),
    color(24, 8_339_378),
    color(25, 3_361_970),
    color(26, 6_704_179),
    color(27, 6_717_235),
    color(28, 10_040_115),
    color(29, 1_644_825),
    color(30, 16_445_005),
    color(31, 6_085_589),
    color(32, 4_882_687),
    color(33, 55_610),
    color(34, 8_476_209),
    color(35, 7_340_544),
    color(36, 13_742_497),
    color(37, 10_441_252),
    color(38, 9_787_244),
    color(39, 7_367_818),
    color(40, 12_223_780),
    color(41, 6_780_213),
    color(42, 10_505_550),
    color(43, 3_746_083),
    color(44, 8_874_850),
    color(45, 5_725_276),
    color(46, 8_014_168),
    color(47, 4_996_700),
    color(48, 4_993_571),
    color(49, 5_001_770),
    color(50, 9_321_518),
    color(51, 2_430_480),
    color(52, 12_398_641),
    color(53, 9_715_553),
    color(54, 6_035_741),
    color(55, 1_474_182),
    color(56, 3_837_580),
    color(57, 5_647_422),
    color(58, 1_356_933),
    color(59, 6_579_300),
    color(60, 14_200_723),
    color(61, 8_365_974),
    // Vanilla leaves 62 and 63 unregistered (`MapColor.CLEAR` fallback).
    color(62, 0),
    color(63, 0),
];

/// Vanilla `MapColor.CLEAR`, the palette id that means "draw nothing here".
pub const CLEAR: MapColor = MAP_COLORS[0];
/// Vanilla `MapColor.WATER_BLUE`, used for the water depth brightness rule.
pub const WATER_BLUE: MapColor = MAP_COLORS[12];

/// Vanilla `MapColor.get(int)`.
#[must_use]
pub const fn by_id(id: u8) -> MapColor {
    MAP_COLORS[id as usize]
}

/// Decodes a stored map color byte into an RGB value.
///
/// Mirrors vanilla `MapColor.getRenderColor(int)`: palette entry `byte >> 2`,
/// brightness `byte & 3`, then the brightness scale applied to every channel.
#[must_use]
pub const fn render_rgb(color_byte: u8) -> u32 {
    let map_color = by_id(color_byte >> 2);
    if map_color.id == 0 {
        return 0;
    }

    let scale = Brightness::from_id(color_byte).scale() as u32;
    let rgb = map_color.rgb;
    let red = ((rgb >> 16) & 0xFF) * scale / 255;
    let green = ((rgb >> 8) & 0xFF) * scale / 255;
    let blue = (rgb & 0xFF) * scale / 255;
    (red << 16) | (green << 8) | blue
}

#[cfg(test)]
mod tests {
    use super::{
        Brightness, CLEAR, MAP_COLORS, WATER_BLUE, by_id, render_rgb,
    };

    #[test]
    fn palette_matches_vanilla_ids() {
        assert_eq!(CLEAR.id(), 0);
        assert_eq!(CLEAR.rgb(), 0);
        assert_eq!(by_id(11).rgb(), 7_368_816); // STONE_GRAY
        assert_eq!(by_id(1).rgb(), 8_368_696); // PALE_GREEN
        assert_eq!(by_id(28).rgb(), 10_040_115); // RED
        assert_eq!(by_id(61).rgb(), 8_365_974); // LICHEN_GREEN
        assert_eq!(MAP_COLORS.len(), 64);
        for (index, entry) in MAP_COLORS.iter().enumerate() {
            assert_eq!(u8::try_from(index).unwrap(), entry.id());
        }
    }

    #[test]
    fn render_byte_packs_id_and_brightness() {
        assert_eq!(WATER_BLUE.render_byte(Brightness::High), (12 << 2) | 2);
        assert_eq!(by_id(11).render_byte(Brightness::Lowest), (11 << 2) | 3);
        assert_eq!(CLEAR.render_byte(Brightness::Normal), 1);
        // The low two bits never leak into the palette lookup.
        assert_eq!(by_id(12).render_byte(Brightness::Lowest) >> 2, 12);
    }

    #[test]
    fn brightness_round_trips_and_scales() {
        for id in 0..4_u8 {
            assert_eq!(Brightness::from_id(id).id(), id);
        }
        assert_eq!(Brightness::Low.scale(), 180);
        assert_eq!(Brightness::Normal.scale(), 220);
        assert_eq!(Brightness::High.scale(), 255);
        assert_eq!(Brightness::Lowest.scale(), 135);
    }

    #[test]
    fn render_rgb_scales_each_channel() {
        // STONE_GRAY 0x707070 at NORMAL (220/255) rounds down to 0x606060.
        assert_eq!(render_rgb(by_id(11).render_byte(Brightness::Normal)), 0x60_6060);
        assert_eq!(render_rgb(by_id(11).render_byte(Brightness::High)), 0x70_7070);
    }

    #[test]
    fn clear_renders_as_transparent() {
        assert_eq!(render_rgb(0), 0);
        assert_eq!(render_rgb(CLEAR.render_byte(Brightness::Lowest)), 0);
    }
}
