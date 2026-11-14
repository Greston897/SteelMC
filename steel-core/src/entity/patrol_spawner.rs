//! Vanilla `net.minecraft.world.entity.monster.PatrolSpawner`.
//!
//! The second of vanilla's custom spawners. It acts once every 12000-13199
//! ticks, and even then only one time in five, only during the day, and only for
//! a player who is neither a spectator nor standing near a village. The patrol
//! itself is `ceil(effectiveDifficulty) + 1` pillagers on the
//! `MOTION_BLOCKING_NO_LEAVES` heightmap; the first one is the captain and wears
//! the ominous banner.
//!
//! Deviations from vanilla, both documented in `PARITY-26.3-RU.md`:
//!
//! * the "have monsters been peaceful long enough" check is the player's play
//!   time, because Steel has no per-world `inhabitedTime` yet and vanilla's
//!   answer for a fresh world is the same;
//! * the village check uses the `meeting` point of interest inside a 5×5×5
//!   section box, which is the block a vanilla village is anchored on.

use std::sync::Arc;

use steel_registry::stat::vanilla_custom_stats;
use steel_registry::vanilla_entities;
use steel_registry::vanilla_game_rules;
use steel_utils::types::Difficulty;
use steel_utils::{BlockPos, Identifier};

use crate::chunk::heightmap::HeightmapType;
use crate::chunk::light::LightLayer;
use crate::entity::spawn::{EntitySpawnPlacement, EntitySpawnReason, EntitySpawnRequest};
use crate::entity::{Entity as _, LivingEntity as _, spawn_entity};
use crate::inventory::equipment::EquipmentSlot;
use crate::player::Player;
use crate::poi::OccupationStatus;
use crate::world::World;

/// Vanilla `PatrolSpawner` first delay window in ticks.
const FIRST_DELAY: i32 = 12_000;
const DELAY_JITTER: i32 = 1_200;
/// Vanilla spawns a patrol on one attempt out of five.
const SPAWN_CHANCE: i32 = 5;
/// Vanilla `PatrolSpawner.spawn`: the captain is 24-48 blocks away per axis.
const MIN_DISTANCE: i32 = 24;
const MAX_DISTANCE: i32 = 48;
/// Vanilla requires a block light level of eight or lower.
const MAX_BLOCK_LIGHT: i32 = 8;
/// Vanilla wants twelve thousand ticks of peace before the first patrol, which
/// Steel approximates with the player's play time.
const MIN_WORLD_AGE: i32 = 12_000;

/// Per-world pillager patrol spawner state.
#[derive(Debug)]
pub struct PatrolSpawner {
    next_tick: i32,
}

impl Default for PatrolSpawner {
    fn default() -> Self {
        Self::new()
    }
}

impl PatrolSpawner {
    /// Creates a spawner with a fresh first delay.
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_tick: next_delay(),
        }
    }

    /// Ticks the spawner once, matching vanilla `PatrolSpawner.tick`.
    pub fn tick(&mut self, world: &Arc<World>) {
        if !world.allows_patrols() {
            return;
        }
        if !world.get_game_rule(&vanilla_game_rules::SPAWN_PATROLS) {
            return;
        }

        self.next_tick -= 1;
        if self.next_tick > 0 {
            return;
        }
        self.next_tick = next_delay();

        // Vanilla gives the day a reachable window too: a patrol is only tried
        // while monsters may spawn, which for the overworld is the night.
        if !world.is_day() && world.dimension_type.has_skylight {
            return;
        }
        if rand::random_range(0..SPAWN_CHANCE) != 0 {
            return;
        }

        let mut candidates = Vec::new();
        world.players.iter_players(|_uuid, player| {
            if patrol_candidate(player) {
                candidates.push(Arc::clone(player));
            }
            true
        });
        let Some(player) = candidates.first() else {
            return;
        };

        spawn_patrol(world, player);
    }
}

/// Vanilla `PatrolSpawner.nextTick`: `12000 + nextInt(1200)` ticks.
fn next_delay() -> i32 {
    FIRST_DELAY + rand::random_range(0..DELAY_JITTER)
}

/// Vanilla `PatrolSpawner` player filter: alive, not a spectator, enough play
/// time, and not inside a village.
fn patrol_candidate(player: &Player) -> bool {
    if player.game_mode() == steel_utils::types::GameType::Spectator || player.is_dead_or_dying() {
        return false;
    }
    // Vanilla gates on `canPillagerPatrolSpawn`: the world must have been
    // inhabited for twelve thousand ticks.
    if player.custom_stat(&vanilla_custom_stats::PLAY_TIME) < MIN_WORLD_AGE {
        return false;
    }
    !near_village(player)
}

/// Vanilla `ServerLevel.isCloseToVillage`: a five-section cube around the
/// position must be free of an inhabited meeting point.
fn near_village(player: &Player) -> bool {
    let Some(world) = player.level() else {
        return false;
    };
    let pos = player.block_position();
    let Some(meeting_id) = steel_registry::REGISTRY
        .poi_types
        .id_from_key(&Identifier::vanilla_static("meeting"))
    else {
        return false;
    };
    // Five sections per axis, so two sections of radius around the player.
    const SECTION_RADIUS_BLOCKS: i32 = 2 * 16;
    let storage = world.poi_storage.lock();
    !storage
        .get_in_horizontal_square(
            &|poi_type_id| poi_type_id == meeting_id,
            pos,
            SECTION_RADIUS_BLOCKS,
            OccupationStatus::Any,
        )
        .is_empty()
}

