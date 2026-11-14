//! Test instance block action packet.

use std::io::{Cursor, Result, Write};

use steel_macros::ServerPacket;
use steel_utils::serial::{ReadFrom, WriteTo};
use steel_utils::{BlockPos, Identifier, Rotation, codec::VarInt};
use text_components::TextComponent;

/// Vanilla `TestInstanceBlockEntity.Status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestInstanceBlockSetupStatus {
    /// No test run yet.
    Cleared,
    /// A test is running.
    Running,
    /// The run finished.
    Finished,
}

impl TestInstanceBlockSetupStatus {
    /// Vanilla `Status.STREAM_CODEC` index.
    #[must_use]
    pub const fn index(self) -> i32 {
        match self {
            Self::Cleared => 0,
            Self::Running => 1,
            Self::Finished => 2,
        }
    }

    /// Vanilla `Status.byIndex`; out-of-range ids map to `CLEARED`.
    #[must_use]
    pub const fn from_index(index: i32) -> Self {
        match index {
            1 => Self::Running,
            2 => Self::Finished,
            _ => Self::Cleared,
        }
    }
}

/// Vanilla `TestInstanceBlockEntity.Data`: the whole editor state of a test
/// instance block. It travels both in packets and in the block entity's NBT.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestInstanceBlockData {
    /// The test instance this block runs, when one is selected.
    pub test: Option<Identifier>,
    /// Structure size, in blocks.
    pub size: glam::IVec3,
    /// Structure rotation.
    pub rotation: Rotation,
    /// Whether entities are ignored when the structure is placed.
    pub ignore_entities: bool,
    /// Current run status.
    pub status: TestInstanceBlockSetupStatus,
    /// Failure text shown in the editor, when the run failed.
    pub error_message: Option<TextComponent>,
}

impl Default for TestInstanceBlockData {
    fn default() -> Self {
        Self {
            test: None,
            size: glam::IVec3::ZERO,
            rotation: Rotation::None,
            ignore_entities: false,
            status: TestInstanceBlockSetupStatus::Cleared,
            error_message: None,
        }
    }
}

/// Vanilla `Rotation` wire index.
const fn rotation_index(rotation: Rotation) -> i32 {
    match rotation {
        Rotation::None => 0,
        Rotation::Clockwise90 => 1,
        Rotation::Clockwise180 => 2,
        Rotation::CounterClockwise90 => 3,
    }
}

/// Vanilla `Rotation` wire index; out-of-range ids map to `NONE`.
const fn rotation_from_index(index: i32) -> Rotation {
    match index {
        1 => Rotation::Clockwise90,
        2 => Rotation::Clockwise180,
        3 => Rotation::CounterClockwise90,
        _ => Rotation::None,
    }
}

impl WriteTo for TestInstanceBlockData {
    fn write(&self, writer: &mut impl Write) -> Result<()> {
        // `ByteBufCodecs.optional(ResourceKey.streamCodec(Registries.TEST_INSTANCE))`
        // is a presence byte followed by the identifier.
        match &self.test {
            Some(test) => {
                true.write(writer)?;
                test.write(writer)?;
            }
            None => false.write(writer)?,
        }
        // `Vec3i.STREAM_CODEC` is three VarInts.
        VarInt(self.size.x).write(writer)?;
        VarInt(self.size.y).write(writer)?;
        VarInt(self.size.z).write(writer)?;
        VarInt(rotation_index(self.rotation)).write(writer)?;
        self.ignore_entities.write(writer)?;
        VarInt(self.status.index()).write(writer)?;
        match &self.error_message {
            Some(message) => {
                true.write(writer)?;
                message.write(writer)?;
            }
            None => false.write(writer)?,
        }
        Ok(())
    }
}

impl ReadFrom for TestInstanceBlockData {
    fn read(data: &mut Cursor<&[u8]>) -> Result<Self> {
        let test = if bool::read(data)? {
            Some(Identifier::read(data)?)
        } else {
            None
        };
        let size = glam::IVec3 {
            x: VarInt::read(data)?.0,
            y: VarInt::read(data)?.0,
            z: VarInt::read(data)?.0,
        };
        let rotation = rotation_from_index(VarInt::read(data)?.0);
        let ignore_entities = bool::read(data)?;
        let status = TestInstanceBlockSetupStatus::from_index(VarInt::read(data)?.0);
        let error_message = if bool::read(data)? {
            Some(TextComponent::read(data)?)
        } else {
            None
        };
        Ok(Self {
            test,
            size,
            rotation,
            ignore_entities,
            status,
            error_message,
        })
    }
}

/// Vanilla `ServerboundTestInstanceBlockActionPacket.Action`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestInstanceBlockAction {
    /// The client asks the server to open the editor.
    Init,
    /// The client asks for the status of the selected test.
    Query,
    /// The client stores new editor data.
    Set,
    /// Reset the structure to its template.
    Reset,
    /// Save the structure back into the test template.
    Save,
    /// Export the structure to disk.
    Export,
    /// Run the test.
    Run,
}

impl TestInstanceBlockAction {
    /// Vanilla `Action` wire id.
    #[must_use]
    pub const fn id(self) -> i32 {
        match self {
            Self::Init => 0,
            Self::Query => 1,
            Self::Set => 2,
            Self::Reset => 3,
            Self::Save => 4,
            Self::Export => 5,
            Self::Run => 6,
        }
    }

    /// Vanilla `ByIdMap.continuous(..., OutOfBoundsStrategy.ZERO)`, so unknown
    /// ids degrade to `INIT`.
    #[must_use]
    pub const fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Query,
            2 => Self::Set,
            3 => Self::Reset,
            4 => Self::Save,
            5 => Self::Export,
            6 => Self::Run,
            _ => Self::Init,
        }
    }

    /// Actions that only read data never store the payload.
    #[must_use]
    pub const fn is_read_only(self) -> bool {
        matches!(self, Self::Query | Self::Init)
    }
}

/// Serverbound packet that drives a test instance block.
#[derive(ServerPacket, Clone, Debug)]
pub struct STestInstanceBlockAction {
    /// Test instance block position.
    pub pos: BlockPos,
    /// Requested action.
    pub action: TestInstanceBlockAction,
    /// Editor payload stored by [`TestInstanceBlockAction::Set`] and friends.
    pub data: TestInstanceBlockData,
}

impl ReadFrom for STestInstanceBlockAction {
    fn read(data: &mut Cursor<&[u8]>) -> Result<Self> {
        let pos = BlockPos::read(data)?;
        let action = TestInstanceBlockAction::from_id(VarInt::read(data)?.0);
        let data = TestInstanceBlockData::read(data)?;
        Ok(Self { pos, action, data })
    }
}
