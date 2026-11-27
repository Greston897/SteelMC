//! Версии протокола — как в ViaVersion, но Rust enum.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProtocolVersion(pub i32);

impl ProtocolVersion {
    // актуальные как в ViaVersion 5.x
    pub const V1_21_5: Self = Self(770);
    pub const V1_21_6: Self = Self(771);
    pub const V1_21_7: Self = Self(772);
    pub const V1_21_8: Self = Self(773);
    pub const V26_3: Self = Self(777); // Steel 26.3 — наш целевой
    pub const UNKNOWN: Self = Self(-1);

    pub const fn new(id: i32) -> Self { Self(id) }
    pub fn id(self) -> i32 { self.0 }
    pub fn is_unknown(self) -> bool { self.0 < 0 }
    pub fn supports(self, other: Self) -> bool { self.0 >= other.0 }

    pub fn minecraft_version(self) -> &'static str {
        match self.0 {
            770 => "1.21.5",
            771 => "1.21.6",
            772 => "1.21.7",
            773 => "1.21.8",
            777 => "26.3",
            _ => "unknown",
        }
    }

    pub fn from_minecraft_version(s: &str) -> Self {
        match s {
            "1.21.5" => Self::V1_21_5,
            "1.21.6" => Self::V1_21_6,
            "1.21.7" => Self::V1_21_7,
            "1.21.8" => Self::V1_21_8,
            "26.3"    => Self::V26_3,
            _ => Self::UNKNOWN,
        }
    }

    /// Все известные версии по возрастанию.
    pub const ALL: &'static [Self] = &[Self::V1_21_5, Self::V1_21_6, Self::V1_21_7, Self::V1_21_8, Self::V26_3];
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (protocol {})", self.minecraft_version(), self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordering() {
        assert!(ProtocolVersion::V26_3 > ProtocolVersion::V1_21_8);
        assert!(ProtocolVersion::V1_21_5.supports(ProtocolVersion::V1_21_5));
        assert!(!ProtocolVersion::V1_21_5.supports(ProtocolVersion::V26_3));
    }
}
