//! Set jigsaw block packet (`ServerboundSetJigsawBlockPacket`).
//!
//! Vanilla `write` order:
//! 1. `pos`: `BlockPos`
//! 2. `name`: `Identifier`
//! 3. `target`: `Identifier`
//! 4. `pool`: `Identifier`
//! 5. `finalState`: `String`
//! 6. `joint`: `String` (`joint.getSerializedName()`: `"rollable"` or `"aligned"`)
//! 7. `selectionPriority`: `VarInt`
//! 8. `placementPriority`: `VarInt`

use steel_macros::{ReadFrom, ServerPacket};
use steel_utils::{BlockPos, Identifier};

/// Serverbound packet saving a jigsaw block's configuration from the jigsaw screen.
#[derive(ReadFrom, ServerPacket, Clone, Debug, PartialEq, Eq)]
pub struct SSetJigsawBlock {
    /// Position of the jigsaw block.
    pub pos: BlockPos,
    /// Name of this jigsaw connector.
    pub name: Identifier,
    /// Target connector name to join against.
    pub target: Identifier,
    /// Template pool key.
    pub pool: Identifier,
    /// Block state string the jigsaw turns into after generation.
    pub final_state: String,
    /// Serialized `JigsawBlockEntity.JointType` (`"rollable"` or `"aligned"`).
    pub joint: String,
    /// Selection priority (`VarInt`).
    #[read(as = VarInt)]
    pub selection_priority: i32,
    /// Placement priority (`VarInt`).
    #[read(as = VarInt)]
    pub placement_priority: i32,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::SSetJigsawBlock;
    use steel_utils::codec::VarInt;
    use steel_utils::serial::{ReadFrom as _, WriteTo as _};
    use steel_utils::{BlockPos, Identifier};

    #[test]
    fn reads_string_joint_before_selection_and_placement_priorities() {
        let mut wire = Vec::new();
        BlockPos::new(1, 70, -3).write(&mut wire).unwrap();
        "minecraft:top".to_owned().write(&mut wire).unwrap();
        "minecraft:bottom".to_owned().write(&mut wire).unwrap();
        "minecraft:village/plains/houses"
            .to_owned()
            .write(&mut wire)
            .unwrap();
        "minecraft:cobblestone".to_owned().write(&mut wire).unwrap();
        "aligned".to_owned().write(&mut wire).unwrap();
        VarInt(4).write(&mut wire).unwrap();
        VarInt(9).write(&mut wire).unwrap();

        let pkt = SSetJigsawBlock::read(&mut Cursor::new(&wire)).unwrap();
        assert_eq!(pkt.pos, BlockPos::new(1, 70, -3));
        assert_eq!(pkt.name, Identifier::vanilla_static("top"));
        assert_eq!(pkt.target, Identifier::vanilla_static("bottom"));
        assert_eq!(pkt.final_state, "minecraft:cobblestone");
        assert_eq!(pkt.joint, "aligned");
        assert_eq!(pkt.selection_priority, 4);
        assert_eq!(pkt.placement_priority, 9);
    }
}
