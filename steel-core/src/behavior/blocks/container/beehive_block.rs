//! Beehive block behavior implementation.

use std::sync::{Arc, Weak};

use steel_macros::block_behavior;
use steel_registry::block_entity_type::BlockEntityTypeRef;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{
    BlockStateProperties, Direction, EnumProperty, IntProperty,
};
use steel_registry::blocks::shapes::{BooleanOp, VoxelShape, join_is_not_empty};
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_block_entity_types;
use steel_registry::vanilla_criteria_triggers::BEE_NEST_DESTROYED;
use steel_registry::vanilla_enchantment_tags::EnchantmentTag;
use steel_registry::{REGISTRY, RegistryExt, TaggedRegistryExt, sound_events, vanilla_items};
use steel_utils::{BlockLocalAabb, BlockPos, BlockStateId, Downcast as _};

use crate::behavior::block::{BlockBehavior, BlockCollisionContext, BlockEntityCreation};
use crate::behavior::blocks::CampfireBlock;
use crate::behavior::context::InventoryAccess;
use crate::behavior::context::{BlockHitResult, BlockPlaceContext};
use crate::behavior::{BLOCK_BEHAVIORS, InteractionResult};
use crate::block_entity::SharedBlockEntity;
use crate::block_entity::entities::BeehiveBlockEntity;
use crate::block_entity::{BLOCK_ENTITIES, BlockEntityTicker};
use crate::entity::Entity as _;
use crate::player::Player;
use crate::player::advancement::predicate::TriggerContext;
use crate::player::player_inventory::Prediction;
use crate::world::World;
use steel_utils::types::InteractionHand;

/// Behavior for beehive and bee nest blocks.
///
/// Mirrors vanilla `BeehiveBlock`:
/// * honey bottles are filled at `LEVEL_HONEY == 5` and reset the hive,
/// * shears harvest honeycomb at `LEVEL_HONEY == 5`,
/// * campfire smoke (a campfire or soul campfire below within five blocks)
///   calms the bees so harvesting does not anger them,
/// * breaking the block releases the stored bees.
#[block_behavior]
pub struct BeehiveBlock {
    block: BlockRef,
}

const HORIZONTAL_FACING: &EnumProperty<Direction> = &BlockStateProperties::HORIZONTAL_FACING;
const LEVEL_HONEY: &IntProperty = &BlockStateProperties::LEVEL_HONEY;

/// Vanilla `BeehiveBlock.MAX_HONEY_LEVEL`.
const MAX_HONEY_LEVEL: i32 = 5;
/// Vanilla `BeehiveBlock` smoke search: a campfire up to this many blocks below.
const SMOKE_SEARCH_DISTANCE: i32 = 5;
const SMOKE_COLUMN: VoxelShape =
    VoxelShape::from_boxes(&[BlockLocalAabb::new(0.375, 0.0, 0.375, 0.625, 1.0, 0.625)]);

impl BeehiveBlock {
    /// Creates a new beehive block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }

    fn is_smoked(world: &World, pos: BlockPos) -> bool {
        let mut below = pos;
        for _ in 0..SMOKE_SEARCH_DISTANCE {
            below = below.below();
            let state = world.get_block_state(below);
            if CampfireBlock::is_lit_campfire(state) {
                return true;
            }
            let shape = BLOCK_BEHAVIORS
                .get_behavior(state.get_block())
                .get_collision_shape(state, world, pos, BlockCollisionContext::empty());
            if join_is_not_empty(SMOKE_COLUMN, shape, BooleanOp::And) {
                return CampfireBlock::is_lit_campfire(world.get_block_state(below.below()));
            }
        }
        false
    }

    /// Vanilla `BeehiveBlock.angerBees`: applies the anger effect to the player.
    fn anger_bees(world: &Arc<World>, pos: BlockPos, player: &Player) {
        let Some(block_entity) = world.get_block_entity(pos) else {
            return;
        };
        let Some(beehive) = block_entity.downcast_ref::<BeehiveBlockEntity>() else {
            return;
        };
        let Some(harvester) = world.get_entity_by_id(player.id()) else {
            return;
        };
        beehive.anger_bees(world, &harvester);
    }

    /// Vanilla `BeehiveBlock.dropHoneycomb` side effects.
    fn drop_honeycomb(
        world: &Arc<World>,
        pos: BlockPos,
        player: &Player,
        state: BlockStateId,
        hive: &BeehiveBlockEntity,
    ) {
        world.play_block_sound(
            &sound_events::BLOCK_BEEHIVE_SHEAR,
            pos,
            1.0,
            1.0,
            Some(player.id()),
        );
        // Vanilla drops one item entity containing three honeycombs.
        world.drop_item_stack(pos, ItemStack::with_count(&vanilla_items::HONEYCOMB, 3));
        hive.on_honey_harvested(state, world, pos, 0, false);
    }

