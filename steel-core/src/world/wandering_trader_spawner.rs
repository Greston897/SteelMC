//! Vanilla `net.minecraft.world.entity.npc.wanderingtrader.WanderingTraderSpawner`.
//!
//! The spawner is a per-world custom spawner: it runs on a 1200-tick schedule,
//! counts down a 24000-tick delay and then rolls a chance that starts at 25% and
//! rises by 25 points (capped at 75%) on every failed attempt. A successful roll
//! still has to pass the `nextInt(10) == 0` gate before a position is searched
//! for, which is why the documented per-attempt chance is 2.5%/5%/7.5%.
//!
//! The spawner only runs in worlds that can hold traders at all: vanilla
//! registers it for the Overworld. Steel runs it for every world whose dimension
//! has skylight, which is the closest property it can test cheaply and keeps
//! Nether/End from spawning traders.

use std::sync::Arc;

use glam::DVec3;
use serde::{Deserialize, Serialize};

use crate::chunk::heightmap::HeightmapType;
use crate::entity::spawn::{EntitySpawnPlacement, EntitySpawnReason, EntitySpawnRequest};
use crate::entity::spawn_entity;
use crate::world::{LevelReader, World};
use steel_registry::{REGISTRY, entity_type::EntityTypeRef, vanilla_game_rules};
use steel_utils::{BlockPos, Identifier};

/// Vanilla `WanderingTraderSpawner.SPAWN_DELAY`: the counter is reset to this.
pub const SPAWN_DELAY_TICKS: i32 = 24_000;
/// Vanilla `WanderingTraderSpawner.spawnDelay` is decremented by this each pass.
pub const SPAWN_CHECK_INTERVAL: i32 = 1_200;
/// Vanilla starts the roll at 25 and advances it by 25 per failed attempt.
pub const INITIAL_SPAWN_CHANCE: i32 = 25;
/// Vanilla `WanderingTraderSpawner.MAX_SPAWN_CHANCE`.
pub const MAX_SPAWN_CHANCE: i32 = 75;
/// Vanilla `WanderingTraderSpawner` caps the horizontal search at 48 blocks.
const SEARCH_RADIUS: i32 = 48;
/// Vanilla searches `-48..=47` around the reference position on both axes.
const SEARCH_ATTEMPTS: usize = 10;
/// Vanilla `WanderingTrader` gets `DespawnDelay` 48000 on a natural spawn.
const DESPAWN_DELAY: i32 = 48_000;

/// Persisted schedule (vanilla `WanderingTraderSpawnDelay`/
/// `WanderingTraderSpawnChance` in `level.dat`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WanderingTraderSchedule {
    /// Ticks left until the next attempt (vanilla `spawnDelay`).
    pub spawn_delay: i32,
    /// Current roll chance in whole percent (vanilla `spawnChance`).
    pub spawn_chance: i32,
}

impl Default for WanderingTraderSchedule {
    fn default() -> Self {
        Self {
            spawn_delay: SPAWN_DELAY_TICKS,
            spawn_chance: INITIAL_SPAWN_CHANCE,
        }
    }
}

impl WanderingTraderSchedule {
    /// Vanilla `WanderingTraderSpawner.tick`: decrements the delay and, once it
    /// runs out, resets it and rolls for a spawn.
    ///
    /// Returns `true` when a spawn should be attempted this pass.
    pub fn tick(&mut self, ticks: i32, roll: i32) -> bool {
        self.spawn_delay -= ticks;
        if self.spawn_delay > 0 {
            return false;
        }
        self.spawn_delay = SPAWN_DELAY_TICKS;
        if roll <= self.spawn_chance {
            return true;
        }
        self.spawn_chance = (self.spawn_chance + INITIAL_SPAWN_CHANCE).min(MAX_SPAWN_CHANCE);
        false
    }

    /// Vanilla resets the chance to 25 after a trader actually appeared, and
    /// also when an attempt ran with no player to place it next to.
    pub const fn reset_chance(&mut self) {
        self.spawn_chance = INITIAL_SPAWN_CHANCE;
    }
}

/// Per-world wandering trader spawner.
#[derive(Debug)]
pub struct WanderingTraderSpawner {
    schedule: WanderingTraderSchedule,
    ticks_since_check: i32,
}

impl WanderingTraderSpawner {
    /// Creates a spawner from a persisted schedule.
    #[must_use]
    pub const fn from_schedule(schedule: WanderingTraderSchedule) -> Self {
        Self {
            schedule,
            ticks_since_check: 0,
        }
    }

    /// Returns the schedule, for persistence.
    #[must_use]
    pub const fn schedule(&self) -> WanderingTraderSchedule {
        self.schedule
    }

