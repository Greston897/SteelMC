//! Minecart move-along-track packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_MOVE_MINECART_ALONG_TRACK;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;

/// One step of the client-side interpolation path.
#[derive(Debug, Clone, WriteTo)]
pub struct MinecartStep {
    /// Step position.
    pub position: glam::DVec3,
    /// Step velocity.
    pub movement: glam::DVec3,
    /// Step yaw in degrees.
    pub yaw: f32,
    /// Step pitch in degrees.
    pub pitch: f32,
}

/// Clientbound packet that moves a minecart along rails smoothly.
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Play = C_MOVE_MINECART_ALONG_TRACK)]
pub struct CMoveMinecartAlongTrack {
    /// Minecart entity id.
    #[write(as = VarInt)]
    pub entity_id: i32,
    /// Interpolation steps.
    pub steps: Vec<MinecartStep>,
}

impl WriteTo for CMoveMinecartAlongTrack {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        VarInt(self.entity_id).write(writer)?;
        VarInt(self.steps.len() as i32).write(writer)?;
        for step in &self.steps {
            write_position(writer, step.position)?;
            write_position(writer, step.movement)?;
            step.yaw.write(writer)?;
            step.pitch.write(writer)?;
        }
        Ok(())
    }
}

fn write_position(
    writer: &mut impl std::io::Write,
    value: glam::DVec3,
) -> std::io::Result<()> {
    value.x.write(writer)?;
    value.y.write(writer)?;
    value.z.write(writer)?;
    Ok(())
}
