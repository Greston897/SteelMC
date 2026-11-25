//! Banner behavior implementation.
//!
//! Vanilla class: `BannerItem`, which is a `VerticallyAttachableBlockItem`
//! with `attachmentDirection = DOWN` (the same shape as torches). Banners place
//! the standing block when looking down and the wall block when looking at a
//! block side, so the behavior is the shared [`StandingAndWallBlockItem`]
//! placement logic configured from `classes.json`
//! (`block`, `wall_block`, `attachment_direction`).

use steel_macros::item_behavior;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::properties::Direction;

use crate::behavior::context::{InteractionResult, UseOnContext};
use crate::behavior::{ItemBehavior, StandingAndWallBlockItem};

/// Banner behavior: places a standing banner on top of a block or a wall
/// banner on the side of one.
#[item_behavior]
pub struct BannerItem {
    /// The block placed when looking toward `attachment_direction` (e.g. `white_banner`).
    #[json_arg(vanilla_blocks, json = "block")]
    pub standing_block: BlockRef,
    /// The block placed otherwise (e.g. `white_wall_banner`).
    #[json_arg(vanilla_blocks, json = "wall_block")]
    pub wall_block: BlockRef,
    /// The direction that triggers the standing block placement.
    /// Banners use `Direction::Down`, exactly like torches.
    #[json_arg(
        r#enum = "Direction",
        module = "steel_registry::blocks::properties",
        json = "attachment_direction"
    )]
    pub attachment_direction: Direction,

    /// Shared standing/wall placement logic.
    layout: StandingAndWallBlockItem,
}

impl BannerItem {
    /// Creates a new banner behavior.
    #[must_use]
    pub const fn new(
        standing_block: BlockRef,
        wall_block: BlockRef,
        attachment_direction: Direction,
    ) -> Self {
        Self {
            standing_block,
            wall_block,
            attachment_direction,
            layout: StandingAndWallBlockItem::new(standing_block, wall_block, attachment_direction),
        }
    }
}

impl ItemBehavior for BannerItem {
    fn use_on(&self, context: &mut UseOnContext) -> InteractionResult {
        self.layout.use_on(context)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use glam::DVec3;
    use steel_registry::blocks::BlockRef;
    use steel_registry::blocks::block_state_ext::BlockStateExt as _;
    use steel_registry::item_stack::ItemStack;
    use steel_registry::{init_vanilla_registry, vanilla_blocks, vanilla_items};
    use steel_utils::types::{InteractionHand, UpdateFlags};
    use steel_utils::{BlockPos, ChunkPos, Direction};

    use crate::behavior::{
        BlockHitResult, ITEM_BEHAVIORS, InteractionResult, UseOnContext, init_behaviors,
    };
    use crate::entity::Entity as _;
    use crate::inventory::container::Container as _;
    use crate::player::Player;
    use crate::test_support::{
        TestPlayerBuilder, TestWorld, fresh_test_world, insert_ready_full_chunk,
    };

    const BANNER_POS: BlockPos = BlockPos::new(8, 64, 8);

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
                BANNER_POS,
                vanilla_blocks::STONE.default_state(),
                UpdateFlags::UPDATE_KNOWN_SHAPE,
            );
            let builder = TestPlayerBuilder::new(Arc::clone(handle), "Banner", 1);
            let player = builder.build();
            // Look north so the wall variant can hang on the south face.
            player.set_rotation((180.0, 0.0));
            Self { world, player }
        }

        fn block_at(&self, pos: BlockPos) -> BlockRef {
            let state = self.world.world.get_block_state(pos);
            state.get_block()
        }

        fn place(&self, face: Direction) -> InteractionResult {
            let item = {
                let inventory = self.player.inventory.lock();
                inventory.get_item_in_hand(InteractionHand::MainHand).item()
            };
            let mut context = UseOnContext::new(
                &self.player,
                InteractionHand::MainHand,
                BlockHitResult {
                    block_pos: BANNER_POS,
                    direction: face,
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

    fn hold_banner(fixture: &Fixture) {
        let mut inventory = fixture.player.inventory.lock();
        inventory.set_item(0, ItemStack::new(&vanilla_items::WHITE_BANNER));
    }

    /// Clicking the top of a block places the standing banner block.
    #[test]
    fn banner_places_standing_block_on_top() {
        let fixture = Fixture::new("banner_placement_standing");
        hold_banner(&fixture);

        assert_eq!(fixture.place(Direction::Up), InteractionResult::Success);
        let placed = fixture.block_at(BANNER_POS.above());
        assert_eq!(placed, &vanilla_blocks::WHITE_BANNER);
    }

    /// Clicking the side of a block places the wall banner block; before this
    /// behavior existed banners placed nothing at all.
    #[test]
    fn banner_places_wall_block_on_the_side() {
        let fixture = Fixture::new("banner_placement_wall");
        hold_banner(&fixture);

        assert_eq!(fixture.place(Direction::South), InteractionResult::Success);
        let placed = fixture.block_at(BANNER_POS.south());
        assert_eq!(placed, &vanilla_blocks::WHITE_WALL_BANNER);
    }
}
