//! Set structure block packet.
//!
//! Wire format parity with vanilla `ServerboundSetStructureBlockPacket`: after
//! the name the client writes six *single bytes* (offset xyz then size xyz) and
//! the four booleans travel packed into one trailing flags byte. Vanilla clamps
//! both ranges on read; Steel keeps the raw bytes and clamps in the accessors so
//! the packet stays a faithful mirror of the wire.

use steel_macros::{ReadFrom, ServerPacket};
use steel_utils::BlockPos;

/// Vanilla `ServerboundSetStructureBlockPacket.FLAG_IGNORE_ENTITIES`.
const FLAG_IGNORE_ENTITIES: u8 = 1;
/// Vanilla `FLAG_SHOW_AIR`.
const FLAG_SHOW_AIR: u8 = 2;
/// Vanilla `FLAG_SHOW_BOUNDING_BOX`.
const FLAG_SHOW_BOUNDING_BOX: u8 = 4;
/// Vanilla `FLAG_STRICT`.
const FLAG_STRICT: u8 = 8;

/// Vanilla clamps the offset bytes to `-48..=48`.
const MAX_OFFSET: i32 = 48;
/// Vanilla clamps the size bytes to `0..=48`.
const MAX_SIZE: i32 = 48;

/// Serverbound packet that edits a structure block.
#[derive(ReadFrom, ServerPacket, Clone, Debug)]
pub struct SSetStructureBlock {
    /// Structure block position.
    pub pos: BlockPos,
    /// Vanilla `StructureBlockEntity.UpdateType` ordinal.
    #[read(as = VarInt)]
    pub update_type: i32,
    /// Mode ordinal (`save`, `load`, `corner`, `data`).
    #[read(as = VarInt)]
    pub mode: i32,
    /// Structure name.
    pub name: String,
    /// Raw offset byte on the x axis (vanilla clamps it to `-48..=48`).
    pub offset_x: i8,
    /// Raw offset byte on the y axis.
    pub offset_y: i8,
    /// Raw offset byte on the z axis.
    pub offset_z: i8,
    /// Raw size byte on the x axis (vanilla clamps it to `0..=48`).
    pub size_x: i8,
    /// Raw size byte on the y axis.
    pub size_y: i8,
    /// Raw size byte on the z axis.
    pub size_z: i8,
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
    /// Packed vanilla flags: `1` ignore entities, `2` show air,
    /// `4` show bounding box, `8` strict.
    pub flags: u8,
}

impl SSetStructureBlock {
    /// Vanilla `isIgnoreEntities`.
    #[must_use]
    pub const fn ignore_entities(&self) -> bool {
        self.flags & FLAG_IGNORE_ENTITIES != 0
    }

    /// Vanilla `isShowAir`.
    #[must_use]
    pub const fn show_air(&self) -> bool {
        self.flags & FLAG_SHOW_AIR != 0
    }

    /// Vanilla `isShowBoundingBox`.
    #[must_use]
    pub const fn show_bounding_box(&self) -> bool {
        self.flags & FLAG_SHOW_BOUNDING_BOX != 0
    }

    /// Vanilla `isStrict`.
    #[must_use]
    pub const fn strict(&self) -> bool {
        self.flags & FLAG_STRICT != 0
    }

    /// Vanilla `Mth.clamp(readByte(), -48, 48)` for the three offset axes.
    #[must_use]
    pub const fn offset(&self) -> BlockPos {
        BlockPos::new(
            clamp_offset(self.offset_x),
            clamp_offset(self.offset_y),
            clamp_offset(self.offset_z),
        )
    }

    /// Vanilla `Mth.clamp(readByte(), 0, 48)` for the three size axes.
    #[must_use]
    pub const fn size(&self) -> BlockPos {
        BlockPos::new(
            clamp_size(self.size_x),
            clamp_size(self.size_y),
            clamp_size(self.size_z),
        )
    }

    /// Vanilla `getIntegrity`, clamped the same way the reader does.
    #[must_use]
    pub fn integrity_clamped(&self) -> f32 {
        self.integrity.clamp(0.0, 1.0)
    }
}

/// `Mth.clamp(value, -48, 48)` for a signed byte.
const fn clamp_offset(value: i8) -> i32 {
    let value = value as i32;
    if value < -MAX_OFFSET {
        -MAX_OFFSET
    } else if value > MAX_OFFSET {
        MAX_OFFSET
    } else {
        value
    }
}

/// `Mth.clamp(value, 0, 48)` for a signed byte.
const fn clamp_size(value: i8) -> i32 {
    let value = value as i32;
    if value < 0 {
        0
    } else if value > MAX_SIZE {
        MAX_SIZE
    } else {
        value
    }
}
