//! Arrow and spectral arrow items (`ArrowItem`, `SpectralArrowItem`).
//!
//! Vanilla arrow items carry no click behaviour of their own: they exist to
//! produce the matching projectile (`ArrowItem#createArrow`). Bows and crossbows
//! use these factories, so the arrow type, its pickup permission and its item
//! copy are decided in one place.

use std::sync::Arc;

use glam::DVec3;
use steel_macros::item_behavior;
use steel_registry::item_stack::ItemStack;
use steel_registry::{vanilla_entities, vanilla_items};

use crate::behavior::item::ItemBehavior;
use crate::entity::entities::{AbstractArrow, ArrowEntity, ArrowPickup, SpectralArrowEntity};
use crate::entity::{Entity as _, Projectile as _, SharedEntity, next_entity_id};
use crate::world::World;

/// Regular arrows (plain and tipped).
#[item_behavior(class = "ArrowItem")]
pub struct ArrowItem;

impl ArrowItem {
    /// Vanilla `ArrowItem#createArrow(world, stack, shooter, pickupStack)`.
    #[must_use]
    pub fn create_arrow(
        stack: &ItemStack,
        position: DVec3,
        world: &Arc<World>,
        shooter: Option<&SharedEntity>,
        pickup: ArrowPickup,
    ) -> Arc<ArrowEntity> {
        let arrow = Arc::new(ArrowEntity::new(
            &vanilla_entities::ARROW,
            next_entity_id(),
            position,
            Arc::downgrade(world),
        ));
        arrow.set_arrow_item(stack.copy_with_count(1));
        arrow.set_pickup(pickup);
        if let Some(shooter) = shooter {
            arrow.set_owner_entity(Some(shooter));
            arrow.cache_owner_entity(shooter);
        }
        arrow
    }

    /// Whether `stack` is any kind of arrow the bow accepts.
    #[must_use]
    pub fn is_arrow(stack: &ItemStack) -> bool {
        stack.is(&vanilla_items::ARROW)
            || stack.is(&vanilla_items::SPECTRAL_ARROW)
            || stack.is(&vanilla_items::TIPPED_ARROW)
    }
}

/// Spectral arrows: same item contract, different projectile and glow effect.
#[item_behavior(class = "SpectralArrowItem")]
pub struct SpectralArrowItem;

impl SpectralArrowItem {
    /// Vanilla `SpectralArrowItem#createArrow`: spawns a `SpectralArrow`.
    #[must_use]
    pub fn create_arrow(
        stack: &ItemStack,
        position: DVec3,
        world: &Arc<World>,
        shooter: Option<&SharedEntity>,
        pickup: ArrowPickup,
    ) -> Arc<SpectralArrowEntity> {
        let arrow = Arc::new(SpectralArrowEntity::new(
            &vanilla_entities::SPECTRAL_ARROW,
            next_entity_id(),
            position,
            Arc::downgrade(world),
        ));
        arrow.set_arrow_item(stack.copy_with_count(1));
        arrow.set_pickup(pickup);
        if let Some(shooter) = shooter {
            arrow.set_owner_entity(Some(shooter));
            arrow.cache_owner_entity(shooter);
        }
        arrow
    }
}

impl ItemBehavior for SpectralArrowItem {}

impl ItemBehavior for ArrowItem {}
