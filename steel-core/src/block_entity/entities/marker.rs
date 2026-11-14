//! Marker block entities without server-side state.
//!
//! Vanilla allocates block entities for these blocks even though the server keeps
//! nothing beyond the block state: the client needs the block entity to render
//! them, and the world format still writes their (empty) NBT. Modelling them
//! explicitly keeps the registry complete instead of falling back to the generic
//! `UnimplementedBlockEntity`.
//!
//! This module is deliberately not a home for blocks whose server side matters.
//! A marker drops every NBT field the client would need (`/setblock` on a
//! command block, a jigsaw or a structure block writes data that is then lost),
//! so any block that gains server behavior must get a real block entity instead.

use std::sync::{Arc, Weak};

use steel_registry::block_entity_type::BlockEntityTypeRef;
use steel_registry::vanilla_block_entity_types;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey};

use simdnbt::borrow::BaseNbtCompound as BorrowedNbtCompound;
use simdnbt::owned::NbtCompound;

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::world::World;

macro_rules! marker_block_entity {
    ($(#[$meta:meta])* $name:ident, $key:literal, $type_const:ident) => {
        $(#[$meta])*
        /// Marker block entity: no extra server-side state.
        pub struct $name {
            base: Arc<BlockEntityBase>,
        }

        // SAFETY: This key is owned by Steel and uniquely identifies this type.
        unsafe impl DowncastType for $name {
            const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new($key);
        }

        impl $name {
            /// Creates a new block entity.
            #[must_use]
            pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
                Self::new_with_type(&vanilla_block_entity_types::$type_const, level, pos, state)
            }

            /// Creates a new block entity for the given registered type.
            #[must_use]
            pub fn new_with_type(
                block_entity_type: BlockEntityTypeRef,
                level: Weak<World>,
                pos: BlockPos,
                state: BlockStateId,
            ) -> Self {
                Self {
                    base: Arc::new(BlockEntityBase::new(block_entity_type, level, pos, state)),
                }
            }
        }

        impl BlockEntity for $name {
            fn base(&self) -> &BlockEntityBase {
                &self.base
            }

            fn load_additional(&self, _nbt: &BorrowedNbtCompound<'_>) {}

            fn save_additional(&self, _nbt: &mut NbtCompound) {}
        }
    };
}

marker_block_entity!(
    JigsawBlockEntity,
    "steel:block_entity/jigsaw",
    JIGSAW
);
marker_block_entity!(
    /// Vanilla `CommandBlockEntity` runs commands from its block entity; this
    /// marker persists nothing, so `/setblock` on a command block currently
    /// yields a block that never executes.
    ///
    /// The `commandBlocksWork` and `commandBlockOutput` game rules depend on that
    /// executor and are therefore declared but unread.
    CommandBlockEntity,
    "steel:block_entity/command_block",
    COMMAND_BLOCK
);
marker_block_entity!(
    StructureBlockEntity,
    "steel:block_entity/structure_block",
    STRUCTURE_BLOCK
);
marker_block_entity!(
    CopperGolemStatueBlockEntity,
    "steel:block_entity/copper_golem_statue",
    COPPER_GOLEM_STATUE
);
marker_block_entity!(
    TestBlockEntity,
    "steel:block_entity/test_block",
    TEST_BLOCK
);
marker_block_entity!(
    TestInstanceBlockEntity,
    "steel:block_entity/test_instance_block",
    TEST_INSTANCE_BLOCK
);
