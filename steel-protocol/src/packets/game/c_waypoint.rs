//! Locator bar waypoint packet (`ClientboundTrackedWaypointPacket`).
//!
//! Vanilla wire format (`ClientboundTrackedWaypointPacket.STREAM_CODEC`):
//! 1. `operation` — `VarInt` (`0 = Track`, `1 = Untrack`, `2 = Update`)
//! 2. `waypoint` (`TrackedWaypoint.STREAM_CODEC`):
//!    - `identifier`: `Either<UUID, String>` (`true` + 16-byte UUID, or `false` + UTF-8 string)
//!    - `icon` (`Waypoint.Icon.STREAM_CODEC`): `Identifier` style + `Option<[u8; 3]>` RGB color
//!    - `type`: `VarInt` (`0 = Empty`, `1 = Vec3i`, `2 = Chunk`, `3 = Azimuth`)
//!    - type payload:
//!      - `Empty`: 0 bytes (used for `Untrack`)
//!      - `Vec3i`: `VarInt(x), VarInt(y), VarInt(z)`
//!      - `Chunk`: `VarInt(chunk_x), VarInt(chunk_z)`
//!      - `Azimuth`: `f32` angle in radians

use std::io::Write;

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_WAYPOINT;
use steel_utils::Identifier;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;
use uuid::Uuid;

/// Identifies a tracked waypoint on the client (`Either<UUID, String>`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WaypointId {
    /// Entity / player UUID (`Either.left`).
    Uuid(Uuid),
    /// Named waypoint string (`Either.right`).
    Name(String),
}

impl WriteTo for WaypointId {
    fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Self::Uuid(uuid) => {
                true.write(writer)?;
                uuid.write(writer)
            }
            Self::Name(name) => {
                false.write(writer)?;
                name.write(writer)
            }
        }
    }
}

/// Vanilla `Waypoint.Icon` (`style: ResourceKey<WaypointStyleAsset>`, `color: Optional<RGB>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaypointIcon {
    /// Waypoint style asset identifier (defaults to `minecraft:default`).
    pub style: Identifier,
    /// Optional 24-bit `[r, g, b]` tint (`ByteBufCodecs.RGB_COLOR`).
    pub color: Option<[u8; 3]>,
}

impl Default for WaypointIcon {
    fn default() -> Self {
        Self {
            style: Identifier::vanilla_static("default"),
            color: None,
        }
    }
}

impl WriteTo for WaypointIcon {
    fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.style.write(writer)?;
        match self.color {
            Some([r, g, b]) => {
                true.write(writer)?;
                writer.write_all(&[r, g, b])
            }
            None => false.write(writer),
        }
    }
}

/// Vanilla `TrackedWaypoint` payload variants and `Type` ordinals.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrackedWaypointData {
    /// `Type.EMPTY` (`0`) — used by `Untrack` packets.
    Empty,
    /// `Type.VEC3I` (`1`) — exact block coordinates (`Vec3i.STREAM_CODEC`: three `VarInt`s).
    Vec3i {
        /// Block X coordinate.
        x: i32,
        /// Block Y coordinate.
        y: i32,
        /// Block Z coordinate.
        z: i32,
    },
    /// `Type.CHUNK` (`2`) — chunk coordinates (`VarInt(chunk_x), VarInt(chunk_z)`).
    Chunk {
        /// Chunk X coordinate.
        chunk_x: i32,
        /// Chunk Z coordinate.
        chunk_z: i32,
    },
    /// `Type.AZIMUTH` (`3`) — horizontal bearing in radians (`f32`).
    Azimuth {
        /// Bearing angle in radians.
        angle: f32,
    },
}

impl WriteTo for TrackedWaypointData {
    fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match *self {
            Self::Empty => VarInt(0).write(writer),
            Self::Vec3i { x, y, z } => {
                VarInt(1).write(writer)?;
                VarInt(x).write(writer)?;
                VarInt(y).write(writer)?;
                VarInt(z).write(writer)
            }
            Self::Chunk { chunk_x, chunk_z } => {
                VarInt(2).write(writer)?;
                VarInt(chunk_x).write(writer)?;
                VarInt(chunk_z).write(writer)
            }
            Self::Azimuth { angle } => {
                VarInt(3).write(writer)?;
                angle.write(writer)
            }
        }
    }
}

/// Vanilla `ClientboundTrackedWaypointPacket.Operation` combined with its `TrackedWaypoint`.
#[derive(Debug, Clone, PartialEq)]
pub enum WaypointOperation {
    /// `Operation.TRACK` (`0`).
    Track {
        /// Waypoint identifier.
        id: WaypointId,
        /// Waypoint style and optional RGB color.
        icon: WaypointIcon,
        /// Initial positional payload.
        data: TrackedWaypointData,
    },
    /// `Operation.UNTRACK` (`1`) — encodes an `EmptyTrackedWaypoint` with `Waypoint.Icon.NULL`.
    Untrack {
        /// Waypoint identifier to remove.
        id: WaypointId,
    },
    /// `Operation.UPDATE` (`2`).
    Update {
        /// Waypoint identifier.
        id: WaypointId,
        /// Current waypoint icon.
        icon: WaypointIcon,
        /// Updated positional payload.
        data: TrackedWaypointData,
    },
}

