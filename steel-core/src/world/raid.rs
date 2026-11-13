//! Vanilla `Raid` bookkeeping for bad omen and raid omen.
//!
//! Vanilla starts a raid when a player carrying bad omen walks into a village
//! (`ServerLevel.isVillage`), then scales the raid's wave count by the omen
//! amplifier. Steel models the raid as data owned by the world: a center block
//! position (the village bell) and the omen level, which is enough for the mob
//! effect behaviours and for `/raid`-style introspection later.

use std::sync::Arc;

use glam::IVec3;
use steel_utils::locks::SyncMutex;

use serde::{Deserialize, Serialize};
use steel_registry::mob_effect::instance::MobEffectInstance;
use steel_registry::vanilla_mob_effects;

use crate::entity::LivingEntity as _;
use crate::poi::OccupationStatus;
use crate::world::World;

/// Vanilla caps bad omen levels at `MAX_OMEN_LEVEL + 1` waves.
pub const MAX_OMEN_LEVEL: i32 = 4;

/// Vanilla village detection radius: `PoiManager.getInSquare(..., 64, ...)`.
const VILLAGE_SCAN_RADIUS: i32 = 64;

/// Vanilla `Raid.RAID_TIMEOUT`: a raid with no progress for this long fizzles out.
const RAID_TIMEOUT_TICKS: i32 = 48_000;

/// Rough per-wave duration used until raiders are simulated: vanilla spawns a
/// wave, waits for it to die, then spawns the next one.
const TICKS_PER_WAVE: i32 = 600;

/// Vanilla `HeroOfTheVillageTrigger` reward: the raid's total bad omen level
/// becomes the hero effect duration in ticks, `20 * 10 * level` seconds.
const HERO_EFFECT_TICKS_PER_OMEN_LEVEL: i32 = 4_800;

/// One active raid.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Raid {
    /// Village center (the bell POI).
    pub center: IVec3,
    /// Omen level that started the raid, `0..=MAX_OMEN_LEVEL`.
    pub omen_level: i32,
    /// Ticks the raid has been running.
    pub ticks_active: i32,
    /// Waves already cleared.
    pub waves_cleared: i32,
}

impl Raid {
    /// Returns the number of waves this raid spawns, matching vanilla
    /// `Raid.getNumGroups` (`omenLevel + 1`).
    #[must_use]
    pub const fn wave_count(&self) -> i32 {
        self.omen_level + 1
    }

    /// Returns true once every wave has been cleared.
    #[must_use]
    pub const fn is_victory(&self) -> bool {
        self.waves_cleared >= self.wave_count()
    }
}

/// Raids currently alive per world.
#[derive(Debug, Default)]
pub struct RaidManager {
    raids: SyncMutex<Vec<Raid>>,
}

impl RaidManager {
    /// Creates an empty manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a manager over raids loaded from disk.
    #[must_use]
    pub fn from_raids(raids: Vec<Raid>) -> Self {
        Self {
            raids: SyncMutex::new(raids),
        }
    }

    /// Snapshot of the raids in the form persisted to disk.
    #[must_use]
    pub fn to_persistent(&self) -> Vec<Raid> {
        self.raids.lock().clone()
    }

    /// Returns the raid whose center lies within the village radius of `pos`.
    #[must_use]
    pub fn raid_at(&self, pos: IVec3) -> Option<Raid> {
        self.raids
            .lock()
            .iter()
            .find(|raid| within_village_radius(raid.center, pos))
            .cloned()
    }

    /// Returns every active raid, in start order.
    #[must_use]
    pub fn raids(&self) -> Vec<Raid> {
        self.raids.lock().clone()
    }

    /// Starts (or extends) a raid centered on `center`, mirroring vanilla
    /// `Raid.raiseOmenLevel` when a raid already covers the village.
    ///
    /// Returns the resulting raid.
    pub fn raise_omen(&self, center: IVec3, omen_level: i32) -> Raid {
        let omen_level = omen_level.clamp(0, MAX_OMEN_LEVEL);
        let mut raids = self.raids.lock();
        if let Some(existing) = raids
            .iter_mut()
            .find(|raid| within_village_radius(raid.center, center))
        {
            existing.omen_level = existing.omen_level.max(omen_level);
            return existing.clone();
        }
        let raid = Raid {
            center,
            omen_level,
            ticks_active: 0,
            waves_cleared: 0,
        };
        raids.push(raid.clone());
        raid
    }

    /// Removes a raid once it is defeated, matching vanilla `Raid.stop`.
    pub fn stop(&self, center: IVec3) -> bool {
        let mut raids = self.raids.lock();
        let before = raids.len();
        raids.retain(|raid| !within_village_radius(raid.center, center));
        raids.len() != before
    }

