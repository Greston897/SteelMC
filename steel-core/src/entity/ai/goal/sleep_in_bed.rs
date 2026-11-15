//! Steel's goal-form of vanilla `net.minecraft.world.entity.ai.behavior.SleepInBed`.
//!
//! Vanilla 1.21 runs villager sleeping as a brain behaviour, not a goal: the
//! `VillagerGoalPackages` brain walks to the `HOME` memory and `SleepInBed` puts
//! the villager to bed once `target.pos().closerToCenterThan(body.position(), 2.0)`
//! holds and the bed is not `OCCUPIED`. Steel does not model villager brains, so
//! this goal reproduces the observable behaviour with the same POI index
//! (`World::poi_storage`), the same 2.0 block distance and the same occupied-bed
//! test. The difference is the trigger: Steel polls every 20 ticks from the goal
//! selector instead of reacting to a brain memory.

use glam::DVec3;
use steel_registry::RegistryExt as _;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::vanilla_poi_types;
use steel_utils::BlockPos;

use super::move_to_block::MoveToBlockGoal;
use super::selector::{Goal, GoalControls};
use crate::entity::{PathfinderMob, SharedEntity};
use crate::poi::poi_storage::OccupationStatus;
use crate::world::World;

/// Vanilla `SleepInBedGoal` search radius.
const SEARCH_RADIUS: i32 = 16;

/// Vanilla walk speed modifier.
const SPEED_MODIFIER: f64 = 0.6;

/// Vanilla stops sleeping as soon as it is this close to the bed.
const SLEEP_DISTANCE: f64 = 2.0;

/// Vanilla `SleepInBedGoal`.
pub struct SleepInBedGoal {
    inner: MoveToBlockGoal,
    /// The bed the villager is currently heading for.
    bed_pos: Option<BlockPos>,
}

impl SleepInBedGoal {
    /// Creates the goal.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            inner: MoveToBlockGoal::new(SPEED_MODIFIER, SEARCH_RADIUS, |_world, _pos| false),
            bed_pos: None,
        }
    }

    /// Nearest free `HOME` POI within the vanilla `SleepInBedGoal` radius.
    fn nearest_free_bed(mob: &dyn PathfinderMob, world: &World) -> Option<BlockPos> {
        // The storage keys POIs by registry id, and `PointOfInterestType` does
        // not carry one; the registry is what maps the key to the id.
        let home_key = &vanilla_poi_types::HOME.key;
        let bed_type = steel_registry::REGISTRY.poi_types.id_from_key(home_key)?;

        let center = mob.block_position();
        let storage = world.poi_storage.lock();
        let mut candidates = storage.get_in_horizontal_square(
            &|poi_type_id| poi_type_id == bed_type,
            center,
            SEARCH_RADIUS,
            OccupationStatus::Free,
        );
        drop(storage);

        candidates.retain(|(pos, _)| mob.is_within_home_pos(*pos));
        candidates
            .into_iter()
            .min_by_key(|(pos, _)| {
                let dx = i64::from(pos.0.x.wrapping_sub(center.0.x));
                let dz = i64::from(pos.0.z.wrapping_sub(center.0.z));
                dx * dx + dz * dz
            })
            .map(|(pos, _)| pos)
    }
}

impl Goal for SleepInBedGoal {
    fn controls(&self) -> GoalControls {
        GoalControls::MOVE
    }

    fn can_use(&mut self, mob: &dyn PathfinderMob) -> bool {
        // Vanilla villagers seek a bed only at night and never while asleep.
        let Some(world) = mob.level() else {
            return false;
        };
        if world.is_day() || mob.is_sleeping() {
            return false;
        }

        let Some(bed) = Self::nearest_free_bed(mob, world.as_ref()) else {
            return false;
        };
        // Vanilla `SleepInBed` also requires the bed itself to be free: the POI
        // ticket may still be held while the block reports `OCCUPIED`.
        if !bed_is_free(&world, bed) {
            return false;
        }
        self.bed_pos = Some(bed);
        true
    }

    fn start(&mut self, mob: &dyn PathfinderMob) {
        self.inner.start(mob);
    }

    fn tick(&mut self, mob: &dyn PathfinderMob, entity: &SharedEntity) {
        let Some(bed) = self.bed_pos else {
            return;
        };

        if (mob.position() - block_center(bed)).length() <= SLEEP_DISTANCE {
            let _ = mob.start_sleeping(bed);
            return;
        }

        self.inner.tick(mob, entity);
    }

    fn stop(&mut self, mob: &dyn PathfinderMob) {
        if mob.is_sleeping() {
            mob.stop_sleeping();
        }
        self.bed_pos = None;
        self.inner.stop(mob);
    }
}

/// Vanilla `SleepInBed` rejects a bed whose `occupied` property is set.
fn bed_is_free(world: &World, pos: BlockPos) -> bool {
    let state = world.get_block_state(pos);
    state.get_property_str("occupied") != Some("true")
}

fn block_center(pos: BlockPos) -> DVec3 {
    DVec3::new(
        f64::from(pos.x()) + 0.5,
        f64::from(pos.y()),
        f64::from(pos.z()) + 0.5,
    )
}
