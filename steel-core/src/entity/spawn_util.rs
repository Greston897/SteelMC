//! Vanilla `net.minecraft.world.entity.SpawnUtil`.
//!
//! `SpawnUtil.trySpawnMob` is the "find a place for this mob near a point"
//! helper: it walks candidate offsets up to `attempts` times, moves each
//! candidate down to the first surface, checks that the mob's box fits, and only
//! then spawns it. The sculk shrieker uses it for wardens, which is why a warden
//! summon can fail and the shrieker plays its "nothing spawned" roar instead.

use std::sync::Arc;

use glam::DVec3;
use rand::Rng;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::entity_type::EntityTypeRef;
use steel_utils::{BlockPos, WorldAabb};

use crate::entity::{
    Entity as _, EntitySpawnPlacement, EntitySpawnReason, EntitySpawnRequest, SharedEntity,
    spawn_entity,
};
use crate::physics::{WorldCollisionProvider, has_collision};
use crate::world::World;

/// Vanilla `SpawnUtil.Strategy.ON_TOP_OF_COLLIDER`.
///
/// The entity may only stand on a solid block; the position itself must be free
/// of colliders. `SpawnUtil` also accepts `IN_WATER`/`NO_RESTRICTIONS` variants,
/// but every caller in Steel (the sculk shrieker, and the trial spawner's
/// `SpawnUtil` path) uses this one.
///
/// Returns the spawned entity, or `None` when all `attempts` candidates were
/// obstructed.
#[must_use]
pub fn try_spawn_mob_on_top_of_collider(
    world: &Arc<World>,
    entity_type: EntityTypeRef,
    reason: EntitySpawnReason,
    center: BlockPos,
    attempts: i32,
    horizontal_range: i32,
    vertical_range: i32,
) -> Option<SharedEntity> {
    let mut rng = rand::rng();
    for _ in 0..attempts {
        let offset = BlockPos::new(
            center.x() + rng.random_range(-horizontal_range..=horizontal_range),
            center.y() + rng.random_range(-vertical_range..=vertical_range),
            center.z() + rng.random_range(-horizontal_range..=horizontal_range),
        );
        if let Some(entity) = try_spawn_at(world, entity_type, reason, offset) {
            return Some(entity);
        }
    }
    None
}

/// One `SpawnUtil` attempt: drop to the surface, then require a fitting box.
fn try_spawn_at(
    world: &Arc<World>,
    entity_type: EntityTypeRef,
    reason: EntitySpawnReason,
    candidate: BlockPos,
) -> Option<SharedEntity> {
    let surface = first_surface_at_or_below(world, candidate)?;
    let dimensions = entity_type.dimensions;
    let half_width = f64::from(dimensions.half_width());
    let feet_y = f64::from(surface.y());
    let aabb = WorldAabb::entity_box(
        f64::from(surface.x()) + 0.5,
        feet_y,
        f64::from(surface.z()) + 0.5,
        half_width,
        f64::from(dimensions.height),
    );
    if has_collision(&WorldCollisionProvider::new(world), aabb) {
        return None;
    }

    let request = EntitySpawnRequest {
        entity_type,
        placement: EntitySpawnPlacement::Block {
            pos: surface,
            try_move_down: false,
            moved_up: false,
        },
        reason,
        finalize_spawn: true,
        play_ambient_sound: false,
        item_stack: None,
        user_is_operator: false,
    };
    let entity = spawn_entity(world, request).ok()?;
    // `EntitySpawnPlacement::Block` centres the entity in the block; vanilla
    // `SpawnUtil` puts it at the bottom centre of the surface block, so an
    // emerging warden rises out of the floor rather than the middle of the air.
    entity.base().set_position_local(DVec3::new(
        f64::from(surface.x()) + 0.5,
        feet_y,
        f64::from(surface.z()) + 0.5,
    ));
    Some(entity)
}

/// Vanilla `SpawnUtil.getTopNonCollidingPos`: walks down until a position has a
/// solid block underneath, i.e. until the entity has something to stand on.
fn first_surface_at_or_below(world: &Arc<World>, candidate: BlockPos) -> Option<BlockPos> {
    let min_y = world.get_min_y();
    let mut pos = candidate;
    while pos.y() > min_y {
        let below = pos.below();
        if !world.get_block_state(below).is_air() {
            return Some(pos);
        }
        pos = below;
    }
    None
}
