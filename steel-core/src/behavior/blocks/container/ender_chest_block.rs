//! Ender chest block behavior implementation.

use std::sync::{Arc, Weak};

use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::BlockStateProperties;
use steel_registry::{vanilla_block_entity_types, vanilla_custom_stats};
use steel_utils::{BlockPos, BlockStateId, WorldAabb, translations};
use text_components::TextComponent;

use crate::behavior::InventoryAccess;
use crate::behavior::block::{BlockBehavior, BlockEntityCreation};
use crate::entity::SharedEntity;
use crate::entity::entities::PiglinEntity;
use crate::entity::mob::Mob as _;
use crate::behavior::context::{BlockHitResult, BlockPlaceContext, InteractionResult};
use crate::block_entity::BLOCK_ENTITIES;
use crate::inventory::menu::kinds::ender_chest;
use crate::player::Player;
use crate::world::{World, is_redstone_conductor};

/// The ender chest block behavior.
#[block_behavior]
pub struct EnderChestBlock {
    block: BlockRef,
}

/// Radius vanilla `EnderChestBlock` scans for piglins to anger.
const PIGLIN_ANGER_RADIUS: f64 = 4.0;

impl EnderChestBlock {
    /// Creates a new ender chest block.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }

    /// Vanilla `EnderChestBlock.useWithoutItem` calls `PlayerLookup.tracking`
    /// around the chest and runs `entity.setPersistentAngerTarget` for piglins,
    /// with the player as the anger target for 20 seconds.
    fn anger_nearby_piglins(world: &Arc<World>, pos: BlockPos, player: &Player) {
        let box_ = WorldAabb::new(
            f64::from(pos.x()) - PIGLIN_ANGER_RADIUS,
            f64::from(pos.y()) - PIGLIN_ANGER_RADIUS,
            f64::from(pos.z()) - PIGLIN_ANGER_RADIUS,
            f64::from(pos.x()) + PIGLIN_ANGER_RADIUS,
            f64::from(pos.y()) + PIGLIN_ANGER_RADIUS,
            f64::from(pos.z()) + PIGLIN_ANGER_RADIUS,
        );
        let player_entity: SharedEntity = Arc::<Player>::clone(player);
        for entity in world.get_entities_in_aabb(&box_) {
            let Some(piglin) = entity.as_entity_event_source().downcast_ref::<PiglinEntity>() else {
                continue;
            };
            piglin.set_target(Some(&player_entity));
        }
    }
}

impl BlockBehavior for EnderChestBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        let facing = context.horizontal_direction().opposite();

        Some(
            self.block
                .default_state()
                .set_value(&BlockStateProperties::FACING, facing)
                .set_value(
                    &BlockStateProperties::WATERLOGGED,
                    context.is_water_source(),
                ),
        )
    }

    fn allows_spectator_menu(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
    ) -> bool {
        // Vanilla `allowsSpectatorMenu` for container blocks.
        let _ = (state, world, pos);
        true
    }

    fn use_without_item(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &Player,
        _hit_result: &BlockHitResult,
        _inv: &mut InventoryAccess,
    ) -> InteractionResult {
        let Some(block_entity) = world.get_block_entity(pos) else {
            return InteractionResult::Success;
        };
        if block_entity.get_type() != &vanilla_block_entity_types::ENDER_CHEST {
            return InteractionResult::Success;
        }

        let above_pos = pos.above();
        if is_redstone_conductor(world.as_ref(), world.get_block_state(above_pos), above_pos) {
            return InteractionResult::Success;
        }

        let inventory = Arc::clone(&player.inventory);
        let container = Arc::clone(&player.ender_chest_inventory);
        let chest = Arc::downgrade(&block_entity);
        player.open_menu(
            TextComponent::translated(translations::CONTAINER_ENDERCHEST.msg()),
            move |context| {
                // open only after active chest has closed, since
                // closing an ender chest menu clears the active chest.
                container.lock().set_active_chest(chest);
                ender_chest(inventory, context.container_id, container)
            },
        );

        player.award_custom_stat(&vanilla_custom_stats::OPEN_ENDERCHEST);
        Self::anger_nearby_piglins(world, pos, player);

        InteractionResult::Success
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::ENDER_CHEST,
            level,
            pos,
            state,
        ))
    }
}
