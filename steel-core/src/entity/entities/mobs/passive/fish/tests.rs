//! Behaviour-parity tests for the shared water-mob helpers.

use glam::DVec3;

use super::{
    FISH_FLOP_HORIZONTAL, FISH_FLOP_IMPULSE, FISH_WATER_DRAG, FISH_WATER_SINKING_BIAS,
    WATER_ANIMAL_MAX_AIR, WATER_ANIMAL_SPAWN_DEPTH_OFFSET, fish_water_drag, flop_velocity,
    is_within_spawn_depth,
};

#[test]
fn spawn_depth_window_matches_vanilla_water_animal() {
    // Vanilla: `pos.getY() >= seaLevel - 13 && pos.getY() <= seaLevel`.
    assert_eq!(WATER_ANIMAL_SPAWN_DEPTH_OFFSET, 13);

    let sea_level = 63;
    assert!(is_within_spawn_depth(sea_level, 63));
    assert!(is_within_spawn_depth(sea_level, 50));
    assert!(!is_within_spawn_depth(sea_level, 49));
    assert!(!is_within_spawn_depth(sea_level, 64));
}

#[test]
fn air_supply_cap_matches_vanilla_water_animal() {
    // Vanilla `WaterAnimal.MAX_AIR = 300`; `Squid` hard-codes the same value.
    assert_eq!(WATER_ANIMAL_MAX_AIR, 300);
}

#[test]
fn flop_impulse_uses_fixed_upward_velocity() {
    let velocity = flop_velocity(0.05, -0.05);
    assert_eq!(velocity.y, FISH_FLOP_IMPULSE);
    assert!((velocity.y - 0.4).abs() < f64::EPSILON);
    assert!((velocity.x - FISH_FLOP_HORIZONTAL).abs() < f64::EPSILON);
    assert!((velocity.z + FISH_FLOP_HORIZONTAL).abs() < f64::EPSILON);
}

#[test]
fn water_drag_matches_vanilla_fish_travel() {
    let velocity = DVec3::new(1.0, 0.5, -0.25);

    let idle = fish_water_drag(velocity, true);
    assert!((idle.x - FISH_WATER_DRAG).abs() < f64::EPSILON);
    assert!((idle.y - (0.5 * FISH_WATER_DRAG - FISH_WATER_SINKING_BIAS)).abs() < 1.0e-12);
    assert!((idle.z + 0.25 * FISH_WATER_DRAG).abs() < f64::EPSILON);

    let moving = fish_water_drag(velocity, false);
    assert!((moving.y - 0.5 * FISH_WATER_DRAG).abs() < f64::EPSILON);
}

#[test]
fn water_drag_keeps_vanilla_constants() {
    assert!((FISH_WATER_DRAG - 0.9).abs() < f64::EPSILON);
    assert!((FISH_WATER_SINKING_BIAS - 0.005).abs() < f64::EPSILON);
}
