use glam::DVec3;

use super::{forced_rotation, landing_particle_count, landing_particle_position, look_at_rotation};
use steel_utils::BlockPos;

#[test]
fn look_at_rotation_matches_vanilla_axes() {
    assert_eq!(
        look_at_rotation(DVec3::ZERO, DVec3::new(0.0, 0.0, 1.0)),
        (0.0, 0.0)
    );
    assert_eq!(
        look_at_rotation(DVec3::ZERO, DVec3::new(1.0, 0.0, 0.0)),
        (-90.0, 0.0)
    );
    assert_eq!(
        look_at_rotation(DVec3::ZERO, DVec3::new(0.0, 1.0, 1.0)),
        (0.0, -45.0)
    );
    assert_eq!(
        look_at_rotation(DVec3::ZERO, DVec3::new(-1.0, 0.0, -1.0)),
        (135.0, 0.0)
    );
}

#[test]
fn forced_rotation_offsets_relative_arguments() {
    // Vanilla `Entity.forceSetRotation` hands `PositionMoveRotation` either an
    // absolute angle or an offset, and the relative flag decides which.
    assert_eq!(
        forced_rotation((30.0, 10.0), 15.0, true, 5.0, true),
        (45.0, 15.0)
    );
    assert_eq!(
        forced_rotation((30.0, 10.0), 15.0, false, 5.0, false),
        (15.0, 5.0)
    );
    assert_eq!(
        forced_rotation((350.0, 0.0), 20.0, true, 0.0, false),
        (10.0, 0.0)
    );
    // The helper only sums and wraps; `set_rotation` performs the pitch clamp
    // that vanilla applies inside `calculateAbsolute`.
    assert_eq!(
        forced_rotation((0.0, 80.0), 0.0, false, 40.0, true),
        (0.0, 120.0)
    );
}

#[test]
fn landing_particles_use_vanilla_fall_power_threshold_and_cap() {
    assert_eq!(landing_particle_count(3.0, 3.0), None);
    assert_eq!(landing_particle_count(4.0, 3.0), Some(40));
    assert_eq!(landing_particle_count(38.0, 3.0), Some(375));
}

#[test]
fn landing_particles_shift_to_the_landed_on_block_edge() {
    let position = landing_particle_position(DVec3::new(1.25, 5.0, 0.75), BlockPos::new(0, 4, 0));
    assert_eq!(position.y, 5.0);
    assert!((position.x - 1.0).abs() < f64::EPSILON);
    assert!((position.z - 2.0 / 3.0).abs() < f64::EPSILON);
}