    /// Advances every raid by one tick.
    ///
    /// Steel does not spawn raider entities yet, so a wave is treated as cleared
    /// once `TICKS_PER_WAVE` ticks have passed with no timeout. When the last
    /// wave clears, the raid stops and every player in the village earns the
    /// vanilla `hero_of_the_village` reward.
    pub fn tick(&self, world: &Arc<World>) {
        let mut victories: Vec<Raid> = Vec::new();
        {
            let mut raids = self.raids.lock();
            for raid in raids.iter_mut() {
                raid.ticks_active = raid.ticks_active.saturating_add(1);
                if raid.ticks_active > RAID_TIMEOUT_TICKS.saturating_mul(raid.wave_count()) {
                    continue;
                }
                let expected_waves =
                    (raid.ticks_active / TICKS_PER_WAVE).min(raid.wave_count());
                if expected_waves > raid.waves_cleared {
                    raid.waves_cleared = expected_waves;
                    if raid.is_victory() {
                        victories.push(raid.clone());
                    }
                }
            }
            raids.retain(|raid| !raid.is_victory());
        }
        for raid in victories {
            Self::award_hero_of_the_village(world, &raid);
        }
    }

    /// Vanilla `Raid.stop` victory path: every player inside the village becomes
    /// the hero of the village and fires its criterion.
    fn award_hero_of_the_village(world: &Arc<World>, raid: &Raid) {
        let center = DVec3::new(
            f64::from(raid.center.x),
            f64::from(raid.center.y),
            f64::from(raid.center.z),
        );
        let radius = f64::from(VILLAGE_SCAN_RADIUS);
        let players: Vec<_> = world
            .players
            .iter_players(|_uuid, player| {
                let position = player.position();
                let dx = position.x - center.x;
                let dz = position.z - center.z;
                if dx * dx + dz * dz > radius * radius {
                    return true;
                }
                // Vanilla gives the hero effect for 20 * 10 * omenLevel seconds.
                let duration = raid
                    .omen_level
                    .max(1)
                    .saturating_mul(HERO_EFFECT_TICKS_PER_OMEN_LEVEL);
                player.add_mob_effect(MobEffectInstance::simple(
                    &vanilla_mob_effects::HERO_OF_THE_VILLAGE,
                    duration,
                    0,
                ));
                player.trigger_advancement_with_context(
                    &steel_registry::vanilla_criteria_triggers::HERO_OF_THE_VILLAGE,
                    &crate::player::advancement::predicate::TriggerContext::at(
                        &player.get_world(),
                        player.block_position(),
                    ),
                );
                true
            })
            .collect();
        drop(players);
    }
}

/// Vanilla `Raid.isVillage` distance rule: the same radius village detection uses.
const fn within_village_radius(center: IVec3, pos: IVec3) -> bool {
    let dx = (center.x - pos.x).abs();
    let dz = (center.z - pos.z).abs();
    dx <= VILLAGE_SCAN_RADIUS && dz <= VILLAGE_SCAN_RADIUS
}

/// Returns whether `pos` stands in a village, i.e. a `meeting` POI (bell) is
/// nearby, mirroring vanilla `ServerLevel.isVillage`.
#[must_use]
pub fn is_village(world: &Arc<World>, pos: IVec3) -> bool {
    let Some(meeting_point_id) = meeting_point_poi_id() else {
        return false;
    };
    let storage = world.poi_storage.lock();
    !storage
        .get_in_horizontal_square(
            &|poi_type| poi_type == meeting_point_id,
            steel_utils::BlockPos::new(pos.x, pos.y, pos.z),
            VILLAGE_SCAN_RADIUS,
            OccupationStatus::Any,
        )
        .is_empty()
}

/// Returns the `meeting` POI type id, or `None` when the registry lacks it.
fn meeting_point_poi_id() -> Option<usize> {
    steel_registry::REGISTRY
        .poi_types
        .iter()
        .find(|(_, poi_type)| poi_type.key.path.as_ref() == "meeting")
        .map(|(id, _)| id)
}

/// [`village_center`] for callers holding a `&World` (mob-effect behaviours).
#[must_use]
pub fn village_center_world(world: &World, pos: steel_utils::BlockPos) -> Option<IVec3> {
    let meeting_point_id = meeting_point_poi_id()?;
    let storage = world.poi_storage.lock();
    let mut matches = storage.get_in_horizontal_square(
        &|poi_type| poi_type == meeting_point_id,
        pos,
        VILLAGE_SCAN_RADIUS,
        OccupationStatus::Any,
    );
    matches.sort_by_key(|(bell, _)| {
        let dx = bell.0.x - pos.0.x;
        let dz = bell.0.z - pos.0.z;
        dx * dx + dz * dz
    });
    matches.first().map(|(bell, _)| bell.0)
}

/// Returns the bell position of the nearest village, used as the raid center.
#[must_use]
pub fn village_center(world: &Arc<World>, pos: IVec3) -> Option<IVec3> {
    let meeting_point_id = meeting_point_poi_id()?;
    let storage = world.poi_storage.lock();
    let mut matches = storage.get_in_horizontal_square(
        &|poi_type| poi_type == meeting_point_id,
        steel_utils::BlockPos::new(pos.x, pos.y, pos.z),
        VILLAGE_SCAN_RADIUS,
        OccupationStatus::Any,
    );
    matches.sort_by_key(|(bell, _)| {
        let dx = bell.0.x - pos.x;
        let dz = bell.0.z - pos.z;
        dx * dx + dz * dz
    });
    matches.first().map(|(bell, _)| bell.0)
}
