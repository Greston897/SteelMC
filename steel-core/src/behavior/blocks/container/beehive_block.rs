//! Beehive block behavior implementation.

use std::sync::Weak;

use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{
    BlockStateProperties, Direction, EnumProperty, IntProperty,
};
use steel_registry::vanilla_block_entity_types;
use steel_registry::{sound_events, vanilla_blocks, vanilla_items};
use steel_utils::types::UpdateFlags;
use steel_utils::{Downcast as _, BlockPos, BlockStateId};

use crate::behavior::block::{BlockBehavior, BlockEntityCreation};
use crate::block_entity::SharedBlockEntity;
use crate::player::player_inventory::Prediction;
use crate::behavior::context::{BlockHitResult, BlockPlaceContext};
use crate::behavior::InteractionResult;
use steel_utils::types::InteractionHand;
use crate::behavior::context::InventoryAccess;
use crate::block_entity::BLOCK_ENTITIES;
use crate::block_entity::entities::BeehiveBlockEntity;
use crate::entity::Entity as _;
use crate::player::Player;
use crate::world::World;

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
const MAX_HONEY_LEVEL: u8 = 5;
/// Vanilla `BeehiveBlock` smoke search: a campfire up to this many blocks below.
const SMOKE_SEARCH_DISTANCE: i32 = 5;

impl BeehiveBlock {
    /// Creates a new beehive block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }

    /// Vanilla `BeehiveBlock.isSmoked`: a campfire or soul campfire underneath.
    fn is_smoked(world: &World, pos: BlockPos) -> bool {
        let mut below = pos;
        for _ in 0..SMOKE_SEARCH_DISTANCE {
            below = below.below();
            let state = world.get_block_state(below);
            if state.get_block() == &vanilla_blocks::CAMPFIRE
                || state.get_block() == &vanilla_blocks::SOUL_CAMPFIRE
            {
                return true;
            }
        }
        false
    }

    /// Vanilla `BeehiveBlock.releaseBees`: spawns every stored occupant.
    fn release_bees(world: &std::sync::Arc<World>, pos: BlockPos) {
        let Some(block_entity) = world.get_block_entity(pos) else {
            return;
        };
        let Some(beehive) = block_entity.downcast_ref::<BeehiveBlockEntity>() else {
            return;
        };
        beehive.release_all_bees(world, pos);
    }

    /// Vanilla `BeehiveBlock.angerBees`: applies the anger effect to the player.
    fn anger_bees(world: &std::sync::Arc<World>, pos: BlockPos, player: &Player) {
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
        world: &std::sync::Arc<World>,
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
        // Vanilla spawns one honeycomb item entity at the hive.
        world.drop_item_stack(
            pos,
            steel_registry::item_stack::ItemStack::new(&vanilla_items::HONEYCOMB),
        );
        hive.on_honey_harvested(state, world, pos, MAX_HONEY_LEVEL, false);
    }

    /// Vanilla `BeehiveBlock.useItemOn` for bottles, shears, and campfires.
    fn handle_use(
        &self,
        state: BlockStateId,
        world: &std::sync::Arc<World>,
        pos: BlockPos,
        player: &Player,
        inv: &mut InventoryAccess,
    ) -> InteractionResult {
        let honey_level = state.get_value(LEVEL_HONEY);

        let action = inv.with_item(|stack| {
            let item = stack.item();
            if item == &*vanilla_items::GLASS_BOTTLE && honey_level == MAX_HONEY_LEVEL {
                return Some(HoneyAction::Bottle);
            }
            if item == &*vanilla_items::SHEARS && honey_level == MAX_HONEY_LEVEL {
                return Some(HoneyAction::Shear);
            }
            if item == &*vanilla_items::CAMPFIRE
                || item == &*vanilla_items::SOUL_CAMPFIRE
            {
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
                    steel_registry::item_stack::ItemStack::new(&vanilla_items::HONEY_BOTTLE),
                    Prediction::ServerOnly,
                );
                hive.on_honey_harvested(state, world, pos, 0, true);
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

    fn use_item_on(
        &self,
        state: BlockStateId,
        world: &std::sync::Arc<World>,
        pos: BlockPos,
        player: &Player,
        _hand: InteractionHand,
        _hit_result: &BlockHitResult,
        inv: &mut InventoryAccess,
    ) -> InteractionResult {
        self.handle_use(state, world, pos, player, inv)
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
        world: &std::sync::Arc<World>,
        _player: &Player,
        pos: BlockPos,
        _state: BlockStateId,
        _block_entity: Option<&SharedBlockEntity>,
        _tool: &steel_registry::item_stack::ItemStack,
    ) {
        // Vanilla `BeehiveBlock.playerDestroy` releases every occupant, so
        // breaking a hive never deletes the bees inside it.
        Self::release_bees(world, pos);
    }

    fn random_tick(&self, state: BlockStateId, world: &std::sync::Arc<World>, pos: BlockPos) {
        // Vanilla `BeehiveBlock.randomTick` only runs when the hive is smoked:
        // smoke slowly drops the honey level. Everything else (bee insert/remove)
        // is driven by the bees themselves.
        let honey_level = state.get_value(LEVEL_HONEY);
        if honey_level == 0 || !Self::is_smoked(world, pos) {
            return;
        }
        world.set_block(
            pos,
            state.set_value(LEVEL_HONEY, honey_level - 1),
            UpdateFlags::UPDATE_ALL,
        );
    }
}
