//! Raider.ObtainRaidLeaderBannerGoal: owns MOVE instead of hijacking navigation
//! after the selector has already chosen another goal.
use super::selector::{Goal, GoalControls};
use crate::entity::ai::path::Path;
use crate::entity::entities::objects::items::ItemEntity;
use crate::entity::{Entity as _, PathfinderMob, SharedEntity};
use std::collections::BTreeMap;
use steel_registry::item_stack::ItemStack;
use steel_utils::Downcast as _;
use steel_registry::vanilla_attributes::FOLLOW_RANGE;
use crate::world::raid::ominous_banner;

pub(crate) struct ObtainRaidBannerGoal {
    unreachable_until: BTreeMap<uuid::Uuid, i64>,
    path: Option<Path>,
    pursued: Option<SharedEntity>,
}

impl ObtainRaidBannerGoal {
    pub(crate) const fn new() -> Self {
        Self {
            unreachable_until: BTreeMap::new(),
            path: None,
            pursued: None,
        }
    }
}

impl Goal for ObtainRaidBannerGoal {
    fn controls(&self) -> GoalControls {
        GoalControls::MOVE
    }

    fn can_use(&mut self, mob: &dyn PathfinderMob) -> bool {
        let Some(world) = mob.level() else {
            return false;
        };
        if !world.raids.can_pick_up_captain_banner(mob) {
            return false;
        }
        let now = world.game_time();
        self.unreachable_until.retain(|_, until| *until > now);
        let range = mob
            .attributes()
            .lock()
            .required_value(FOLLOW_RANGE);
        for candidate in
            world.get_entities_in_aabb(&mob.bounding_box().inflate_xyz(range, 8.0, range))
        {
            let Some(item) = candidate.downcast_ref::<ItemEntity>() else {
                continue;
            };
            if item.is_removed()
                || item.has_pickup_delay()
                || !ItemStack::matches(&item.get_item(), &ominous_banner())
                || self.unreachable_until.contains_key(&item.uuid())
            {
                continue;
            }
            if let Some(path) = mob
                .create_path_to(item.block_position(), 1)
                .filter(Path::can_reach)
            {
                self.path = Some(path);
                self.pursued = Some(candidate);
                return true;
            }
            self.unreachable_until.insert(item.uuid(), now + 600);
        }
        false
    }

    fn can_continue_to_use(&mut self, mob: &dyn PathfinderMob) -> bool {
        self.pursued.as_ref().is_some_and(|item| !item.is_removed())
            && !mob.mob_base().navigation().lock().is_done()
            && mob
                .level()
                .is_some_and(|world| world.raids.can_pick_up_captain_banner(mob))
    }

    fn start(&mut self, mob: &dyn PathfinderMob) {
        mob.move_to_path(self.path.take(), 1.15);
    }

    fn stop(&mut self, _mob: &dyn PathfinderMob) {
        self.path = None;
        self.pursued = None;
    }

    fn tick(&mut self, _mob: &dyn PathfinderMob, entity: &SharedEntity) {
        if let Some(item) = self
            .pursued
            .as_ref()
            .and_then(|item| item.downcast_ref::<ItemEntity>())
            && let Some(world) = entity.level()
        {
            world.raids.claim_captain_banner(entity, item, &world);
        }
    }
}
