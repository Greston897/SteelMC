//! Vanilla `RestrictOpenDoorGoal`.
//!
//! A villager that believes it is still inside its village closes doors that are
//! being held open by a raider, so the raid cannot walk straight through the
//! houses. Vanilla keeps the goal in control of nothing: it only reacts to a
//! door that [`super::door_interact::DoorInteractGoal`] found on the current
//! path, which is why it sits at the same priority as `OpenDoorGoal` in
//! `Villager.registerGoals`.

use glam::IVec3;
use steel_utils::BlockPos;

use super::door_interact::DoorInteractGoal;
use super::selector::{Goal, GoalControls};
use crate::entity::PathfinderMob;

/// Vanilla counts a position as still inside the village within this radius.
const VILLAGE_RADIUS: f64 = 64.0;

pub struct RestrictOpenDoorGoal {
    door_interact: DoorInteractGoal,
}

impl RestrictOpenDoorGoal {
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            door_interact: DoorInteractGoal::new(),
        }
    }
}

impl Goal for RestrictOpenDoorGoal {
    fn controls(&self) -> GoalControls {
        // Vanilla `RestrictOpenDoorGoal` claims no controls; it only reacts.
        GoalControls::EMPTY
    }

    fn can_use(&mut self, mob: &dyn PathfinderMob) -> bool {
        // Vanilla first requires a village around the mob, then a door on the
        // current path that is still open.
        if !is_in_village(mob) {
            return false;
        }
        if !self.door_interact.can_use(mob) {
            return false;
        }
        self.door_interact.is_open(mob)
    }

    fn can_continue_to_use(&mut self, mob: &dyn PathfinderMob) -> bool {
        if !is_in_village(mob) {
            return false;
        }
        self.door_interact.has_door() && self.door_interact.is_open(mob)
    }

    fn requires_update_every_tick(&self) -> bool {
        // The door state can change from outside at any time, so vanilla
        // re-evaluates `canContinueToUse` every tick rather than on a delay.
        true
    }

    fn stop(&mut self, mob: &dyn PathfinderMob) {
        // Vanilla closes the door again when the goal stops, which is the whole
        // point of the goal: a villager never leaves a door held open.
        self.door_interact.set_open(mob, false);
    }
}

/// Approximates vanilla `RestrictOpenDoorGoal`'s `village.isCloseTo(...)`.
///
/// Vanilla asks the mob's `Brain` for the nearest village centre and compares
/// the distance. Steel has no per-mob village memory, so it uses the same POI
/// radius the raid code uses (`POI_VILLAGE_RADIUS`) around the mob's own
/// position: a door on the path only exists when the mob is standing next to a
/// village house, so the check is equivalent in practice without the extra
/// bookkeeping.
fn is_in_village(mob: &dyn PathfinderMob) -> bool {
    let Some(world) = mob.level() else {
        return false;
    };
    let pos = mob.block_position();
    crate::world::raid::village_center_world(&world, pos)
        .is_some_and(|center| distance_sqr(center, pos) <= VILLAGE_RADIUS * VILLAGE_RADIUS)
}

fn distance_sqr(center: IVec3, pos: BlockPos) -> f64 {
    let dx = f64::from(center.x - pos.x());
    let dz = f64::from(center.z - pos.z());
    dx * dx + dz * dz
}

#[cfg(test)]
mod tests {
    use std::sync::Weak;

    use glam::DVec3;
    use steel_registry::{init_vanilla_registry, vanilla_entities};

    use super::*;
    use crate::entity::Mob as _;
    use crate::entity::entities::PigEntity;

    fn pig() -> PigEntity {
        PigEntity::new(&vanilla_entities::PIG, 1, DVec3::ZERO, Weak::new())
    }

    #[test]
    fn restrict_open_door_claims_no_controls_like_vanilla() {
        assert_eq!(RestrictOpenDoorGoal::new().controls(), GoalControls::EMPTY);
    }

    #[test]
    fn restrict_open_door_requires_a_world() {
        init_vanilla_registry();
        let mob = pig();
        assert!(!RestrictOpenDoorGoal::new().can_use(&mob));
    }

    #[test]
    fn village_distance_matches_vanilla_radius() {
        let center = IVec3::new(0, 64, 0);
        assert!(distance_sqr(center, BlockPos::new(63, 70, 0)) <= VILLAGE_RADIUS * VILLAGE_RADIUS);
        assert!(distance_sqr(center, BlockPos::new(0, 70, 64)) <= VILLAGE_RADIUS * VILLAGE_RADIUS);
        assert!(distance_sqr(center, BlockPos::new(65, 70, 0)) > VILLAGE_RADIUS * VILLAGE_RADIUS);
    }
}
