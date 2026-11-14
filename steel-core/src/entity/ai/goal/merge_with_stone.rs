//! Vanilla `Silverfish.SilverfishMergeWithStoneGoal`: an unnoticed silverfish
//! slips back into the infested block it came from.
//!
//! Vanilla keeps the plain↔infested mapping on `InfestedBlock`
//! (`InfestedBlock.getHostBlock`), which is why this goal asks the block
//! behavior instead of hardcoding a table. The merge only happens while no
//! player can see the silverfish, so an infested block stays a trap instead of
//! quietly despawning its occupant.

use glam::DVec3;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::particle_type::ParticleData;
use steel_registry::vanilla_particle_types;
use steel_utils::types::UpdateFlags;

use super::selector::{Goal, GoalControls};
use crate::entity::{Entity, PathfinderMob, RemovalReason, SharedEntity};
use crate::world::{ClipBlockShape, ClipFluid, LevelAccessor, World};

/// Vanilla rolls `nextInt(adjustedTickDelay(20))` before every attempt.
const MERGE_CHECK_INTERVAL: i32 = 20;
/// How many ticks the goal must have been running before it may merge. Keeps a
/// silverfish that just popped out of a block from vanishing straight back.
const MIN_TICKS_BEFORE_MERGE: i32 = 20;
/// Vanilla `Silverfish.SilverfishMergeWithStoneGoal` looks for watchers within
/// the standard hostile-mob follow range squared.
const WATCHER_RANGE_SQR: f64 = 256.0;

pub(crate) struct SilverfishMergeWithStoneGoal {
    check_cooldown: i32,
    ticks_running: i32,
}

impl SilverfishMergeWithStoneGoal {
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            check_cooldown: 0,
            ticks_running: 0,
        }
    }
}

/// Returns `true` when any player has unobstructed line of sight to `target`.
fn is_watched(world: &World, target: DVec3) -> bool {
    let mut watched = false;
    world.players.iter_players(|_uuid, player| {
        let eye = player.position() + DVec3::new(0.0, 1.62, 0.0);
        if eye.distance_squared(target) > WATCHER_RANGE_SQR {
            return true;
        }
        let hit = world.clip(eye, target, ClipBlockShape::Collider, ClipFluid::None);
        if hit.is_miss() {
            watched = true;
            false
        } else {
            true
        }
    });
    watched
}

impl Goal for SilverfishMergeWithStoneGoal {
    fn controls(&self) -> GoalControls {
        GoalControls::MOVE
    }

    fn can_use(&mut self, mob: &dyn PathfinderMob) -> bool {
        if self.check_cooldown > 0 {
            self.check_cooldown -= 1;
            return false;
        }
        self.check_cooldown = MERGE_CHECK_INTERVAL;

        let Some(world) = mob.level() else {
            return false;
        };
        // Vanilla bails out when a player is watching: hiding only works while
        // the silverfish is unnoticed.
        if is_watched(&world, mob.position()) {
            return false;
        }
        // Vanilla requires an infested block underfoot.
        world
            .get_block_state(mob.block_position().below())
            .get_block()
            .host_block()
            .is_some()
    }

    fn can_continue_to_use(&mut self, mob: &dyn PathfinderMob) -> bool {
        self.can_use(mob)
    }

    fn start(&mut self, _mob: &dyn PathfinderMob) {
        self.ticks_running = 0;
    }

    fn stop(&mut self, _mob: &dyn PathfinderMob) {
        self.ticks_running = 0;
    }

    fn requires_update_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, mob: &dyn PathfinderMob, _entity: &SharedEntity) {
        self.ticks_running += 1;
        if self.ticks_running < MIN_TICKS_BEFORE_MERGE {
            return;
        }

        let Some(world) = mob.level() else {
            return;
        };
        let pos = mob.block_position().below();
        let Some(host) = world.get_block_state(pos).get_block().host_block() else {
            return;
        };

        // Vanilla swaps the block for its plain form, spawns the poof particle
        // and removes the silverfish — `infested_stone` becomes `stone`, so a
        // player who later mines it is no longer ambushed.
        world.set_block_state(pos, host.default_state(), UpdateFlags::UPDATE_ALL);
        world.send_particles(
            ParticleData::simple(&vanilla_particle_types::POOF),
            mob.position(),
            1,
            DVec3::ZERO,
            0.0,
        );
        mob.set_removed(RemovalReason::Discarded);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_constants_match_vanilla() {
        // Vanilla `SilverfishMergeWithStoneGoal` uses `adjustedTickDelay(20)`.
        assert_eq!(MERGE_CHECK_INTERVAL, 20);
        assert_eq!(MIN_TICKS_BEFORE_MERGE, 20);
    }
}