    /// Ticks the spawner once, matching vanilla `WanderingTraderSpawner.tick`.
    pub fn tick(&mut self, world: &Arc<World>) {
        // Vanilla gates the whole spawner on `spawn_wandering_traders` and only
        // registers it for dimensions where traders may exist at all.
        if !world.allows_wandering_traders() {
            return;
        }
        if !world.get_game_rule(&vanilla_game_rules::SPAWN_WANDERING_TRADERS) {
            return;
        }

        self.ticks_since_check += 1;
        if self.ticks_since_check < SPAWN_CHECK_INTERVAL {
            return;
        }
        self.ticks_since_check = 0;

        // Vanilla keeps at most one naturally spawned trader per world and
        // skips the roll entirely while one is still around, so the delay does
        // not advance while a trader is present.
        if world.has_natural_wandering_trader() {
            return;
        }

        let roll = rand::random_range(0..100);
        if !self.schedule.tick(SPAWN_CHECK_INTERVAL, roll) {
            return;
        }

        if !world.get_game_rule(&vanilla_game_rules::SPAWN_MOBS) {
            return;
        }

        let Some(player_position) = random_player_position(world) else {
            // Vanilla: "no player to spawn next to"; the chance is reset so the
            // next attempt starts the 25/50/75 ladder over.
            self.schedule.reset_chance();
            return;
        };

        // Vanilla picks the nearest `MEETING` point of interest (a bell) within
        // 48 blocks of the player as the reference position. Steel has no POI
        // storage yet, so the player position is the reference.
        if !self.try_spawn_trader(world, player_position) {
            return;
        }
        self.schedule.reset_chance();
    }

    /// Vanilla `tryToSpawnWanderingTrader`: one in ten attempts proceeds past
    /// the player pick, then ten surface positions are tried.
    fn try_spawn_trader(&self, world: &Arc<World>, reference: DVec3) -> bool {
        // Vanilla `nextInt(10) == 0`.
        if rand::random_range(0..10) != 0 {
            return false;
        }

        let Some(entity_type) = trader_entity_type() else {
            return false;
        };

        for _ in 0..SEARCH_ATTEMPTS {
            let x = reference.x.floor() as i32 + rand::random_range(-SEARCH_RADIUS..SEARCH_RADIUS);
            let z = reference.z.floor() as i32 + rand::random_range(-SEARCH_RADIUS..SEARCH_RADIUS);
            // The search only randomizes X/Z; vanilla always lands the trader on
            // the uppermost motion-blocking surface, never inside a cave.
            let y = world.height_at(HeightmapType::MotionBlockingNoLeaves, x, z);
            let pos = BlockPos::new(x, y, z);

            let request = EntitySpawnRequest {
                entity_type,
                placement: EntitySpawnPlacement::Block {
                    pos,
                    try_move_down: false,
                    moved_up: false,
                },
                // Vanilla spawns the event trader through `EntityType.spawn`.
                reason: EntitySpawnReason::Event,
                finalize_spawn: true,
                play_ambient_sound: false,
                item_stack: None,
                user_is_operator: false,
            };
            if let Ok(entity) = spawn_entity(world, request) {
                if let Some(trader) = entity
                    .as_ref()
                    .downcast_ref::<crate::entity::entities::WanderingTraderEntity>()
                {
                    // Vanilla `WanderingTraderSpawner`: the trader leaves after
                    // `DESPAWN_DELAY` ticks unless somebody is trading with it.
                    trader.set_despawn_delay(natural_despawn_delay());
                }
                return true;
            }
        }
        false
    }
}

/// The vanilla `minecraft:wandering_trader` entity type, if it is registered.
fn trader_entity_type() -> Option<EntityTypeRef> {
    let key = Identifier::vanilla_static("wandering_trader");
    REGISTRY.entity_types.by_key(&key)
}

/// Picks a random player position in `world`, matching vanilla's uniform pick.
fn random_player_position(world: &Arc<World>) -> Option<DVec3> {
    let count = world.players.len();
    if count == 0 {
        return None;
    }
    let target = rand::random_range(0..count);
    let mut index = 0;
    let mut chosen = None;
    world.players.iter_players(|_uuid, player| {
        if index == target {
            chosen = Some(player.position());
        }
        index += 1;
        true
    });
    chosen
}

/// The despawn delay a naturally spawned trader starts with (vanilla
/// `WanderingTrader.setDespawnDelay(48000)`).
#[must_use]
pub const fn natural_despawn_delay() -> i32 {
    DESPAWN_DELAY
}

#[cfg(test)]
mod tests {
    use super::{
        INITIAL_SPAWN_CHANCE, MAX_SPAWN_CHANCE, SPAWN_DELAY_TICKS, WanderingTraderSchedule,
    };

    #[test]
    fn a_failed_roll_advances_the_chance_by_a_quarter() {
        let mut schedule = WanderingTraderSchedule::default();
        // Roll above the chance: no spawn, chance climbs.
        assert!(!schedule.tick(SPAWN_DELAY_TICKS, 100));
        assert_eq!(schedule.spawn_chance, 50);
        assert!(!schedule.tick(SPAWN_DELAY_TICKS, 100));
        assert_eq!(schedule.spawn_chance, MAX_SPAWN_CHANCE);
        assert!(!schedule.tick(SPAWN_DELAY_TICKS, 100));
        assert_eq!(schedule.spawn_chance, MAX_SPAWN_CHANCE, "capped at 75");
    }

    #[test]
    fn a_successful_roll_attempts_and_resets_the_delay() {
        let mut schedule = WanderingTraderSchedule::default();
        assert!(schedule.tick(SPAWN_DELAY_TICKS, 0));
        assert_eq!(schedule.spawn_delay, SPAWN_DELAY_TICKS);
        schedule.reset_chance();
        assert_eq!(schedule.spawn_chance, INITIAL_SPAWN_CHANCE);
    }

    #[test]
    fn the_delay_only_expires_after_the_full_day() {
        let mut schedule = WanderingTraderSchedule::default();
        let mut attempts = 0;
        for _ in 0..(SPAWN_DELAY_TICKS / 1_200) {
            if schedule.tick(1_200, 100) {
                attempts += 1;
            }
        }
        assert_eq!(attempts, 1, "one roll per 24000 ticks");
    }
}
