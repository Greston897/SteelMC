//! Boat item behaviour.
//!
//! Port of 26.3 `BoatItem`: raycast through blocks *and* fluids, spawn the item's
//! boat entity at the hit position, bail out when the player is looking at a
//! living entity or the boat does not fit, then play `ENTITY_PLACE` and shrink
//! the stack. One instance serves all 22 boat/raft items via the
//! `entity_type` field of `classes.json`.

use std::sync::Arc;

use glam::DVec3;
use steel_macros::item_behavior;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_game_events;

use crate::behavior::{InteractionResult, ItemBehavior, UseItemContext};
use crate::entity::{ENTITIES, Entity, next_entity_id};
use crate::physics::collision::{WorldCollisionProvider, has_collision};
use crate::world::game_event::GameEventContext;
use crate::world::raycast::{ClipBlockShape, ClipFluid};

/// Vanilla `BoatItem.ENTITY_SEARCH_RANGE`.
const ENTITY_SEARCH_RANGE: f64 = 5.0;

/// Item behavior for boat and raft items (vanilla `BoatItem`).
#[item_behavior(class = "BoatItem")]
pub struct BoatItem {
    #[json_arg(vanilla_entities, json = "entity_type")]
    entity_type: EntityTypeRef,
}

impl BoatItem {
    /// Creates a boat item behavior.
    #[must_use]
    pub const fn new(entity_type: EntityTypeRef) -> Self {
        Self { entity_type }
    }
}

impl ItemBehavior for BoatItem {
    fn use_item(&self, context: &mut UseItemContext) -> InteractionResult {
        let world = context.world;
        let player = context.player;

        let position = player.position();
        let eye = DVec3::new(position.x, player.get_eye_y(), position.z);
        let hit = world.clip_including_border(
            eye,
            eye + player.look_angle() * player.block_interaction_range(),
            ClipBlockShape::Outline,
            ClipFluid::Any,
        );
        if hit.is_miss() {
            return InteractionResult::Pass;
        }

        // Vanilla looks five blocks ahead for a living entity blocking the placement.
        let look = player.look_angle() * ENTITY_SEARCH_RANGE;
        let search_box = player.bounding_box().expand_towards(look).inflate(1.0);
        let player_id = player.id();
        let nearby = world.get_entities_in_aabb_matching(&search_box, |entity| {
            entity.id() != player_id
                && entity.is_pickable()
                && !entity.is_spectator()
                && entity.attackable()
        });
        if nearby
            .iter()
            .any(|entity| entity.bounding_box().contains(eye))
        {
            return InteractionResult::Pass;
        }

        let yaw = player.rotation().0;
        let Some(boat) = ENTITIES.create(
            self.entity_type,
            next_entity_id(),
            hit.location,
            Arc::downgrade(world),
        ) else {
            log::warn!("Failed to create boat entity {}", self.entity_type.key);
            return InteractionResult::Fail;
        };
        boat.set_rotation((yaw, 0.0));

        // Vanilla `World#isSpaceEmpty(entity, box)`: no block, entity or border collision.
        let collision = WorldCollisionProvider::for_entity(world, boat.as_ref());
        if has_collision(&collision, boat.bounding_box()) {
            return InteractionResult::Fail;
        }

        if let Err(error) = world.try_add_entity(boat) {
            log::warn!("Failed to add boat {}: {error}", self.entity_type.key);
            return InteractionResult::Fail;
        }

        let pos = hit.block_pos;
        world.game_event(
            &vanilla_game_events::ENTITY_PLACE,
            pos,
            &GameEventContext::new(
                Some(player.as_ref()),
                Some(world.get_block_state(pos.below())),
            ),
        );

        context.inv.with_item(ItemStack::shrink_one);

        InteractionResult::Success
    }
}
