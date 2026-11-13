//! Behaviors for the technical blocks that only own a block entity.
//!
//! Command blocks, jigsaw blocks, structure blocks and the sculk family have
//! server side state but no player-facing container: their behavior supplies the
//! placement state and the block entity factory, and the rest is driven by the
//! block entity and the redstone/game-event systems.

use std::sync::{Arc, Weak};

use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{BlockStateProperties, FrontAndTop};
use steel_registry::{vanilla_block_entity_types, vanilla_damage_types};
use steel_utils::{BlockPos, BlockStateId};

use crate::behavior::block::{BlockBehavior, BlockEntityCreation};
use crate::entity::damage::DamageSource;
use crate::entity::{Entity as _, LivingEntity as _};
use crate::behavior::context::{BlockPlaceContext, InteractionResult};
use crate::block_entity::BLOCK_ENTITIES;
use crate::world::World;

macro_rules! block_entity_block {
    ($name:ident, $type_const:ident, $placement:expr) => {
        #[block_behavior]
        pub struct $name {
            block: BlockRef,
        }

        impl $name {
            /// Creates a new block behavior.
            #[must_use]
            pub const fn new(block: BlockRef) -> Self {
                Self { block }
            }
        }

        impl BlockBehavior for $name {
            fn get_state_for_placement(
                &self,
                context: &BlockPlaceContext<'_>,
            ) -> Option<BlockStateId> {
                let state = self.block.default_state();
                Some($placement(state, context))
            }

            fn new_block_entity(
                &self,
                level: Weak<World>,
                pos: BlockPos,
                state: BlockStateId,
            ) -> BlockEntityCreation {
                BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
                    &vanilla_block_entity_types::$type_const,
                    level,
                    pos,
                    state,
                ))
            }
        }
    };
}

/// Maps a clicked facing onto vanilla's `front_and_top_to_orientation` for a
/// jigsaw placed against that face.
const fn orientation_for(front: steel_utils::Direction) -> FrontAndTop {
    use steel_utils::Direction;
    match front {
        Direction::Down => FrontAndTop::DownEast,
        Direction::Up => FrontAndTop::UpEast,
        Direction::North => FrontAndTop::NorthUp,
        Direction::South => FrontAndTop::SouthUp,
        Direction::West => FrontAndTop::WestUp,
        Direction::East => FrontAndTop::EastUp,
    }
}

/// Vanilla `CommandBlock` and its repeating/chain variants.
#[block_behavior]
pub struct CommandBlock {
    block: BlockRef,
}

impl CommandBlock {
    /// Creates a new command block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for CommandBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        // Vanilla faces the command block toward the player and starts unpowered.
        Some(
            self.block
                .default_state()
                .set_value(
                    &BlockStateProperties::FACING,
                    context.clicked_face().opposite(),
                )
                .set_value(&BlockStateProperties::CONDITIONAL, false),
        )
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::COMMAND_BLOCK,
            level,
            pos,
            state,
        ))
    }
}

/// Vanilla `JigsawBlock`.
#[block_behavior]
pub struct JigsawBlock {
    block: BlockRef,
}

impl JigsawBlock {
    /// Creates a new jigsaw block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for JigsawBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        // Vanilla `JigsawBlock.getStateForPlacement` derives `orientation` from
        // the clicked face (front) with `north_up` as the top.
        Some(
            self.block
                .default_state()
                .set_value(
                    &BlockStateProperties::ORIENTATION,
                    orientation_for(context.clicked_face().opposite()),
                ),
        )
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::JIGSAW,
            level,
            pos,
            state,
        ))
    }
}

block_entity_block!(
    StructureBlock,
    STRUCTURE_BLOCK,
    |state: BlockStateId, _context: &BlockPlaceContext<'_>| state
);
block_entity_block!(
    CreakingHeartBlock,
    CREAKING_HEART,
    |state: BlockStateId, _context: &BlockPlaceContext<'_>| state
);

impl CreakingHeartBlock {
    /// Vanilla `CreakingHeartBlockEntity.removeCreaking`: kills the linked
    /// creaking when the heart is destroyed.
    fn kill_creaking(world: &Arc<World>, pos: BlockPos) {
        let Some(block_entity) = world.get_block_entity(pos) else {
            return;
        };
        let Some(heart) = block_entity
            .downcast_ref::<crate::block_entity::entities::CreakingHeartBlockEntity>()
        else {
            return;
        };
        let Some(creaking_uuid) = heart.creaking() else {
            return;
        };
        heart.set_creaking(None);
        if let Some(entity) = world.get_entity_by_uuid(&uuid::Uuid::from_u128(creaking_uuid)) {
            let damage = DamageSource::environment(&vanilla_damage_types::GENERIC_KILL);
            entity.hurt(world, &damage, f32::MAX);
        }
    }
}

impl BlockBehavior for CreakingHeartBlock {
    /// Vanilla `CreakingHeartBlock`: breaking the heart takes its creaking with
    /// it, which is the only way to kill a creaking.
    fn on_place(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        old_state: BlockStateId,
        _moved_by_piston: bool,
    ) {
        if old_state.get_block() == state.get_block() {
            return;
        }
        if old_state.get_block() == &self.block {
            Self::kill_creaking(world, pos);
        }
    }
}
block_entity_block!(
    TrialSpawnerBlock,
    TRIAL_SPAWNER,
    |state: BlockStateId, _context: &BlockPlaceContext<'_>| state
);
block_entity_block!(
    VaultBlock,
    VAULT,
    |state: BlockStateId, context: &BlockPlaceContext<'_>| state.set_value(
        &BlockStateProperties::HORIZONTAL_FACING,
        context.horizontal_direction().opposite()
    )
);
block_entity_block!(
    SculkSensorBlock,
    SCULK_SENSOR,
    |state: BlockStateId, _context: &BlockPlaceContext<'_>| state
);
block_entity_block!(
    CalibratedSculkSensorBlock,
    CALIBRATED_SCULK_SENSOR,
    |state: BlockStateId, context: &BlockPlaceContext<'_>| state.set_value(
        &BlockStateProperties::HORIZONTAL_FACING,
        context.horizontal_direction().opposite()
    )
);
block_entity_block!(
    SculkCatalystBlock,
    SCULK_CATALYST,
    |state: BlockStateId, _context: &BlockPlaceContext<'_>| state
);
block_entity_block!(
    SculkShriekerBlock,
    SCULK_SHRIEKER,
    |state: BlockStateId, _context: &BlockPlaceContext<'_>| state
);

/// Vanilla `BonemealableFeaturePlacerBlock` (moss and pale moss blocks): bonemeal
/// places a vegetation feature around the block.
///
/// The feature placement itself lives in `BonemealableFeaturePlacer`; Steel has
/// no moss-vegetation feature wired yet, so the behavior only carries the
/// placement state and leaves the feature call to the bonemeal dispatcher.
#[block_behavior]
pub struct BonemealableFeaturePlacerBlock {
    block: BlockRef,
}

impl BonemealableFeaturePlacerBlock {
    /// Creates a new moss block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for BonemealableFeaturePlacerBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }
}
