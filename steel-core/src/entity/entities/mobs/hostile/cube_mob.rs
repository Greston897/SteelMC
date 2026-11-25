//! Shared size, jump and death-split helpers for the vanilla slime family.
//!
//! Vanilla 26.3 no longer makes `MagmaCube` extend `Slime`; both are
//! `AbstractCubeMob` subclasses. Steel mirrors that by keeping the shared
//! behaviour here instead of in an inheritance chain: the concrete `Slime` and
//! `MagmaCube` entities own their goal selectors and sounds, and call into these
//! helpers for the parts that are identical in both classes.
//!
//! The helpers take the values they need as arguments instead of reaching back
//! into a trait, so each caller keeps control over its own locks and over which
//! of the two vanilla classes it models.

use glam::DVec3;

use crate::entity::Mob;
use crate::entity::conversion::{AfterConversion, ConversionParams, ConversionType, convert_to};

/// Smallest vanilla cube size (`AbstractCubeMob.MIN_SIZE`).
pub(crate) const MIN_SIZE: i32 = 1;

/// Largest size vanilla clamps a cube to (`AbstractCubeMob.MAX_SIZE`).
///
/// Natural spawning only rolls up to four; the larger bound exists so
/// commands and NBT can create the big cubes vanilla supports.
pub(crate) const MAX_SIZE: i32 = 127;

/// Largest size vanilla's natural spawn rolls.
#[cfg(test)]
pub(crate) const MAX_RANDOM_SIZE: i32 = 4;

/// Returns vanilla `getSize`-derived health: `size * size`.
#[must_use]
pub(crate) fn health_for_size(size: i32) -> f64 {
    let size = f64::from(size);
    size * size
}

/// Returns vanilla movement speed: `0.2 + 0.1 * size`.
#[must_use]
pub(crate) fn movement_speed_for_size(size: i32) -> f64 {
    0.2 + 0.1 * f64::from(size)
}

/// Returns vanilla `Slime.getAttackDamage`, which is the raw size.
#[must_use]
pub(crate) fn slime_attack_damage(size: i32) -> f64 {
    f64::from(size)
}

/// Returns vanilla `MagmaCube.getAttackDamage`, which is the size plus two.
#[must_use]
pub(crate) fn magma_cube_attack_damage(size: i32) -> f64 {
    f64::from(size) + 2.0
}

/// Returns vanilla `MagmaCube` innate armor, which is the size times three.
#[must_use]
pub(crate) fn magma_cube_armor(size: i32) -> f64 {
    f64::from(size) * 3.0
}

/// Returns the XZ offset vanilla uses to place split child `index`.
///
/// Vanilla 26.3 places children at `(index % 2 - 0.5) * width / 2` and
/// `(index / 2 - 0.5) * width / 2`, where `width` is the *dying* cube's current
/// hitbox width (`0.52 * size` for both slimes and magma cubes).
#[must_use]
pub(crate) fn split_offset(index: i32, width: f64) -> (f64, f64) {
    let cube_offset = width / 2.0;
    let x = (f64::from(index % 2) - 0.5) * cube_offset;
    let z = (f64::from(index / 2) - 0.5) * cube_offset;
    (x, z)
}

/// Returns the half size a dying cube of `size` splits into, or `None` when the
/// cube is too small to split (vanilla `size > 1`).
#[must_use]
pub(crate) const fn split_child_size(size: i32) -> Option<i32> {
    if size > 1 { Some(size / 2) } else { None }
}

/// Returns vanilla `getSplitCount()`: two to four children.
#[must_use]
pub(crate) fn split_count() -> i32 {
    2 + rand::random_range(0..3)
}

/// Returns vanilla's initial hop delay for a freshly created cube.
#[must_use]
pub(crate) fn initial_jump_delay() -> i32 {
    rand::random_range(10..=30)
}

/// Returns vanilla's next hop delay.
///
/// Vanilla's `SlimeKeepOnJumpingGoal` uses a fixed 10-tick delay, which
/// `Slime.getJumpDelay` halves to 5 while the cube has a target; without a
/// target it uses `Slime.getTicksUntilNextJump`.
#[must_use]
pub(crate) const fn next_jump_delay(aggressive: bool) -> i32 {
    if aggressive { 5 } else { 10 }
}

