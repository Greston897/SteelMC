//! End crystal (`EndCrystalItem`).
//!
//! Vanilla `EndCrystalItem#useOnBlock`: crystals may only be placed on an
//! obsidian or bedrock block, need air directly above it and a free 1×2×1
//! volume, and always spawn with `show_bottom = false`.

use std::sync::Arc;

use steel_macros::item_behavior;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::{vanilla_blocks, vanilla_entities, vanilla_game_events};
use steel_utils::{BlockPos, WorldAabb};

use crate::behavior::context::{InteractionResult, UseOnContext};
use crate::behavior::item::ItemBehavior;
use crate::entity::entities::EndCrystalEntity;
use crate::entity::{Entity, SharedEntity, next_entity_id};
use crate::world::game_event::GameEventContext;
use crate::world::World;

/// Vanilla `EndCrystalItem`.
#[item_behavior]
pub struct EndCrystalItem;

impl EndCrystalItem {
    /// Vanilla accepts only obsidian and bedrock as the crystal's base.
    #[must_use]
    pub fn is_valid_base(world: &World, pos: BlockPos) -> bool {
        let block = world.get_block_state(pos).get_block();
        block == &vanilla_blocks::OBSIDIAN || block == &vanilla_blocks::BEDROCK
    }

    /// The entity volume the crystal needs free: one block wide, two tall.
    #[must_use]
    pub fn occupied_box(spawn: BlockPos) -> WorldAabb {
        WorldAabb::entity_box(
            f64::from(spawn.x()) + 0.5,
            f64::from(spawn.y()),
            f64::from(spawn.z()) + 0.5,
            0.5,
            1.0,
        )
    }
}

impl ItemBehavior for EndCrystalItem {
    fn use_on(&self, context: &mut UseOnContext) -> InteractionResult {
        let pos = context.hit_result.block_pos;
        if !Self::is_valid_base(context.world, pos) {
            return InteractionResult::Fail;
        }

        let spawn = pos.above();
        if !context.world.get_block_state(spawn).is_air() {
            return InteractionResult::Fail;
        }

        let occupied = Self::occupied_box(spawn);
        if context
            .world
            .has_entity_in_aabb_matching(&occupied, |_| true)
        {
            return InteractionResult::Fail;
        }

        let position = glam::DVec3::new(
            f64::from(spawn.x()) + 0.5,
            f64::from(spawn.y()),
            f64::from(spawn.z()) + 0.5,
        );
        let crystal = EndCrystalEntity::new(
            &vanilla_entities::END_CRYSTAL,
            next_entity_id(),
            position,
            Arc::downgrade(context.world),
        );
        // Vanilla `EndCrystalItem` never shows the base plate.
        crystal.set_show_bottom(false);

        let entity: SharedEntity = Arc::new(crystal);
        if context.world.try_add_entity(entity).is_err() {
            return InteractionResult::Fail;
        }

        context.world.game_event(
            &vanilla_game_events::ENTITY_PLACE,
            spawn,
            &GameEventContext::new(Some(&**context.player as &dyn Entity), None),
        );
        // TODO: Trigger the ender dragon respawn (`EnderDragonFight#respawnDragon`)
        // once Steel has a dragon fight implementation.
        context.inv.with_item(|stack| stack.consume_one(false));

        InteractionResult::Success
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use glam::DVec3;
    use steel_registry::item_stack::ItemStack;
    use steel_registry::items::item::BlockHitResult;
    use steel_registry::{init_vanilla_registry, vanilla_blocks, vanilla_items};
    use steel_utils::Downcast as _;
    use steel_utils::types::{InteractionHand, UpdateFlags};
    use steel_utils::{BlockPos, ChunkPos, Direction};

    use super::EndCrystalItem;
    use crate::behavior::ItemBehavior as _;
    use crate::behavior::context::{InteractionResult, UseOnContext};
    use crate::behavior::init_behaviors;
    use crate::entity::SharedEntity;
    use crate::entity::entities::EndCrystalEntity;
    use crate::test_support::{
        TestPlayerBuilder, TestWorld, fresh_test_world, insert_ready_full_chunk,
    };
    use crate::world::World;
    use crate::player::Player;
    use steel_registry::blocks::BlockRef;

    fn base_fixture(
        name: &str,
        base: BlockRef,
    ) -> (TestWorld, Arc<Player>, BlockPos) {
        init_vanilla_registry();
        init_behaviors();

        let world_fixture = fresh_test_world(name);
        let world = Arc::clone(&world_fixture.world);
        let pos = BlockPos::new(0, 64, 0);
        insert_ready_full_chunk(&world, ChunkPos::from_block_pos(pos));
        assert!(world.set_block(pos, base.default_state(), UpdateFlags::UPDATE_ALL));

        let player = TestPlayerBuilder::new(Arc::clone(&world), "CrystalTester", 1).build();
        player.set_client_loaded(true);
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::with_count(&vanilla_items::END_CRYSTAL, 2));

        (world_fixture, player, pos)
    }

