//! Vanilla `RangedAttackMob` projectile helpers.
//!
//! The goal side lives in [`crate::entity::ai::goal::RangedAttackGoal`]; this module
//! owns the aiming math and the arrow spawn used by the skeleton family.

use std::sync::Arc;

use glam::DVec3;
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_items;
use steel_utils::types::Difficulty;

use crate::behavior::items::ArrowItem;
use crate::entity::entities::{AbstractArrow, ArrowEntity, ArrowPickup};
use crate::entity::{Projectile as _, SharedEntity};

/// Vanilla `AbstractSkeleton` arrow launch speed (`1.6`).
pub const SKELETON_ARROW_SPEED: f32 = 1.6;
/// Vanilla `AbstractSkeleton` vertical arc factor: `horizontalDistance * 0.2`.
pub const SKELETON_ARROW_ARC: f64 = 0.2;
/// Vanilla `AbstractSkeleton` spawn offset below the eye (`eyeY - 0.1`).
pub const SKELETON_ARROW_EYE_OFFSET: f64 = 0.1;
/// Vanilla mob-shooter arrow base damage.
pub const MOB_ARROW_DAMAGE: f64 = 2.0;
/// Vanilla `Entity.getY(0.3333)` factor used to aim at the target's torso.
pub const TARGET_AIM_HEIGHT_FACTOR: f64 = 0.333_333_333_333_333_3;
/// Base inaccuracy of a mob arrow (`14` minus four per difficulty step).
pub const BASE_ARROW_INACCURACY: f32 = 14.0;
/// Inaccuracy removed per difficulty step.
pub const ARROW_INACCURACY_PER_DIFFICULTY: f32 = 4.0;

/// Vanilla `AbstractSkeleton.performRangedAttack` aim vector.
///
/// The horizontal distance is folded into the vertical component (`* 0.2`) so
/// arrows arc onto the target instead of being shot flat.
#[must_use]
pub fn arced_direction(from: DVec3, to: DVec3) -> DVec3 {
    let dx = to.x - from.x;
    let dz = to.z - from.z;
    let dy = to.y - from.y;
    let horizontal = (dx * dx + dz * dz).sqrt();
    DVec3::new(dx, dy + horizontal * SKELETON_ARROW_ARC, dz)
}

/// Vanilla `(14 - difficulty.getId() * 4)` arrow inaccuracy.
#[must_use]
pub fn arrow_inaccuracy(difficulty: Difficulty) -> f32 {
    BASE_ARROW_INACCURACY - (difficulty as i32 as f32) * ARROW_INACCURACY_PER_DIFFICULTY
}

/// Spawns a skeleton-style arrow from `shooter` towards `target`.
///
/// Mirrors vanilla `AbstractSkeleton#performRangedAttack`: the arrow spawns at the
/// shooter's eye height minus `0.1`, aims at the target's torso, arcs by
/// `horizontalDistance * 0.2`, is launched at `1.6` power with the difficulty
/// dependent inaccuracy, and carries the vanilla mob arrow damage.
pub fn shoot_arrow(
    shooter: &SharedEntity,
    target: &SharedEntity,
    damage: f64,
) -> Option<Arc<ArrowEntity>> {
    let world = shooter.level()?;
    let shooter_position = shooter.position();
    let from = DVec3::new(
        shooter_position.x,
        shooter.get_eye_y() - SKELETON_ARROW_EYE_OFFSET,
        shooter_position.z,
    );

    let target_position = target.position();
    let target_height = target.bounding_box().height();
    let to = DVec3::new(
        target_position.x,
        target_position.y + target_height * TARGET_AIM_HEIGHT_FACTOR,
        target_position.z,
    );

    let arrow_stack = ItemStack::new(&vanilla_items::ARROW);
    let arrow = ArrowItem::create_arrow(
        &arrow_stack,
        from,
        &world,
        Some(shooter),
        ArrowPickup::Allowed,
    );
    arrow.set_arrow_damage(damage);
    arrow.shoot(
        arced_direction(from, to),
        SKELETON_ARROW_SPEED,
        arrow_inaccuracy(world.difficulty()),
    );
    arrow.set_has_been_shot(true);

    let shared: SharedEntity = Arc::<ArrowEntity>::clone(&arrow);
    if let Err(error) = world.try_add_entity(shared) {
        log::debug!("failed to spawn mob arrow: {error}");
        return None;
    }
    Some(arrow)
}

#[cfg(test)]
mod tests {
    use glam::DVec3;
    use steel_utils::types::Difficulty;

    use super::{SKELETON_ARROW_ARC, TARGET_AIM_HEIGHT_FACTOR, arced_direction, arrow_inaccuracy};

    #[test]
    fn flat_shot_keeps_horizontal_direction_and_adds_the_arc() {
        let from = DVec3::new(0.0, 64.0, 0.0);
        let to = DVec3::new(10.0, 64.0, 0.0);
        let direction = arced_direction(from, to);
        assert!((direction.x - 10.0).abs() < 1e-9);
        assert!((direction.z).abs() < 1e-9);
        assert!(
            (direction.y - 10.0 * SKELETON_ARROW_ARC).abs() < 1e-9,
            "a flat shot still arcs upward: {direction:?}"
        );
    }

    #[test]
    fn vertical_shot_keeps_the_drop() {
        let from = DVec3::new(0.0, 70.0, 0.0);
        let to = DVec3::new(0.0, 60.0, 0.0);
        let direction = arced_direction(from, to);
        assert!(
            (direction.y + 10.0).abs() < 1e-9,
            "no horizontal distance means no arc: {direction:?}"
        );
    }

    #[test]
    fn diagonal_distance_uses_the_horizontal_plane() {
        let from = DVec3::ZERO;
        let to = DVec3::new(3.0, 0.0, 4.0);
        let direction = arced_direction(from, to);
        assert!(
            (direction.y - 5.0 * SKELETON_ARROW_ARC).abs() < 1e-9,
            "horizontal distance of a 3-4-5 triangle is 5"
        );
    }

    #[test]
    fn inaccuracy_matches_the_vanilla_difficulty_table() {
        assert!((arrow_inaccuracy(Difficulty::Peaceful) - 14.0).abs() < f32::EPSILON);
        assert!((arrow_inaccuracy(Difficulty::Easy) - 10.0).abs() < f32::EPSILON);
        assert!((arrow_inaccuracy(Difficulty::Normal) - 6.0).abs() < f32::EPSILON);
        assert!((arrow_inaccuracy(Difficulty::Hard) - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn torso_aim_factor_matches_vanilla_get_y() {
        assert!((TARGET_AIM_HEIGHT_FACTOR - 1.0 / 3.0).abs() < 1e-12);
    }
}
