use std::sync::Arc;

use steel_macros::block_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::{sound_events, vanilla_block_tags::BlockTag, vanilla_blocks};
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId};

use crate::behavior::block::BlockBehavior;
use crate::behavior::context::BlockPlaceContext;
use crate::world::{LevelReader, World};

use super::{BlockRef, default_surviving_state, survives_on_tag};

#[derive(Clone, Copy)]
/// Vanilla open/closed eyeblossom type from `classes.json`.
pub enum EyeblossomType {
    /// Emits open-eyeblossom effects and transforms closed at daytime.
    Open,
    /// Emits closed-eyeblossom effects and transforms open at nighttime.
    Closed,
}

/// Vanilla `EyeblossomBlock` survival, day/night transform, and effects.
///
/// Vanilla samples `EnvironmentAttributes.EYEBLOSSOM_OPEN` (a boolean timeline
/// track) and swaps the open/closed block with the timeline's own sound; Steel
/// resolves the attribute through [`World::environment_attribute`] and treats a
/// value below `0.5` as closed.
#[block_behavior]
pub struct EyeblossomBlock {
    block: BlockRef,
    #[json_arg(r#enum = "EyeblossomType", json = "type")]
    eyeblossom_type: EyeblossomType,
}

impl EyeblossomBlock {
    /// Creates a new eyeblossom behavior.
    #[must_use]
    pub const fn new(block: BlockRef, eyeblossom_type: EyeblossomType) -> Self {
        Self {
            block,
            eyeblossom_type,
        }
    }
}

impl BlockBehavior for EyeblossomBlock {
    fn can_survive(&self, _state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        survives_on_tag(world, pos, &BlockTag::SUPPORTS_VEGETATION)
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        default_surviving_state(self.block, self, context)
    }

    fn random_tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        self.transform_if_needed(state, world, pos);
    }

    fn tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        self.transform_if_needed(state, world, pos);
    }
}

impl EyeblossomBlock {
    /// Vanilla `EyeblossomBlock.tick`: swap to the other flower when the
    /// `eyeblossom_open` environment attribute disagrees with this block's state.
    fn transform_if_needed(&self, _state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        let should_be_open = world.environment_attribute(EYEBLOSSOM_OPEN_ATTRIBUTE, 0.0) >= 0.5;
        let is_open = matches!(self.eyeblossom_type, EyeblossomType::Open);
        if should_be_open == is_open {
            return;
        }

        let (target, sound) = if should_be_open {
            (&vanilla_blocks::OPEN_EYEBLOSSOM, &sound_events::BLOCK_EYEBLOSSOM_OPEN)
        } else {
            (&vanilla_blocks::CLOSED_EYEBLOSSOM, &sound_events::BLOCK_EYEBLOSSOM_CLOSE)
        };
        world.set_block(pos, target.default_state(), UpdateFlags::UPDATE_ALL);
        world.play_sound(sound, SoundSource::Blocks, pos, 1.0, 1.0, None);
    }
}

/// Vanilla `EnvironmentAttributes.EYEBLOSSOM_OPEN` key.
const EYEBLOSSOM_OPEN_ATTRIBUTE: &str = "minecraft:gameplay/eyeblossom_open";


#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_blocks};
    use steel_utils::BlockPos;

    use crate::test_support::TestLevel;

    use super::*;

    fn level_with_support(support: BlockRef) -> TestLevel {
        TestLevel::default().with_block(BlockPos::new(0, 63, 0), support.default_state())
    }

    #[test]
    fn eyeblossom_requires_vegetation_support() {
        init_vanilla_registry();
        let behavior =
            EyeblossomBlock::new(&vanilla_blocks::CLOSED_EYEBLOSSOM, EyeblossomType::Closed);
        let pos = BlockPos::new(0, 64, 0);
        let state = vanilla_blocks::CLOSED_EYEBLOSSOM.default_state();

        assert!(behavior.can_survive(state, &level_with_support(&vanilla_blocks::DIRT), pos));
        assert!(!behavior.can_survive(state, &level_with_support(&vanilla_blocks::AIR), pos));
    }
}
