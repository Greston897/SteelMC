//! Game-test block status packet.

use std::io::{Result, Write};

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_TEST_INSTANCE_BLOCK_STATUS;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;
use text_components::TextComponent;

/// Clientbound packet that reports the state of a test-instance block.
///
/// Vanilla `ClientboundTestInstanceBlockStatus(Component status, Optional<Vec3i> size)`.
#[derive(ClientPacket, Clone, Debug)]
#[packet_id(Play = C_TEST_INSTANCE_BLOCK_STATUS)]
pub struct CTestInstanceBlockStatus {
    /// Status text shown in the test screen.
    pub status: TextComponent,
    /// Size of the tested structure, or `None` when the test has no structure.
    pub size: Option<glam::IVec3>,
}

impl WriteTo for CTestInstanceBlockStatus {
    fn write(&self, writer: &mut impl Write) -> Result<()> {
        self.status.write(writer)?;
        match self.size {
            Some(size) => {
                true.write(writer)?;
                // `Vec3i.STREAM_CODEC` is three VarInts, not the packed BlockPos.
                VarInt(size.x).write(writer)?;
                VarInt(size.y).write(writer)?;
                VarInt(size.z).write(writer)?;
            }
            None => false.write(writer)?,
        }
        Ok(())
    }
}
