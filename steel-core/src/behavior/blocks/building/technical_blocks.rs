//! Behaviors for the technical blocks that only own a block entity.
//!
//! Command blocks, jigsaw blocks, structure blocks and the sculk family have
//! server side state but no player-facing container: their behavior supplies the
//! placement state and the block entity factory, and the rest is driven by the
//! block entity and the redstone/game-event systems.

use std::sync::{Arc, Weak};

use rand::{Rng, RngExt};
use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{BlockStateProperties, FrontAndTop};
use steel_registry::{vanilla_block_entity_types, vanilla_blocks, vanilla_damage_types};
use steel_utils::Downcast as _;
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId};

use crate::behavior::BLOCK_BEHAVIORS;
use crate::behavior::block::{BlockBehavior, BlockEntityCreation};
use crate::behavior::blocks::vegetation::bonemealable::Bonemealable;
use crate::entity::damage::DamageSource;
use crate::world::LevelReader;
use crate::entity::{Entity as _, LivingEntity as _};
use crate::behavior::context::{BlockPlaceContext, InteractionResult};
use crate::block_entity::BLOCK_ENTITIES;
use crate::world::World;

macro_rules! block_entity_block {
    ($name:ident, $type_const:ident, $placement:expr $(, $hook:item)?) => {
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

            $( $hook )?
        }
    };
}

/// Declares a block-entity block whose `on_place` runs `$hook` when the old
/// state belonged to a different block.
macro_rules! block_entity_block_with_on_place {
    ($name:ident, $type_const:ident, $placement:expr, $hook:expr) => {
        block_entity_block!(
            $name,
            $type_const,
            $placement,
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
                ($hook)(world, pos, &self);
            }
        );
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
/// Vanilla `CreakingHeartBlockEntity.removeCreaking`: destroying the heart takes
/// its creaking with it, which is the only way to kill a creaking.
fn kill_linked_creaking(world: &Arc<World>, pos: BlockPos) {
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

block_entity_block_with_on_place!(
    CreakingHeartBlock,
    CREAKING_HEART,
    |state: BlockStateId, _context: &BlockPlaceContext<'_>| state,
    |world, pos, _behavior| kill_linked_creaking(world, pos)
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

/// Attempts vanilla's moss feature makes to find a valid position.
const MOSS_PLACEMENT_ATTEMPTS: usize = 8;

/// Vanilla `BonemealableFeaturePlacerBlock` (moss and pale moss blocks): bonemeal
/// scatters the block's vegetation feature over the surrounding blocks.
///
/// Vanilla reads the feature key from the block's `FEATURE` data component
/// (`moss_vegetation` for moss, `pale_moss_vegetation` for pale moss) and asks
/// the feature system to place it inside a 3x3x3 area, skipping the source block.
/// Steel places the simple vegetation set directly: vanilla's features are thin
/// wrappers that put one random plant on a random supported position.
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

    fn as_bonemealable(&self) -> Option<&dyn Bonemealable> {
        Some(self)
    }
}

impl Bonemealable for BonemealableFeaturePlacerBlock {
    fn is_valid_bonemeal_target(
        &self,
        _state: BlockStateId,
        world: &dyn LevelReader,
        pos: BlockPos,
    ) -> bool {
        // Vanilla `BonemealableFeaturePlacerBlock.isValidBonemealTarget`: any
        // position with room for the feature is fine; the feature's own placement
        // check decides the rest.
        world.get_block_state(pos.above()).is_air()
    }

    fn perform_bonemeal(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        rng: &mut dyn rand::Rng,
        pos: BlockPos,
    ) {
        // Vanilla's moss feature places a *single* random plant: the configured
        // feature picks one weighted entry and tries to put it on a random
        // position inside the 3x3x3 area around the block.
        let plant = self.vegetation_block(rng);
        let plant_state = plant.default_state();
        let behavior = BLOCK_BEHAVIORS.get_behavior(plant);
        for _ in 0..MOSS_PLACEMENT_ATTEMPTS {
            let dx = rng.random_range(-1..=1);
            let dy = rng.random_range(-1..=1);
            let dz = rng.random_range(-1..=1);
            let candidate = pos.offset(dx, dy, dz);
            if candidate == pos {
                continue;
            }
            if world.get_block_state(candidate).is_air()
                && behavior.can_survive(plant_state, world, candidate)
            {
                world.set_block(candidate, plant_state, UpdateFlags::UPDATE_ALL);
                return;
            }
        }
    }
}

impl BonemealableFeaturePlacerBlock {
    /// Vanilla `moss_vegetation` / `pale_moss_vegetation` feature contents: the
    /// weighted plant set the feature scatters.
    fn vegetation_block(&self, rng: &mut dyn Rng) -> BlockRef {
        let pale = self.block.key == vanilla_blocks::PALE_MOSS_BLOCK.key;
        // `vanilla_blocks::X` are already `BlockRef` (`&'static Block`), so the
        // array holds the references directly rather than taking them again.
        let choices: [BlockRef; 4] = if pale {
            [
                vanilla_blocks::PALE_HANGING_MOSS,
                vanilla_blocks::SHORT_GRASS,
                vanilla_blocks::PALE_MOSS_CARPET,
                vanilla_blocks::TALL_GRASS,
            ]
        } else {
            [
                vanilla_blocks::SHORT_GRASS,
                vanilla_blocks::MOSS_CARPET,
                vanilla_blocks::TALL_GRASS,
                vanilla_blocks::AZALEA,
            ]
        };
        let index = rng.random_range(0..choices.len());
        choices[index]
    }
}