    /// Vanilla `BeehiveBlock.useItemOn` for bottles, shears, and campfires.
    fn handle_use(
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &Player,
        inv: &mut InventoryAccess,
    ) -> InteractionResult {
        let honey_level = state.get_value(LEVEL_HONEY);

        let action = inv.with_item(|stack| {
            let item = stack.item();
            if item == &*vanilla_items::GLASS_BOTTLE && i32::from(honey_level) == MAX_HONEY_LEVEL {
                return Some(HoneyAction::Bottle);
            }
            if item == &*vanilla_items::SHEARS && i32::from(honey_level) == MAX_HONEY_LEVEL {
                return Some(HoneyAction::Shear);
            }
            if item == &*vanilla_items::CAMPFIRE || item == &*vanilla_items::SOUL_CAMPFIRE {
                return Some(HoneyAction::PlaceCampfire);
            }
            None
        });

        let Some(action) = action else {
            return InteractionResult::Pass;
        };

        let smoked = Self::is_smoked(world, pos);
        let Some(block_entity) = world.get_block_entity(pos) else {
            return InteractionResult::Pass;
        };
        let Some(hive) = block_entity.downcast_ref::<BeehiveBlockEntity>() else {
            return InteractionResult::Pass;
        };

        match action {
            HoneyAction::Bottle => {
                let infinite = player.has_infinite_materials();
                inv.with_item(|stack| {
                    stack.consume_one(infinite);
                });
                // Vanilla gives the honey bottle to the player or drops it.
                player.add_item_or_drop(
                    ItemStack::new(&vanilla_items::HONEY_BOTTLE),
                    Prediction::ServerOnly,
                );
                hive.on_honey_harvested(state, world, pos, 0, false);
                if !smoked {
                    Self::anger_bees(world, pos, player);
                }
                InteractionResult::Success
            }
            HoneyAction::Shear => {
                Self::drop_honeycomb(world, pos, player, state, hive);
                if !smoked {
                    Self::anger_bees(world, pos, player);
                }
                InteractionResult::Success
            }
            HoneyAction::PlaceCampfire => InteractionResult::Pass,
        }
    }
}

/// Which vanilla beehive interaction a held item triggers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HoneyAction {
    Bottle,
    Shear,
    PlaceCampfire,
}

