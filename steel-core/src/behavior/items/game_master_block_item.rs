//! Operator-only block items.
//!
//! Vanilla class: `OperatorOnlyBlockItem` (the 26.3 behavior dump names the
//! class `GameMasterBlockItem`). Command blocks, structure blocks, jigsaw
//! blocks and test blocks can only be placed by players that hold the
//! "creative level two op" permission; for anybody else the placement state is
//! `null`, so nothing is placed and no item is consumed.

use steel_macros::item_behavior;
use steel_registry::blocks::BlockRef;

use crate::behavior::context::{InteractionResult, UseOnContext};
use crate::behavior::{BlockItem, ItemBehavior};

/// Block item that refuses to place anything for non-operators.
#[item_behavior]
pub struct GameMasterBlockItem {
    /// The block this item places (command block, structure block, ...).
    #[json_arg(vanilla_blocks, json = "block")]
    pub block: BlockRef,

    /// Regular block placement, used once the operator gate passes.
    base: BlockItem,
}

impl GameMasterBlockItem {
    /// Creates a new operator-only block item behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self {
            block,
            base: BlockItem::new(block),
        }
    }
}

impl ItemBehavior for GameMasterBlockItem {
    fn use_on(&self, context: &mut UseOnContext) -> InteractionResult {
        // Vanilla `OperatorOnlyBlockItem.getPlacementState` returns `null`
        // unless `player.isCreativeLevelTwoOp()`, which aborts placement before
        // any state is chosen. Steel models the vanilla permission levels with
        // permission groups, so the operator group is the equivalent gate.
        if !context.player.is_operator() {
            return InteractionResult::Fail;
        }
        self.base.use_on(context)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use glam::DVec3;
    use steel_registry::blocks::BlockRef;
    use steel_registry::blocks::block_state_ext::BlockStateExt as _;
    use steel_registry::item_stack::ItemStack;
    use steel_registry::items::ItemRef;
    use steel_registry::{init_vanilla_registry, vanilla_blocks, vanilla_items};
    use steel_utils::types::{InteractionHand, UpdateFlags};
    use steel_utils::{BlockPos, ChunkPos, Direction};

    use crate::behavior::{
        BlockHitResult, ITEM_BEHAVIORS, InteractionResult, UseOnContext, init_behaviors,
    };
    use crate::inventory::container::Container as _;
    use crate::player::Player;
    use crate::test_support::{
        TestPlayerBuilder, TestWorld, fresh_test_world, insert_ready_full_chunk,
    };

    const TARGET: BlockPos = BlockPos::new(8, 64, 8);

    struct Fixture {
        world: TestWorld,
        player: Arc<Player>,
    }

    impl Fixture {
        fn new(name: &'static str) -> Self {
            init_vanilla_registry();
            init_behaviors();
            let world = fresh_test_world(name);
            let handle = &world.world;
            insert_ready_full_chunk(handle, ChunkPos::new(0, 0));
            handle.set_block(
                TARGET,
                vanilla_blocks::STONE.default_state(),
                UpdateFlags::UPDATE_KNOWN_SHAPE,
            );
            let builder = TestPlayerBuilder::new(Arc::clone(handle), "Operator", 1);
            let player = builder.build();
            Self { world, player }
        }

        fn block_at(&self, pos: BlockPos) -> BlockRef {
            let state = self.world.world.get_block_state(pos);
            state.get_block()
        }

        fn place(&self, item: ItemRef) -> InteractionResult {
            {
                let mut inventory = self.player.inventory.lock();
                inventory.set_item(0, ItemStack::new(item));
            }
            let mut context = UseOnContext::new(
                &self.player,
                InteractionHand::MainHand,
                BlockHitResult {
                    block_pos: TARGET,
                    direction: Direction::Up,
                    location: DVec3::new(8.5, 64.5, 8.5),
                    miss: false,
                    inside: false,
                    world_border_hit: false,
                },
                &self.world.world,
                Arc::clone(&self.player.inventory),
            );
            ITEM_BEHAVIORS.get_behavior(item).use_on(&mut context)
        }
    }

    /// A non-operator must not be able to place a command block, while a plain
    /// block item still places normally in the same setup.
    #[test]
    fn command_blocks_are_operator_only() {
        let fixture = Fixture::new("game_master_block_gate");

        let denied = fixture.place(&*vanilla_items::COMMAND_BLOCK);
        assert_eq!(denied, InteractionResult::Fail);
        assert_eq!(fixture.block_at(TARGET.above()), &vanilla_blocks::AIR);

        let placed = fixture.place(&*vanilla_items::STONE);
        assert_eq!(placed, InteractionResult::Success);
        assert_eq!(fixture.block_at(TARGET.above()), &vanilla_blocks::STONE);
    }
}
