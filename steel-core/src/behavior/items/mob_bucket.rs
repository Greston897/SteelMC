//! Vanilla `MobBucketItem` (7 items: `cod_bucket`, `salmon_bucket`,
//! `tropical_fish_bucket`, `pufferfish_bucket`, `axolotl_bucket`,
//! `sulfur_cube_bucket`, `tadpole_bucket`).
//!
//! Vanilla generates all seven from `EntityBucketItem(entityType, settings,
//! fluid, unbucketSound)`, so one behavior instance serves them all and the
//! entity type is resolved from the held item.

use std::sync::Arc;

use glam::DVec3;
use steel_macros::item_behavior;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_blocks;
use steel_registry::vanilla_entities;
use steel_registry::vanilla_game_events;
use steel_registry::vanilla_items;

use crate::behavior::item_utils::create_filled_result;
use crate::behavior::{InteractionResult, ItemBehavior, UseItemContext};
use crate::entity::{ENTITIES, Entity, next_entity_id};
use crate::fluid::is_water_fluid;
use crate::world::RaytraceAction;
use crate::world::game_event::GameEventContext;

/// Vanilla `EntityBucketItem`.
#[item_behavior(class = "MobBucketItem")]
pub struct MobBucketItem;

impl MobBucketItem {
    /// Maps a mob bucket item to the entity it carries.
    #[must_use]
    pub fn entity_type_for_item(stack: &ItemStack) -> Option<EntityTypeRef> {
        if stack.is(&vanilla_items::COD_BUCKET) {
            Some(&vanilla_entities::COD)
        } else if stack.is(&vanilla_items::SALMON_BUCKET) {
            Some(&vanilla_entities::SALMON)
        } else if stack.is(&vanilla_items::TROPICAL_FISH_BUCKET) {
            Some(&vanilla_entities::TROPICAL_FISH)
        } else if stack.is(&vanilla_items::PUFFERFISH_BUCKET) {
            Some(&vanilla_entities::PUFFERFISH)
        } else if stack.is(&vanilla_items::AXOLOTL_BUCKET) {
            Some(&vanilla_entities::AXOLOTL)
        } else if stack.is(&vanilla_items::TADPOLE_BUCKET) {
            Some(&vanilla_entities::TADPOLE)
        } else if stack.is(&vanilla_items::SULFUR_CUBE_BUCKET) {
            Some(&vanilla_entities::SULFUR_CUBE)
        } else {
            None
        }
    }
}

impl ItemBehavior for MobBucketItem {
    fn use_item(&self, context: &mut UseItemContext) -> InteractionResult {
        // Vanilla `EntityBucketItem.use`: the bucket only works on a water source.
        let held = context.inv.with_item(|stack: &mut ItemStack| stack.clone());
        let Some(entity_type) = MobBucketItem::entity_type_for_item(&held) else {
            return InteractionResult::Pass;
        };

        let (start, end) = context.player.get_ray_endpoints();
        let (hit_block, _direction) = context.world.raytrace(start, end, |pos, world| {
            let state = world.get_block_state(pos);
            if state.get_block() == &vanilla_blocks::AIR {
                return RaytraceAction::Pass;
            }
            let fluid_state = state.get_fluid_state();
            if fluid_state.is_source() {
                return RaytraceAction::ImmediateHit;
            }
            if !fluid_state.is_empty() {
                return RaytraceAction::Pass;
            }
            RaytraceAction::CheckShape
        });

        let Some(hit_pos) = hit_block else {
            return InteractionResult::Pass;
        };

        if !is_water_fluid(context.world.get_block_state(hit_pos).get_fluid_state().fluid_id) {
            return InteractionResult::Pass;
        }

        let position = DVec3::new(
            f64::from(hit_pos.x()) + 0.5,
            f64::from(hit_pos.y()) + 0.5 - f64::from(entity_type.dimensions.height) / 2.0,
            f64::from(hit_pos.z()) + 0.5,
        );

        let Some(entity) = ENTITIES.create(
            entity_type,
            next_entity_id(),
            position,
            Arc::downgrade(&context.world),
        ) else {
            log::warn!("Failed to create bucketed entity {}", entity_type.key);
            return InteractionResult::Fail;
        };
        entity.set_rotation((context.player.rotation().0, 0.0));
        // Vanilla `EntityBucketItem`: the released mob comes from a bucket.
        crate::entity::bucketable::release_from_bucket(entity.as_ref(), &held);

        if let Err(error) = context.world.try_add_entity(entity) {
            log::warn!("Failed to add bucketed entity {}: {error}", entity_type.key);
            return InteractionResult::Fail;
        }

        context.world.game_event(
            &vanilla_game_events::ENTITY_PLACE,
            hit_pos,
            &GameEventContext::new(None, Some(context.world.get_block_state(hit_pos.below()))),
        );

        create_filled_result(context, ItemStack::new(&vanilla_items::WATER_BUCKET), true);

        InteractionResult::Success
    }
}
