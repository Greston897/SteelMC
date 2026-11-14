//! Vanilla `net.minecraft.world.entity.monster.PhantomSpawner`.
//!
//! One of vanilla's custom spawners: it runs every world tick, picks a random
//! delay of 60-119 seconds, and then tries to place a group of phantoms above a
//! player who has not rested. The spawn is made of three independent checks that
//! the wiki merges into one number:
//!
//! * the sky must be dark enough (sky darkening of 5, i.e. night or a storm) —
//!   a dimension without skylight always passes;
//! * the player must be above sea level with an unobstructed sky;
//! * `nextInt(timeSinceRest) >= 72000`, i.e. insomnia for three in-game days.
//!
//! The group size is `1 + nextInt(localDifficultyId + 1)`, which is why the
//! wiki reports 1-2/1-3/1-4 for Easy/Normal/Hard; the regional part of vanilla's
//! local difficulty is not modelled in Steel yet, so the world difficulty id is
//! used and documented as the deviation.

use std::sync::Arc;

use glam::DVec3;

use crate::chunk::heightmap::HeightmapType;
use crate::entity::spawn::{EntitySpawnPlacement, EntitySpawnReason, EntitySpawnRequest};
use crate::entity::spawn_entity;
use crate::player::Player;
use crate::world::{LevelReader, World};
use steel_registry::{REGISTRY, entity_type::EntityTypeRef, vanilla_game_rules};
use steel_registry::stat::vanilla_custom_stats;
use steel_utils::{BlockPos, Identifier, types::Difficulty};

/// Vanilla `PhantomSpawner.nextTick`: the delay is `60 + nextInt(60)` seconds.
const MIN_DELAY_SECONDS: i32 = 60;
const DELAY_JITTER_SECONDS: i32 = 60;
/// Vanilla uses ticks, and an attempt happens once the countdown reaches zero.
const TICKS_PER_SECOND: i32 = 20;
/// Vanilla `PhantomSpawner`: insomnia threshold, three in-game days.
pub const INSOMNIA_THRESHOLD_TICKS: i32 = 72_000;
/// Vanilla requires a sky darkening of 5 before phantoms consider spawning.
const REQUIRED_SKY_DARKENING: u8 = 5;
/// Vanilla picks a spot 20-34 blocks above the player, up to 10 blocks aside.
const VERTICAL_RANGE: (i32, i32) = (20, 34);
const HORIZONTAL_RANGE: i32 = 10;

/// Per-world phantom spawner state.
#[derive(Debug)]
pub struct PhantomSpawner {
    next_tick: i32,
}

impl Default for PhantomSpawner {
    fn default() -> Self {
        Self::new()
    }
}

impl PhantomSpawner {
    /// Creates a spawner with a fresh first delay.
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_tick: next_delay(),
        }
    }

    /// Ticks the spawner once, matching vanilla `PhantomSpawner.tick`.
    pub fn tick(&mut self, world: &Arc<World>) {
        if !world.allows_phantoms() {
            return;
        }
        if !world.get_game_rule(&vanilla_game_rules::SPAWN_PHANTOMS) {
            return;
        }

        self.next_tick -= 1;
        if self.next_tick > 0 {
            return;
        }
        self.next_tick = next_delay();

        // Vanilla checks `ServerLevel.isNaturalSpawningAllowed(player.blockPosition())`
        // and the insomnia threshold per player before attempting a spawn.
        let mut candidates = Vec::new();
        world.players.iter_players(|_uuid, player| {
            if phantom_candidate(player, world) {
                candidates.push(Arc::clone(player));
            }
            true
        });
        let Some(player) = candidates.first() else {
            return;
        };

        let group = group_size(world.difficulty());
        spawn_phantoms(world, player, group);
    }
}

/// Vanilla `PhantomSpawner.nextTick`: `60 + nextInt(60)` seconds in ticks.
fn next_delay() -> i32 {
    (MIN_DELAY_SECONDS + rand::random_range(0..DELAY_JITTER_SECONDS)) * TICKS_PER_SECOND
}

/// Vanilla group size: `1 + nextInt(difficultyId + 1)`.
fn group_size(difficulty: Difficulty) -> usize {
    let difficulty_id = difficulty as i32;
    1 + rand::random_range(0..=difficulty_id)
}