/// Clientbound packet that adds, removes, or updates a locator bar waypoint.
#[derive(ClientPacket, Debug, Clone, PartialEq)]
#[packet_id(Play = C_WAYPOINT)]
pub struct CWaypoint {
    /// Operation and waypoint payload.
    pub operation: WaypointOperation,
}

impl CWaypoint {
    /// Builds a `TRACK` packet for an entity UUID.
    #[must_use]
    pub const fn track_uuid(uuid: Uuid, icon: WaypointIcon, data: TrackedWaypointData) -> Self {
        Self {
            operation: WaypointOperation::Track {
                id: WaypointId::Uuid(uuid),
                icon,
                data,
            },
        }
    }

    /// Builds an `UPDATE` packet for an entity UUID.
    #[must_use]
    pub const fn update_uuid(uuid: Uuid, icon: WaypointIcon, data: TrackedWaypointData) -> Self {
        Self {
            operation: WaypointOperation::Update {
                id: WaypointId::Uuid(uuid),
                icon,
                data,
            },
        }
    }

    /// Builds an `UNTRACK` packet for an entity UUID.
    #[must_use]
    pub const fn untrack_uuid(uuid: Uuid) -> Self {
        Self {
            operation: WaypointOperation::Untrack {
                id: WaypointId::Uuid(uuid),
            },
        }
    }
}

impl WriteTo for CWaypoint {
    fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match &self.operation {
            WaypointOperation::Track { id, icon, data } => {
                VarInt(0).write(writer)?;
                id.write(writer)?;
                icon.write(writer)?;
                data.write(writer)
            }
            WaypointOperation::Untrack { id } => {
                VarInt(1).write(writer)?;
                id.write(writer)?;
                WaypointIcon::default().write(writer)?;
                TrackedWaypointData::Empty.write(writer)
            }
            WaypointOperation::Update { id, icon, data } => {
                VarInt(2).write(writer)?;
                id.write(writer)?;
                icon.write(writer)?;
                data.write(writer)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(pkt: &CWaypoint) -> Vec<u8> {
        let mut out = Vec::new();
        pkt.write(&mut out).expect("encode waypoint");
        out
    }

    #[test]
    fn track_vec3i_encodes_exact_vanilla_layout() {
        let uuid = Uuid::from_u128(0x0102_0304_0506_0708_090A_0B0C_0D0E_0F10);
        let icon = WaypointIcon {
            style: Identifier::vanilla_static("bowtie"),
            color: Some([0x12, 0x34, 0x56]),
        };
        let pkt = CWaypoint::track_uuid(
            uuid,
            icon,
            TrackedWaypointData::Vec3i {
                x: 10,
                y: 64,
                z: 20,
            },
        );
        let bytes = encode(&pkt);

        let mut expected = Vec::new();
        expected.push(0); // Operation.TRACK
        expected.push(1); // Either.left (UUID)
        expected.extend_from_slice(uuid.as_bytes());
        "minecraft:bowtie".to_owned().write(&mut expected).unwrap();
        expected.push(1); // color present
        expected.extend_from_slice(&[0x12, 0x34, 0x56]);
        expected.push(1); // Type.VEC3I
        expected.extend_from_slice(&[10, 64, 20]);
        assert_eq!(bytes, expected);
    }

    #[test]
    fn untrack_encodes_empty_tracked_waypoint() {
        let pkt = CWaypoint {
            operation: WaypointOperation::Untrack {
                id: WaypointId::Name("home".to_owned()),
            },
        };
        let bytes = encode(&pkt);

        let mut expected = Vec::new();
        expected.push(1); // Operation.UNTRACK
        expected.push(0); // Either.right (String)
        "home".to_owned().write(&mut expected).unwrap();
        "minecraft:default".to_owned().write(&mut expected).unwrap();
        expected.push(0); // color absent
        expected.push(0); // Type.EMPTY
        assert_eq!(bytes, expected);
    }

    #[test]
    fn update_chunk_and_azimuth_encode_exact_type_ordinals() {
        let uuid = Uuid::nil();
        let chunk_pkt = CWaypoint::update_uuid(
            uuid,
            WaypointIcon::default(),
            TrackedWaypointData::Chunk {
                chunk_x: -2,
                chunk_z: 5,
            },
        );
        let mut expected_chunk = Vec::new();
        expected_chunk.push(2); // Operation.UPDATE
        expected_chunk.push(1);
        expected_chunk.extend_from_slice(uuid.as_bytes());
        "minecraft:default"
            .to_owned()
            .write(&mut expected_chunk)
            .unwrap();
        expected_chunk.push(0);
        expected_chunk.push(2); // Type.CHUNK
        VarInt(-2).write(&mut expected_chunk).unwrap();
        VarInt(5).write(&mut expected_chunk).unwrap();
        assert_eq!(encode(&chunk_pkt), expected_chunk);

        let az_pkt = CWaypoint::update_uuid(
            uuid,
            WaypointIcon::default(),
            TrackedWaypointData::Azimuth { angle: 1.5 },
        );
        let az_bytes = encode(&az_pkt);
        assert_eq!(&az_bytes[az_bytes.len() - 5..az_bytes.len() - 4], &[3]);
        assert_eq!(&az_bytes[az_bytes.len() - 4..], &1.5_f32.to_be_bytes());
    }
}
