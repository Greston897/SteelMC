//! Raider.RaiderCelebration: surviving raiders celebrate a lost village raid.
use super::selector::{Goal, GoalControls};
use crate::entity::{PathfinderMob, SharedEntity};
use steel_registry::{sound_events, vanilla_entities};

pub(crate) struct RaidCelebrationGoal;

impl Goal for RaidCelebrationGoal {
    fn controls(&self) -> GoalControls {
        GoalControls::MOVE
    }

    fn can_use(&mut self, mob: &dyn PathfinderMob) -> bool {
        crate::entity::LivingEntity::is_alive(mob)
            && mob.target().is_none()
            && mob
                .level()
                .is_some_and(|world| world.raids.is_loss_member(mob.uuid()))
    }

    fn start(&mut self, mob: &dyn PathfinderMob) {
        mob.set_raid_celebrating(true);
        mob.mob_base().navigation().lock().stop();
    }

    fn stop(&mut self, mob: &dyn PathfinderMob) {
        mob.set_raid_celebrating(false);
    }

    fn tick(&mut self, mob: &dyn PathfinderMob, _entity: &SharedEntity) {
        if !mob.is_silent() && rand::random_range(0..50) == 0 {
            let kind = mob.entity_type();
            let sound = if kind == &vanilla_entities::PILLAGER {
                &sound_events::ENTITY_PILLAGER_CELEBRATE
            } else if kind == &vanilla_entities::VINDICATOR {
                &sound_events::ENTITY_VINDICATOR_CELEBRATE
            } else if kind == &vanilla_entities::EVOKER {
                &sound_events::ENTITY_EVOKER_CELEBRATE
            } else if kind == &vanilla_entities::WITCH {
                &sound_events::ENTITY_WITCH_CELEBRATE
            } else if kind == &vanilla_entities::RAVAGER {
                &sound_events::ENTITY_RAVAGER_CELEBRATE
            } else {
                &sound_events::ENTITY_ILLUSIONER_AMBIENT
            };
            mob.play_sound(sound, 1.0, 1.0);
        }
        if !mob.is_passenger() && rand::random_range(0..25) == 0 {
            mob.jump_control_jump();
        }
    }
}
