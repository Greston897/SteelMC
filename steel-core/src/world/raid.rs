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

use crate::poi::OccupationStatus;
use crate::world::World;

/// Vanilla caps bad omen levels at `MAX_OMEN_LEVEL + 1` waves.
pub const MAX_OMEN_LEVEL: i32 = 4;

/// Vanilla village detection radius: `PoiManager.getInSquare(..., 64, ...)`.
const VILLAGE_SCAN_RADIUS: i32 = 64;

/// One active raid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Raid {
    /// Village center (the bell POI).
    pub center: IVec3,
    /// Omen level that started the raid, `0..=MAX_OMEN_LEVEL`.
    pub omen_level: i32,
    /// Ticks the raid has been running.
    pub ticks_active: i32,
}

impl Raid {
    /// Returns the number of waves this raid spawns, matching vanilla
    /// `Raid.getNumGroups` (`omenLevel + 1`).
    #[must_use]
    pub const fn wave_count(&self) -> i32 {
        self.omen_level + 1
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
    pub fn tick(&self) {
        for raid in self.raids.lock().iter_mut() {
            raid.ticks_active = raid.ticks_active.saturating_add(1);
        }
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
