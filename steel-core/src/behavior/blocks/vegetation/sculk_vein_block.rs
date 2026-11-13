use std::sync::Arc;

use rand::Rng;
use steel_macros::block_behavior;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::blocks::properties::Direction;
use steel_registry::vanilla_blocks;
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId};

use crate::behavior::block::BlockBehavior;
use crate::behavior::blocks::MultifaceBlock;
use crate::behavior::context::BlockPlaceContext;
use crate::world::{LevelReader, ScheduledTickAccess, World};

use super::BlockRef;

/// Vanilla `SculkVeinBlock` survival, spread, and catalyst charge handling.
///
/// Vanilla splits the charge logic between `SculkVeinBlock` (a vein that carries
/// a sculk charge hides it in a block entity) and `SculkSpreader`. Steel models
/// the runtime side here: a randomly-ticked vein spreads onto adjacent faces and,
/// when it sits inside a sculk patch near a catalyst, converts into plain sculk.
///
/// Survival and shape updates come from the shared `MultifaceBlock` implementation
/// — the vein is a multiface block, so its support rules are identical.
#[block_behavior]
pub struct SculkVeinBlock {
    multiface: MultifaceBlock,
}

impl SculkVeinBlock {
    /// All six directions, matching vanilla `Direction.values()`.
    const DIRECTIONS: [Direction; 6] = [
        Direction::Down,
        Direction::Up,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ];

    /// Creates a new sculk vein block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self {
            multiface: MultifaceBlock::new(block),
        }
    }

    /// Vanilla `SculkVeinBlock.spreadFromVein`: grows a new vein on the far side of
    /// every other face, which is what makes veins creep along cave walls.
    fn spread_from_vein(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        rng: &mut impl Rng,
    ) -> bool {
        let mut spread = false;
        let mut directions = Self::DIRECTIONS;
        // Vanilla shuffles the directions so growth is not biased to one side.
        shuffle(&mut directions, rng);

        for direction in directions {
            if !MultifaceBlock::has_face(state, direction) {
                continue;
            }
            let support_pos = pos.relative(direction);
            if !world.get_block_state(support_pos).is_solid_render() {
                continue;
            }

            // Vanilla `SculkVeinBlock.spreadFromFaceTowardRandomDirection`: try a
            // few times per supported face with a random spread direction.
            for _ in 0..4 {
                let spread_direction = Self::DIRECTIONS[rng.random_range(0..6)];
                if self
                    .multiface
                    .spread_from_face_toward_direction(
                        state,
                        world,
                        pos,
                        direction,
                        spread_direction,
                    )
                    .is_some()
                {
                    spread = true;
                    break;
                }
            }
        }

        spread
    }

    /// Vanilla `SculkVeinBlock.regrowVeins`: fills every free face of this position
    /// with a vein once the block underneath is sculk-replaceable.
    fn regrow_veins(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        rng: &mut impl Rng,
    ) -> bool {
        let mut regrown = false;
        let mut directions = Self::DIRECTIONS;
        shuffle(&mut directions, rng);
        for direction in directions {
            if MultifaceBlock::has_face(state, direction) {
                continue;
            }
            // Only faces backed by a solid neighbour can hold a vein.
            if !world
                .get_block_state(pos.relative(direction))
                .is_solid_render()
            {
                continue;
            }
            let spread_direction = Self::DIRECTIONS[rng.random_range(0..6)];
            if self
                .multiface
                .spread_from_face_toward_direction(
                    state,
                    world,
                    pos,
                    direction,
                    spread_direction,
                )
                .is_some()
            {
                regrown = true;
            }
        }
        regrown
    }

    /// Vanilla `SculkVeinBlock.attemptUseCharge`: a vein next to plain sculk inside
    /// a patch converts into sculk, which is how catalysts "grow" solid sculk.
    fn attempt_use_charge(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        rng: &mut impl Rng,
    ) -> bool {
        // Vanilla only converts when the vein touches sculk on the same plane.
        for direction in Self::DIRECTIONS {
            if !MultifaceBlock::has_face(state, direction) {
                continue;
            }
            let neighbor = world.get_block_state(pos.relative(direction));
            if neighbor.get_block() != &vanilla_blocks::SCULK {
                continue;
            }
            if rng.random_range(0..2) == 0 {
                world.set_block(
                    pos,
                    vanilla_blocks::SCULK.default_state(),
                    UpdateFlags::UPDATE_ALL,
                );
                return true;
            }
        }
        false
    }
}

/// In-place Fisher–Yates shuffle, matching vanilla's `Util.shuffle`.
fn shuffle(directions: &mut [Direction; 6], rng: &mut impl Rng) {
    for index in (1..directions.len()).rev() {
        let swap_with = rng.random_range(0..=index);
        directions.swap(index, swap_with);
    }
}

impl BlockBehavior for SculkVeinBlock {
    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        direction: Direction,
        neighbor_pos: BlockPos,
        neighbor_state: BlockStateId,
    ) -> BlockStateId {
        self.multiface
            .update_shape(state, world, pos, direction, neighbor_pos, neighbor_state)
    }

    fn can_survive(&self, state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        self.multiface.can_survive(state, world, pos)
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        self.multiface.get_state_for_placement(context)
    }

    fn can_be_replaced(&self, state: BlockStateId, context: &BlockPlaceContext<'_>) -> bool {
        self.multiface.can_be_replaced(state, context)
    }

    fn random_tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        let mut rng = rand::rng();
        // Vanilla `SculkVeinBlock.attemptSpreadVein` also runs on the same tick as
        // the charge attempt; both are cheap and idempotent when nothing changes.
        self.spread_from_vein(state, world, pos, &mut rng);
        if !self.attempt_use_charge(state, world, pos, &mut rng) {
            self.regrow_veins(state, world, pos, &mut rng);
        }
    }
}
