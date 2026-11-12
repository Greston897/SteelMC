//! Set structure block packet.

use steel_macros::{ReadFrom, ServerPacket};
use steel_utils::BlockPos;
use steel_utils::codec::{VarInt, VarLong};

/// Serverbound packet that edits a structure block.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SSetStructureBlock {
    /// Structure block position.
    pub pos: BlockPos,
    /// Vanilla `StructureBlockEntity.Action` ordinal.
    #[read(as = VarInt)]
    pub action: i32,
    /// Mode ordinal (`save`, `load`, `corner`, `data`).
    #[read(as = VarInt)]
    pub mode: i32,
    /// Structure name.
    pub name: String,
    /// Offset of the structure corner.
    pub offset: BlockPos,
    /// Size of the structure box.
    pub size: BlockPos,
    /// Mirror ordinal (`none`, `left_right`, `front_back`).
    #[read(as = VarInt)]
    pub mirror: i32,
    /// Rotation ordinal (`none`, `clockwise_90`, `180`, `counterclockwise_90`).
    #[read(as = VarInt)]
    pub rotation: i32,
    /// Metadata string stored in the structure.
    pub metadata: String,
    /// Integrity in `0.0..=1.0`.
    pub integrity: f32,
    /// Seed used to randomize blocks.
    #[read(as = VarLong)]
    pub seed: i64,
    /// Whether the structure ignores entities.
    pub ignore_entities: bool,
    /// Whether the structure shows the bounding box.
    pub show_bounding_box: bool,
    /// Whether the bounding box is shown in air.
    pub show_air: bool,
    /// Whether the blocks are currently being written to disk.
    pub strict: bool,
}
