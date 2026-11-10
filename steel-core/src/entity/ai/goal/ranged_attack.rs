//! Vanilla `RangedAttackGoal`.
//!
//! The goal keeps the shooter at range: it closes in until it can see the target
//! for `SEE_TIME_THRESHOLD` ticks inside `attack_radius`, then stops navigating
//! and fires every `attack_interval` ticks through
//! [`PathfinderMob::perform_ranged_attack`]. The projectile power follows
//! vanilla's `clamp(distance / radius, 0.1, 1.0)` curve, so a point-blank shot is
//! weak and a max-range shot is full strength.

use glam::DVec3;

use super::selector::{Goal, GoalControls};
use crate::entity::{PathfinderMob, SharedEntity};

const LOOK_YAW_SPEED: f32 = 30.0;
const LOOK_PITCH_SPEED: f32 = 30.0;
/// Vanilla `RangedAttackGoal` requirement: `20` ticks of continuous line of sight.
pub(crate) const SEE_TIME_THRESHOLD: i32 = 20;
const MIN_POWER: f32 = 0.1;
const MAX_POWER: f32 = 1.0;

/// Vanilla `RangedAttackGoal`.
pub(crate) struct RangedAttackGoal {
    speed_modifier: f64,
    attack_interval: i32,
    attack_radius: f32,
    attack_radius_sqr: f64,
    attack_time: i32,
    see_time: i32,
}

impl RangedAttackGoal {
    /// Vanilla `new RangedAttackGoal(mob, speedModifier, attackInterval, attackRadius)`.
    #[must_use]
    pub(crate) fn new(speed_modifier: f64, attack_interval: i32, attack_radius: f32) -> Self {
        Self {
            speed_modifier,
            attack_interval,
            attack_radius,
            attack_radius_sqr: f64::from(attack_radius) * f64::from(attack_radius),
            attack_time: -1,
            see_time: 0,
        }
    }

    /// Vanilla `Mth.clamp(sqrt(distanceSqr) / attackRadius, 0.1F, 1.0F)`.
    #[must_use]
    pub(crate) fn attack_power(&self, distance_sqr: f64) -> f32 {
        if self.attack_radius <= 0.0 {
            return MAX_POWER;
        }
        let ratio = (distance_sqr.sqrt() / f64::from(self.attack_radius)) as f32;
        ratio.clamp(MIN_POWER, MAX_POWER)
    }
}

impl Goal for RangedAttackGoal {
    fn controls(&self) -> GoalControls {
        GoalControls::MOVE | GoalControls::LOOK
    }

    fn can_use(&mut self, mob: &dyn PathfinderMob) -> bool {
        mob.target().is_some_and(|target| target.is_alive())
    }

    fn can_continue_to_use(&mut self, mob: &dyn PathfinderMob) -> bool {
        let Some(target) = mob.target() else {
            return false;
        };
        target.is_alive()
    }

    fn start(&mut self, mob: &dyn PathfinderMob) {
        mob.set_aggressive(true);
        self.see_time = 0;
        self.attack_time = -1;
    }

    fn stop(&mut self, mob: &dyn PathfinderMob) {
        mob.set_aggressive(false);
        mob.mob_base().navigation().lock().stop();
        self.see_time = 0;
        self.attack_time = -1;
    }

    fn requires_update_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, mob: &dyn PathfinderMob, entity: &SharedEntity) {
        let Some(target) = mob.target() else {
            return;
        };
        let target_position = target.position();
        let distance_sqr = mob.distance_to_sqr(target_position);

        let can_see = mob.has_line_of_sight_cached(target.as_ref());
        if can_see != (self.see_time > 0) {
            self.see_time = 0;
        }
        self.see_time += if can_see { 1 } else { -1 };

        if distance_sqr > self.attack_radius_sqr || self.see_time < SEE_TIME_THRESHOLD {
            mob.move_to_pos(target_position, self.speed_modifier);
            self.attack_time = -1;
        } else {
            mob.mob_base().navigation().lock().stop();
            self.attack_time += 1;
            if self.attack_time >= self.attack_interval {
                let power = self.attack_power(distance_sqr);
                mob.perform_ranged_attack(entity, &target, power);
                self.attack_time = 0;
            }
        }

        mob.mob_base().controls().lock().look_control.set_look_at(
            DVec3::new(target_position.x, target.get_eye_y(), target_position.z),
            LOOK_YAW_SPEED,
            LOOK_PITCH_SPEED,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{RangedAttackGoal, SEE_TIME_THRESHOLD};
    use crate::entity::ai::goal::selector::{Goal, GoalControl};

    #[test]
    fn ranged_attack_goal_uses_move_and_look_controls() {
        let goal = RangedAttackGoal::new(1.0, 60, 15.0);
        let controls = goal.controls();
        assert!(controls.contains(GoalControl::Move), "drives navigation");
        assert!(controls.contains(GoalControl::Look), "tracks the target");
        assert!(!controls.contains(GoalControl::Jump));
        assert!(goal.requires_update_every_tick());
    }

    #[test]
    fn attack_power_follows_the_vanilla_curve() {
        let goal = RangedAttackGoal::new(1.0, 60, 15.0);
        // Point blank clamps to the 0.1 floor.
        assert!((goal.attack_power(0.0) - 0.1).abs() < f32::EPSILON);
        // Half the radius is half power.
        assert!((goal.attack_power(7.5 * 7.5) - 0.5).abs() < 1e-5);
        // Exactly at the radius is full power, and beyond it clamps to 1.0.
        assert!((goal.attack_power(15.0 * 15.0) - 1.0).abs() < 1e-5);
        assert!(
            (goal.attack_power(40.0 * 40.0) - 1.0).abs() < 1e-5,
            "clamped"
        );
    }

    #[test]
    fn zero_radius_never_divides_by_zero() {
        let goal = RangedAttackGoal::new(1.0, 20, 0.0);
        assert!((goal.attack_power(4.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn see_time_threshold_matches_vanilla() {
        assert_eq!(SEE_TIME_THRESHOLD, 20);
    }
}