impl BlockBehavior for BeehiveBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(
            self.block
                .default_state()
                .set_value(HORIZONTAL_FACING, context.horizontal_direction().opposite()),
        )
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::BEEHIVE,
            level,
            pos,
            state,
        ))
    }

    /// Vanilla `BeehiveBlock.getTicker` returns `BeehiveBlockEntity.serverTick`.
    ///
    /// Without it stored bees never age and a worldgen nest never lets them out.
    fn get_block_entity_ticker(
        &self,
        _world: &Arc<World>,
        _state: BlockStateId,
        block_entity_type: BlockEntityTypeRef,
    ) -> Option<BlockEntityTicker> {
        BlockEntityTicker::for_matching_entity_tick(
            block_entity_type,
            &vanilla_block_entity_types::BEEHIVE,
        )
    }

    fn use_item_on(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &Player,
        _hand: InteractionHand,
        _hit_result: &BlockHitResult,
        inv: &mut InventoryAccess,
    ) -> InteractionResult {
        Self::handle_use(state, world, pos, player, inv)
    }

    fn has_analog_output_signal(&self, _state: BlockStateId) -> bool {
        true
    }

    fn get_analog_output_signal(
        &self,
        state: BlockStateId,
        _world: &World,
        _pos: BlockPos,
        _direction: Direction,
    ) -> i32 {
        state.get_value(LEVEL_HONEY).into()
    }

    fn player_destroy(
        &self,
        world: &Arc<World>,
        player: &Player,
        pos: BlockPos,
        state: BlockStateId,
        block_entity: Option<&SharedBlockEntity>,
        tool: &ItemStack,
    ) {
        let Some(hive) =
            block_entity.and_then(|entity| entity.downcast_ref::<BeehiveBlockEntity>())
        else {
            return;
        };
        let prevents_release = tool.get_enchantments().is_some_and(|enchantments| {
            enchantments.iter().any(|(key, _)| {
                REGISTRY
                    .enchantments
                    .by_key(key)
                    .is_some_and(|enchantment| {
                        REGISTRY.enchantments.is_in_tag(
                            enchantment,
                            &EnchantmentTag::PREVENTS_BEE_SPAWNS_WHEN_MINING,
                        )
                    })
            })
        });
        if !prevents_release {
            hive.release_all_bees(world, pos);
        }
        let item = (!tool.is_empty()).then_some(&tool.item().key);
        let mut context = TriggerContext::block(&state.get_block().key, item);
        context.bees_inside = Some(i32::try_from(hive.occupant_count()).unwrap_or(i32::MAX));
        player.trigger_advancement_with_context(&BEE_NEST_DESTROYED, &context);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bootstrap::init_globals;
    use crate::entity::entities::ItemEntity;
    use crate::test_support::{
        TestPlayerBuilder, dropped_items, fresh_test_world, insert_ready_full_chunk,
    };
    use glam::DVec3;
    use steel_registry::vanilla_blocks;
    use steel_utils::ChunkPos;
    use steel_utils::types::UpdateFlags;

    #[test]
    fn failed_release_keeps_bees_in_the_hive() {
        init_globals();
        let fixture = fresh_test_world("hive_failed_release");
        let world = &fixture.world;
        let pos = BlockPos::new(1000, 64, 1000);
        let hive = BeehiveBlockEntity::new(
            Arc::downgrade(world),
            pos,
            vanilla_blocks::BEE_NEST.default_state(),
        );
        hive.store_worldgen_bee(0);
        hive.release_all_bees(world, pos);
        assert_eq!(hive.occupant_count(), 1);
    }

    #[test]
    fn smoke_requires_lit_campfire_and_respects_collision_column() {
        init_globals();
        let fixture = fresh_test_world("hive_smoke_obstruction");
        let world = &fixture.world;
        let pos = BlockPos::new(8, 64, 8);
        let _holder = insert_ready_full_chunk(world, ChunkPos::from_block_pos(pos));
        let fire_pos = pos.below().below().below();
        let lit = vanilla_blocks::CAMPFIRE
            .default_state()
            .set_value(&BlockStateProperties::LIT, true);
        world.set_block(fire_pos, lit, UpdateFlags::UPDATE_CLIENTS);
        assert!(BeehiveBlock::is_smoked(world, pos));
        world.set_block(
            fire_pos,
            lit.set_value(&BlockStateProperties::LIT, false),
            UpdateFlags::UPDATE_CLIENTS,
        );
        assert!(!BeehiveBlock::is_smoked(world, pos));
        world.set_block(fire_pos, lit, UpdateFlags::UPDATE_CLIENTS);
        world.set_block(
            pos.below(),
            vanilla_blocks::STONE.default_state(),
            UpdateFlags::UPDATE_CLIENTS,
        );
        assert!(!BeehiveBlock::is_smoked(world, pos));
        world.set_block(pos.below().below(), lit, UpdateFlags::UPDATE_CLIENTS);
        assert!(BeehiveBlock::is_smoked(world, pos));
    }

    #[test]
    fn breaking_removed_hive_respects_enchantment_and_releases_occupants() {
        use steel_registry::vanilla_enchantments;

        init_globals();
        let fixture = fresh_test_world("hive_break_occupants");
        let world = &fixture.world;
        let pos = BlockPos::new(8, 64, 8);
        let _holder = insert_ready_full_chunk(world, ChunkPos::from_block_pos(pos));
        let player = TestPlayerBuilder::new(Arc::clone(world), "HiveBreaker", 1).build();
        let behavior = BeehiveBlock::new(&vanilla_blocks::BEE_NEST);
        let state = vanilla_blocks::BEE_NEST.default_state();

        for silk_touch in [false, true] {
            world.set_block(pos, state, UpdateFlags::UPDATE_ALL);
            let entity = world.get_block_entity(pos).expect("hive block entity");
            let hive = entity
                .downcast_ref::<BeehiveBlockEntity>()
                .expect("typed hive");
            for _ in 0..3 {
                hive.store_worldgen_bee(0);
            }
            world.set_block(
                pos,
                vanilla_blocks::AIR.default_state(),
                UpdateFlags::UPDATE_ALL,
            );
            assert!(world.get_block_entity(pos).is_none());
            let mut tool = ItemStack::new(&vanilla_items::DIAMOND_PICKAXE);
            if silk_touch {
                tool.upgrade_enchantment(vanilla_enchantments::SILK_TOUCH.key.clone(), 1);
            }
            behavior.player_destroy(world, &player, pos, state, Some(&entity), &tool);
            assert_eq!(hive.occupant_count(), if silk_touch { 3 } else { 0 });
        }
    }

    #[test]
    fn shearing_full_hive_resets_honey_and_drops_three_honeycombs() {
        init_globals();
        let fixture = fresh_test_world("hive_shear_resets_honey");
        let world = &fixture.world;
        let pos = BlockPos::new(8, 64, 8);
        let _holder = insert_ready_full_chunk(world, ChunkPos::from_block_pos(pos));
        let state = vanilla_blocks::BEEHIVE
            .default_state()
            .set_value(LEVEL_HONEY, 5);
        world.set_block(pos, state, UpdateFlags::UPDATE_ALL);
        let player = TestPlayerBuilder::new(Arc::clone(world), "HiveShearer", 1).build();
        let entity = world.get_block_entity(pos).expect("hive block entity");
        let hive = entity
            .downcast_ref::<BeehiveBlockEntity>()
            .expect("typed hive");
        BeehiveBlock::drop_honeycomb(world, pos, &player, state, hive);
        assert_eq!(world.get_block_state(pos).get_value(LEVEL_HONEY), 0);
        let drops = dropped_items(world, DVec3::new(8.5, 65.0, 8.5));
        assert_eq!(drops.len(), 1);
        let item = drops[0]
            .downcast_ref::<ItemEntity>()
            .expect("honeycomb item")
            .get_item();
        assert!(item.is(&vanilla_items::HONEYCOMB));
        assert_eq!(item.count, 3);
    }
}
