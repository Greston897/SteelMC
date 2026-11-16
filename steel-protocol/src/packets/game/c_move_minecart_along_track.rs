//! Minecart move-along-track packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::play::C_MOVE_MINECART_ALONG_TRACK;
use steel_utils::codec::VarInt;
// `as _` would be enough for the `value.write(writer)` calls below, but the
// manual `impl WriteTo for CMoveMinecartAlongTrack` needs the trait in scope by
// name. The derive macro of the same name lives in the macro namespace, so
// both resolve without ambiguity.
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
    /// Vanilla `MinecartStep.weight`, the share of the lerp window this step
    /// covers. `ServerEntity` always sends `1.0`.
    pub weight: f32,
}

impl MinecartStep {
    /// Vanilla `MinecartStep.ZERO`.
    pub const ZERO: Self = Self {
        position: glam::DVec3::ZERO,
        movement: glam::DVec3::ZERO,
        yaw: 0.0,
        pitch: 0.0,
        weight: 0.0,
    };
}

/// Clientbound packet that moves a minecart along rails smoothly.
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Play = C_MOVE_MINECART_ALONG_TRACK)]
pub struct CMoveMinecartAlongTrack {
    /// Minecart entity id.
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
            // Vanilla `ByteBufCodecs.ROTATION_BYTE` for both rotations.
            crate::packets::game::to_angle_byte(step.yaw).write(writer)?;
            crate::packets::game::to_angle_byte(step.pitch).write(writer)?;
            step.weight.write(writer)?;
        }
        Ok(())
    }
}

fn write_position(writer: &mut impl std::io::Write, value: glam::DVec3) -> std::io::Result<()> {
    value.x.write(writer)?;
    value.y.write(writer)?;
    value.z.write(writer)?;
    Ok(())
}
