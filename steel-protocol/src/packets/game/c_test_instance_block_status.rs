//! Game-test block status packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_utils::serial::PrefixedWrite as _;
use steel_utils::serial::WriteTo as _;
use steel_registry::packets::play::C_TEST_INSTANCE_BLOCK_STATUS;
use steel_utils::BlockPos;
use steel_utils::codec::VarInt;

/// One block status entry of a test instance.
#[derive(WriteTo, Clone, Debug)]
pub struct TestBlockStatus {
    /// Block position.
    pub pos: BlockPos,
    /// Status message shown above the block.
    pub message: String,
}

/// Clientbound packet that reports the state of test-instance blocks.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_TEST_INSTANCE_BLOCK_STATUS)]
pub struct CTestInstanceBlockStatus {
    /// Status text shown in the test screen.
    pub status: String,
    /// Size of the tested structure, or `None`.
    pub size: Option<BlockPos>,
    /// Per-block status entries.
    #[write(as = Prefixed(VarInt))]
    pub blocks: Vec<TestBlockStatus>,
}
