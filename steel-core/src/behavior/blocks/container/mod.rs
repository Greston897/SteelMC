use std::sync::Arc;

use steel_utils::{Downcast as _, BlockPos, WorldAabb};

use crate::entity::{EntityEventSource as _, Mob as _, SharedEntity};
use crate::entity::entities::PiglinEntity;
use crate::entity::Mob as _;
use crate::player::Player;
use crate::world::World;

mod anvil_block;
mod block_entity_blocks;
mod barrel_block;
mod beacon_block;
mod beehive_block;
mod chest_block;
mod chiseled_bookshelf_block;
mod copper_chest_block;
mod crafting_table_block;
mod enchanting_table_block;
mod ender_chest_block;
mod furnace_block;
mod grindstone_block;
mod shulker_box_block;
mod station_blocks;

pub use anvil_block::*;
pub use block_entity_blocks::*;
pub use barrel_block::*;
pub use beacon_block::*;
pub use beehive_block::*;
pub use chest_block::*;
pub use chiseled_bookshelf_block::*;
pub use copper_chest_block::*;
pub use crafting_table_block::*;
pub use enchanting_table_block::*;
pub use ender_chest_block::*;
pub use furnace_block::*;
pub use grindstone_block::*;
pub use shulker_box_block::*;

/// Radius vanilla `EnderChestBlock` and `ShulkerBoxBlock` scan for piglins.
const PIGLIN_ANGER_RADIUS: f64 = 4.0;

/// Angers every piglin around `pos` and points it at `player`.
///
/// Vanilla runs this from `EnderChestBlock.useWithoutItem` and
/// `ShulkerBoxBlock.useWithoutItem`, both through `setPersistentAngerTarget` for
/// 20 seconds. Steel stores the anger as the piglin's attack target.
pub(crate) fn anger_nearby_piglins(world: &Arc<World>, pos: BlockPos, player: &Player) {
    let box_ = WorldAabb::new(
        f64::from(pos.x()) - PIGLIN_ANGER_RADIUS,
        f64::from(pos.y()) - PIGLIN_ANGER_RADIUS,
        f64::from(pos.z()) - PIGLIN_ANGER_RADIUS,
        f64::from(pos.x()) + PIGLIN_ANGER_RADIUS,
        f64::from(pos.y()) + PIGLIN_ANGER_RADIUS,
        f64::from(pos.z()) + PIGLIN_ANGER_RADIUS,
    );
    // The target list stores shared entities; a player reference turns back into
    // one through the world's player index, which owns the `Arc`.
    let Some(player_entity) = world
        .players
        .get_by_entity_id(player.id())
        .map(EntityEventSource::into_shared_entity)
    else {
        return;
    };
    for entity in world.get_entities_in_aabb(&box_) {
        let Some(piglin) = entity.as_entity_event_source().downcast_ref::<PiglinEntity>() else {
            continue;
        };
        let _ = piglin.set_target(Some(&player_entity));
    }
}
pub use station_blocks::*;
