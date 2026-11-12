//! Waypoint packet.

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_WAYPOINT;
use steel_utils::Identifier;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;

/// Vanilla `WaypointManager` operation encoded in the packet header.
#[derive(Debug, Clone)]
pub enum WaypointOperation {
    /// Registers a new waypoint (`0`).
    Track {
        /// Transmission source identifier.
        source: Identifier,
        /// Icon registry index.
        icon: i32,
        /// Waypoint colour (`ARGB`).
        color: i32,
        /// Waypoint payload.
        waypoint: WaypointData,
    },
    /// Removes a waypoint (`1`).
    Untrack {
        /// Transmission source identifier.
        source: Identifier,
    },
    /// Updates the position of a tracked waypoint (`2`).
    Update {
        /// Transmission source identifier.
        source: Identifier,
        /// Waypoint payload.
        waypoint: WaypointData,
    },
}

/// Position and styling shared by tracked and updated waypoints.
#[derive(Debug, Clone)]
pub struct WaypointData {
    /// `UUID` of the waypoint owner.
    pub uuid: uuid::Uuid,
    /// World x coordinate.
    pub x: f64,
    /// World z coordinate.
    pub z: f64,
    /// Chunk x coordinate.
    pub chunk_x: i32,
    /// Chunk z coordinate.
    pub chunk_z: i32,
}

/// Clientbound packet that tracks, updates or untracks a waypoint.
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Play = C_WAYPOINT)]
pub struct CWaypoint {
    /// Operation to apply.
    pub operation: WaypointOperation,
}

impl WriteTo for CWaypoint {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        match &self.operation {
            WaypointOperation::Track {
                source,
                icon,
                color,
                waypoint,
            } => {
                VarInt(0).write(writer)?;
                source.write(writer)?;
                VarInt(*icon).write(writer)?;
                color.write(writer)?;
                write_waypoint(waypoint, writer)
            }
            WaypointOperation::Untrack { source } => {
                VarInt(1).write(writer)?;
                source.write(writer)
            }
            WaypointOperation::Update { source, waypoint } => {
                VarInt(2).write(writer)?;
                source.write(writer)?;
                write_waypoint(waypoint, writer)
            }
        }
    }
}

fn write_waypoint(
    waypoint: &WaypointData,
    writer: &mut impl std::io::Write,
) -> std::io::Result<()> {
    waypoint.uuid.write(writer)?;
    waypoint.x.write(writer)?;
    waypoint.z.write(writer)?;
    VarInt(waypoint.chunk_x).write(writer)?;
    VarInt(waypoint.chunk_z).write(writer)
}