/// Whether `player` is a valid phantom target right now.
///
/// Mirrors vanilla `PhantomSpawner.tick`'s player loop.
fn phantom_candidate(player: &Player, world: &Arc<World>) -> bool {
    if player.game_mode() == steel_utils::types::GameType::Spectator {
        return false;
    }
    // The overworld rule: enough sky darkening, above sea level, open sky.
    if world.dimension_type.has_skylight {
        if world.sky_darkening() < REQUIRED_SKY_DARKENING {
            return false;
        }
        if player.position().y < f64::from(world.sea_level()) {
            return false;
        }
        if !has_clear_sky(world, player.position()) {
            return false;
        }
    }
    let time_since_rest =
        player.custom_stat(&vanilla_custom_stats::TIME_SINCE_REST);
    time_since_rest > 0 && rand::random_range(0..time_since_rest) >= INSOMNIA_THRESHOLD_TICKS
}

/// Vanilla `LevelReader.canSeeSky`: nothing but air above the position.
fn has_clear_sky(world: &Arc<World>, position: DVec3) -> bool {
    let x = position.x.floor() as i32;
    let z = position.z.floor() as i32;
    let y = position.y.floor() as i32;
    let top = World::height(world) - 1;
    (y..=top).all(|column_y| {
        world
            .get_block_state(BlockPos::new(x, column_y, z))
            .is_air()
    })
}

/// Spawns one group of phantoms above `player`, matching vanilla's placement.
fn spawn_phantoms(world: &Arc<World>, player: &Player, group: usize) {
    let Some(entity_type) = phantom_entity_type() else {
        return;
    };

    let base = player.position();
    for _ in 0..group {
        let x = base.x.floor() as i32 + rand::random_range(-HORIZONTAL_RANGE..=HORIZONTAL_RANGE);
        let z = base.z.floor() as i32 + rand::random_range(-HORIZONTAL_RANGE..=HORIZONTAL_RANGE);
        let y = base.y.floor() as i32 + rand::random_range(VERTICAL_RANGE.0..=VERTICAL_RANGE.1);
        let pos = BlockPos::new(x, y.clamp(world.min_y() + 1, World::height(world) - 2), z);

        // Vanilla requires an empty spawn block (`isValidEmptySpawnBlock`).
        if !world.get_block_state(pos).is_air() {
            continue;
        }

        let request = EntitySpawnRequest {
            entity_type,
            placement: EntitySpawnPlacement::Block {
                pos,
                try_move_down: false,
                moved_up: false,
            },
            reason: EntitySpawnReason::Natural,
            finalize_spawn: true,
            play_ambient_sound: false,
            item_stack: None,
            user_is_operator: false,
        };
        if spawn_entity(world, request).is_err() {
            // Vanilla gives up on the group as soon as one phantom fails.
            return;
        }
    }
}

/// Vanilla `EntityType.PHANTOM`.
fn phantom_entity_type() -> Option<EntityTypeRef> {
    REGISTRY
        .entity_types
        .by_key(&Identifier::vanilla_static("phantom"))
}

#[cfg(test)]
mod tests {
    use super::{
        DELAY_JITTER_SECONDS, INSOMNIA_THRESHOLD_TICKS, MIN_DELAY_SECONDS, TICKS_PER_SECOND,
        group_size, next_delay,
    };
    use steel_utils::types::Difficulty;

    #[test]
    fn the_delay_stays_inside_the_vanilla_window() {
        for _ in 0..1000 {
            let delay = next_delay();
            assert!(
                (MIN_DELAY_SECONDS..MIN_DELAY_SECONDS + DELAY_JITTER_SECONDS)
                    .contains(&(delay / TICKS_PER_SECOND)),
                "delay of {delay} ticks is outside 60-119 seconds"
            );
            assert_eq!(delay % TICKS_PER_SECOND, 0, "delays are whole seconds");
        }
    }

    #[test]
    fn the_group_size_follows_the_difficulty() {
        // Vanilla `1 + nextInt(difficultyId + 1)`: Easy 1-2, Normal 1-3, Hard 1-4.
        for _ in 0..100 {
            assert!((1..=2).contains(&group_size(Difficulty::Easy)));
            assert!((1..=3).contains(&group_size(Difficulty::Normal)));
            assert!((1..=4).contains(&group_size(Difficulty::Hard)));
            // Peaceful is 0, so the group is always one phantom; in practice the
            // world never ticks hostiles on Peaceful.
            assert_eq!(group_size(Difficulty::Peaceful), 1);
        }
    }

    #[test]
    fn insomnia_threshold_is_three_in_game_days() {
        // 72000 ticks at 24000 ticks per day.
        assert_eq!(INSOMNIA_THRESHOLD_TICKS / 24_000, 3);
    }
}