/// Vanilla `PatrolSpawner.spawnPatrol`.
fn spawn_patrol(world: &Arc<World>, player: &Player) {
    let base = player.position();
    // Vanilla picks each axis independently: 24-47 blocks, sign included.
    let offset_x = rand::random_range(MIN_DISTANCE..MAX_DISTANCE)
        * if rand::random::<bool>() { 1 } else { -1 };
    let offset_z = rand::random_range(MIN_DISTANCE..MAX_DISTANCE)
        * if rand::random::<bool>() { 1 } else { -1 };
    let target = BlockPos::new(
        base.x.floor() as i32 + offset_x,
        0,
        base.z.floor() as i32 + offset_z,
    );
    let surface = world.heightmap_pos(HeightmapType::MotionBlockingNoLeaves, target);

    if !monster_spawn_allowed(world, surface) {
        return;
    }

    // Vanilla `ceil(effectiveDifficulty) + 1`; Steel has no regional difficulty,
    // so the world difficulty id stands in for the effective value.
    let count = usize::try_from(effective_difficulty(world).ceil() as i32 + 1)
        .unwrap_or(1)
        .max(1);

    for index in 0..count {
        // Vanilla spreads the followers by `nextInt(5) - nextInt(5)` on each
        // axis around the previous pillager, so the group stays together.
        let spread_x = rand::random_range(0..5) - rand::random_range(0..5);
        let spread_z = rand::random_range(0..5) - rand::random_range(0..5);
        let pos = if index == 0 {
            surface
        } else {
            let previous =
                BlockPos::new(surface.x() + spread_x, surface.y(), surface.z() + spread_z);
            world.heightmap_pos(HeightmapType::MotionBlockingNoLeaves, previous)
        };
        let Some(entity) = spawn_pillager(world, pos) else {
            // Vanilla gives up on the whole patrol once a member cannot spawn.
            break;
        };
        if index == 0 {
            mark_captain(&entity);
        }
    }
}

/// Vanilla `PatrolSpawner.spawnPillager` preconditions: the spawn block is a
/// valid empty spawn block, the block below is not air, and the *block* light is
/// eight or lower. Unlike ordinary monster spawning a patrol does not roll the
/// sky light, because it only runs during the day.
fn monster_spawn_allowed(world: &Arc<World>, pos: BlockPos) -> bool {
    use steel_registry::blocks::block_state_ext::BlockStateExt as _;

    if !world.get_block_state(pos).is_air() || world.get_block_state(pos.below()).is_air() {
        return false;
    }
    i32::from(world.light_value_at(LightLayer::Block, pos)) <= MAX_BLOCK_LIGHT
}

/// Spawns one pillager at `pos`, or `None` when the spawn was refused.
fn spawn_pillager(world: &Arc<World>, pos: BlockPos) -> Option<crate::entity::SharedEntity> {
    let request = EntitySpawnRequest {
        entity_type: &vanilla_entities::PILLAGER,
        placement: EntitySpawnPlacement::Block {
            pos,
            try_move_down: false,
            moved_up: false,
        },
        reason: EntitySpawnReason::Patrol,
        finalize_spawn: true,
        play_ambient_sound: false,
        item_stack: None,
        user_is_operator: false,
    };
    spawn_entity(world, request).ok()
}

/// Vanilla `PatrolSpawner.spawnPatrol` marks the first pillager as the patrol
/// captain: it wears the ominous banner.
///
/// Steel has no `PatrolLeader` entity flag yet (the field is client-visible
/// metadata, not behaviour the server reads), so the banner is what distinguishes
/// the captain today; the flag is the remaining gap.
fn mark_captain(entity: &crate::entity::SharedEntity) {
    entity.with_equipment_slot_mut(EquipmentSlot::Head, &mut |head| {
        if head.is_empty() {
            *head = steel_registry::item_stack::ItemStack::with_count(
                &*steel_registry::vanilla_items::WHITE_BANNER,
                1,
            );
        }
    });
}

/// Vanilla `Level.getCurrentDifficultyAt(...).getEffectiveDifficulty()`, without
/// the regional and day-scale terms Steel does not model.
fn effective_difficulty(world: &Arc<World>) -> f64 {
    vanilla_effective_difficulty(world.difficulty())
}

/// The difficulty half of vanilla's effective difficulty.
fn vanilla_effective_difficulty(difficulty: Difficulty) -> f64 {
    match difficulty {
        Difficulty::Peaceful => 0.0,
        Difficulty::Easy => 1.0,
        Difficulty::Normal => 2.0,
        Difficulty::Hard => 3.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_delay_stays_inside_the_vanilla_window() {
        for _ in 0..1_000 {
            let delay = next_delay();
            assert!(
                (FIRST_DELAY..FIRST_DELAY + DELAY_JITTER).contains(&delay),
                "delay {delay} outside vanilla's 12000-13199 window"
            );
        }
    }

    #[test]
    fn the_effective_difficulty_matches_the_vanilla_ids() {
        // Vanilla ids: peaceful 0, easy 1, normal 2, hard 3; a patrol is
        // `ceil(value) + 1`, so the group sizes are 1, 2, 3 and 4.
        let group = |difficulty| {
            usize::try_from(vanilla_effective_difficulty(difficulty).ceil() as i32 + 1)
                .unwrap_or(1)
                .max(1)
        };
        assert_eq!(group(Difficulty::Peaceful), 1);
        assert_eq!(group(Difficulty::Easy), 2);
        assert_eq!(group(Difficulty::Normal), 3);
        assert_eq!(group(Difficulty::Hard), 4);
    }
}