    fn place_context<'a>(
        world: &'a Arc<World>,
        player: &'a Arc<Player>,
        pos: BlockPos,
    ) -> UseOnContext<'a> {
        UseOnContext::new(
            player,
            InteractionHand::MainHand,
            BlockHitResult {
                location: DVec3::new(0.5, 65.0, 0.5),
                direction: Direction::Up,
                block_pos: pos,
                miss: false,
                inside: false,
                world_border_hit: false,
            },
            world,
            Arc::clone(&player.inventory),
        )
    }

    fn crystals(world: &Arc<World>) -> Vec<SharedEntity> {
        world
            .get_entities_in_aabb(&steel_utils::WorldAabb::new(
                -2.0, 62.0, -2.0, 2.0, 70.0, 2.0,
            ))
            .into_iter()
            .filter(|entity| entity.downcast_ref::<EndCrystalEntity>().is_some())
            .collect()
    }

    #[test]
    fn placing_on_obsidian_spawns_a_crystal_without_a_base_plate() {
        let (fixture, player, pos) = base_fixture("crystal_obsidian", &vanilla_blocks::OBSIDIAN);
        let world = &fixture.world;

        let mut context = place_context(world, &player, pos);
        assert_eq!(
            EndCrystalItem.use_on(&mut context),
            InteractionResult::Success
        );

        let spawned = crystals(world);
        assert_eq!(spawned.len(), 1);
        let crystal = spawned[0]
            .downcast_ref::<EndCrystalEntity>()
            .expect("entity should be an end crystal");
        assert!(!crystal.shows_bottom());
        assert!(spawned[0].position().distance(DVec3::new(0.5, 65.0, 0.5)) < 1.0e-6);

        let held = player.inventory.lock().get_selected_item().clone();
        assert_eq!(held.count(), 1);
    }

    #[test]
    fn bedrock_is_also_a_valid_base() {
        let (fixture, player, pos) = base_fixture("crystal_bedrock", &vanilla_blocks::BEDROCK);
        let world = &fixture.world;

        let mut context = place_context(world, &player, pos);
        assert_eq!(
            EndCrystalItem.use_on(&mut context),
            InteractionResult::Success
        );
        assert_eq!(crystals(world).len(), 1);
    }

    #[test]
    fn other_blocks_reject_the_crystal() {
        let (fixture, player, pos) = base_fixture("crystal_stone", &vanilla_blocks::STONE);
        let world = &fixture.world;

        let mut context = place_context(world, &player, pos);
        assert_eq!(EndCrystalItem.use_on(&mut context), InteractionResult::Fail);
        assert!(crystals(world).is_empty());
        assert_eq!(
            player.inventory.lock().get_selected_item().count(),
            2,
            "a failed placement must not consume the crystal"
        );
    }

    #[test]
    fn a_blocked_block_above_rejects_the_crystal() {
        let (fixture, player, pos) = base_fixture("crystal_blocked", &vanilla_blocks::OBSIDIAN);
        let world = &fixture.world;
        assert!(world.set_block(
            pos.above(),
            vanilla_blocks::STONE.default_state(),
            UpdateFlags::UPDATE_ALL
        ));

        let mut context = place_context(world, &player, pos);
        assert_eq!(EndCrystalItem.use_on(&mut context), InteractionResult::Fail);
        assert!(crystals(world).is_empty());
    }
}
