//! Vanilla VindicatorBreakDoorGoal / BreakDoorGoal, restricted to active raids.
use super::door_interact::DoorInteractGoal;
use super::selector::{Goal, GoalControls};
use crate::entity::{PathfinderMob, SharedEntity};
use steel_utils::types::Difficulty;

pub(crate) struct RaidBreakDoorGoal {
    door: DoorInteractGoal,
    elapsed: i32,
    progress: i32,
}

impl RaidBreakDoorGoal {
    pub(crate) const fn new() -> Self {
        Self { door: DoorInteractGoal::new(), elapsed: 0, progress: -1 }
    }

    fn allowed(mob: &dyn PathfinderMob) -> bool {
        mob.level().is_some_and(|world| {
            matches!(world.difficulty(), Difficulty::Normal | Difficulty::Hard)
                && world.get_game_rule(&steel_registry::vanilla_game_rules::MOB_GRIEFING)
                && world.raids.contains_raider(mob.uuid())
        })
    }
}

impl Goal for RaidBreakDoorGoal {
    fn controls(&self) -> GoalControls { GoalControls::MOVE }

    fn can_use(&mut self, mob: &dyn PathfinderMob) -> bool {
        Self::allowed(mob) && rand::random_range(0..5) == 0
            && self.door.can_use(mob) && !self.door.is_open(mob)
    }

    fn can_continue_to_use(&mut self, mob: &dyn PathfinderMob) -> bool {
        Self::allowed(mob) && self.elapsed <= 240 && !self.door.is_open(mob)
            && self.door.has_door()
            && self.door.door_pos().closer_to_center_than(mob.position(), 2.0)
    }

    fn start(&mut self, mob: &dyn PathfinderMob) {
        self.door.start(mob);
        self.elapsed = 0;
        self.progress = -1;
        mob.set_no_action_time(0);
    }

    fn stop(&mut self, mob: &dyn PathfinderMob) {
        if let Some(world) = mob.level() {
            world.broadcast_block_destruction(mob.id(), self.door.door_pos(), -1);
        }
    }

    fn requires_update_every_tick(&self) -> bool { true }

    fn tick(&mut self, mob: &dyn PathfinderMob, _entity: &SharedEntity) {
        let Some(world) = mob.level() else { return; };
        self.door.tick(mob);
        let pos = self.door.door_pos();
        if rand::random_range(0..20) == 0 {
            world.level_event(1019, pos, 0, None);
            mob.swing_for_attack(steel_utils::types::InteractionHand::MainHand);
        }
        self.elapsed += 1;
        let progress = self.elapsed * 10 / 240;
        if progress != self.progress {
            world.broadcast_block_destruction(mob.id(), pos, progress);
            self.progress = progress;
        }
        if self.elapsed == 240 && Self::allowed(mob) {
            let state = world.get_block_state(pos);
            world.remove_block(pos, false);
            world.level_event(1021, pos, 0, None);
            world.level_event(2001, pos, i32::from(state.0), None);
        }
    }
}
