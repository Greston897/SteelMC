//! Minimal `EnderDragonFight` state.
//!
//! Vanilla keeps one fight per End world: it owns the alive dragon, the crystal
//! ring, the exit portal and the respawn sequence. Steel needs the parts that
//! other systems observe today — which block position a crystal should beam at,
//! and whether destroying a crystal can start the respawn — so this module keeps
//! those facts per world and leaves wave logic to the dragon entity.

use glam::IVec3;
use steel_utils::locks::SyncMutex;

/// Vanilla exit portal position in the End (`0, 0` column).
pub const EXIT_PORTAL: IVec3 = IVec3::new(0, 0, 0);

/// One dragon fight.
#[derive(Debug, Default)]
pub struct DragonFight {
    /// Block position of the alive dragon, used as the crystal beam target.
    dragon_target: SyncMutex<Option<IVec3>>,
    /// Whether the respawn sequence has already been triggered.
    respawn_started: SyncMutex<bool>,
}

impl DragonFight {
    /// Creates a fight with no dragon recorded.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records the alive dragon's block position, so crystals relink their beam.
    pub fn set_dragon(&self, position: Option<IVec3>) {
        *self.dragon_target.lock() = position;
        if position.is_some() {
            *self.respawn_started.lock() = false;
        }
    }

    /// Returns the block position crystals should beam at, if a dragon is alive.
    #[must_use]
    pub fn dragon_target(&self) -> Option<IVec3> {
        *self.dragon_target.lock()
    }

    /// Vanilla `EnderDragonFight.tryRespawn`: destroying a crystal on the exit
    /// portal's column while no dragon is alive schedules a respawn.
    ///
    /// Returns `true` when this call started the respawn.
    pub fn try_respawn(&self, crystal_pos: IVec3) -> bool {
        if self.dragon_target().is_some() {
            return false;
        }
        if crystal_pos.x != EXIT_PORTAL.x || crystal_pos.z != EXIT_PORTAL.z {
            return false;
        }
        let mut started = self.respawn_started.lock();
        if *started {
            return false;
        }
        *started = true;
        true
    }
}