/// Rolls vanilla's natural-spawn size (`1 << size_scale`).
///
/// `special_multiplier` is vanilla's regional-difficulty factor; Steel does not
/// track per-chunk difficulty yet, so callers pass `1.0`.
#[must_use]
pub(crate) fn random_spawn_size(special_multiplier: f32) -> i32 {
    let mut size_scale = rand::random_range(0..3);
    if size_scale < 2 && rand::random::<f32>() < 0.5 * special_multiplier {
        size_scale += 1;
    }
    1 << size_scale
}

/// Splits a dying cube into two to four half-sized children.
///
/// `size` is the dying cube's size, `width` its current hitbox width, and
/// `set_child_size` applies the child's size once conversion has copied the
/// shared mob state. Children land in vanilla's two-by-two fan, half a block up,
/// each with its own random yaw.
///
/// Returns the number of children actually created: zero when the cube is size
/// one, and fewer than [`split_count`] when the world rejects a conversion.
pub(crate) fn split_on_death(
    source: &dyn Mob,
    size: i32,
    width: f64,
    set_child_size: &dyn Fn(&dyn Mob, i32),
) -> usize {
    let Some(child_size) = split_child_size(size) else {
        return 0;
    };

    let params = ConversionParams {
        conversion_type: ConversionType::SplitOnDeath,
        keep_equipment: false,
        preserve_can_pick_up_loot: false,
    };

    let position = source.position();
    let count = split_count();
    let mut created = 0;
    for index in 0..count {
        let (x_offset, z_offset) = split_offset(index, width);
        let yaw = rand::random::<f32>() * 360.0;

        let after: &AfterConversion<'_> = &|child| {
            set_child_size(child, child_size);
            // Vanilla `setUpSplitCube` snaps the child into place, which also
            // resets its rotation to the freshly rolled yaw.
            let _ = child.try_set_position(position + DVec3::new(x_offset, 0.5, z_offset));
            child.set_rotation((yaw, 0.0));
        };

        if convert_to(source, source.entity_type(), params, Some(after)).is_some() {
            created += 1;
        }
    }
    created
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_RANDOM_SIZE, MAX_SIZE, MIN_SIZE, health_for_size, magma_cube_armor,
        magma_cube_attack_damage, movement_speed_for_size, next_jump_delay, random_spawn_size,
        slime_attack_damage, split_child_size, split_count, split_offset,
    };

    #[test]
    fn size_derived_values_match_vanilla_formulas() {
        assert_eq!(health_for_size(1), 1.0);
        assert_eq!(health_for_size(4), 16.0);
        assert!((movement_speed_for_size(1) - 0.3).abs() < 1e-9);
        assert!((movement_speed_for_size(4) - 0.6).abs() < 1e-9);
        assert_eq!(slime_attack_damage(2), 2.0);
        assert_eq!(magma_cube_attack_damage(2), 4.0);
        assert_eq!(magma_cube_armor(4), 12.0);
    }

    #[test]
    fn splits_only_halve_cubes_bigger_than_one() {
        assert_eq!(split_child_size(MIN_SIZE), None);
        assert_eq!(split_child_size(2), Some(1));
        assert_eq!(split_child_size(4), Some(2));
        assert_eq!(split_child_size(MAX_SIZE), Some(63));
    }

    #[test]
    fn split_offsets_use_half_the_dying_width() {
        // A size-4 cube is 0.52 * 4 = 2.08 wide, so the offset unit is 1.04.
        let (x, z) = split_offset(0, 2.08);
        assert!((x + 0.52).abs() < 1e-9);
        assert!((z + 0.52).abs() < 1e-9);

        let (x, z) = split_offset(3, 2.08);
        assert!((x - 0.52).abs() < 1e-9);
        assert!((z - 0.52).abs() < 1e-9);
    }

    #[test]
    fn random_spawn_sizes_stay_inside_the_natural_range() {
        for _ in 0..256 {
            let size = random_spawn_size(1.0);
            assert!((MIN_SIZE..=MAX_RANDOM_SIZE).contains(&size));
            assert!(size.cast_unsigned().is_power_of_two());
        }
    }

    #[test]
    fn jump_delays_follow_the_goal_and_target_state() {
        assert_eq!(next_jump_delay(false), 10);
        assert_eq!(next_jump_delay(true), 5);
    }

    #[test]
    fn split_count_is_between_two_and_four() {
        for _ in 0..256 {
            assert!((2..=4).contains(&split_count()));
        }
    }
}
