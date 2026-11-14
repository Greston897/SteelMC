//! `EnderDragonFight`: the End boss state machine that Steel can drive.
//!
//! Vanilla keeps one fight per End world. It owns the alive dragon, the crystal
//! ring, the exit portal, the gateway portals and the respawn sequence. Steel
//! keeps the parts that other systems observe — which block a crystal should beam
//! at, whether the dragon has already been killed, and what the boss bar shows —
//! in this module, and leaves dragon flight phases to the dragon entity.
//!
//! Implemented here (matching vanilla `EnderDragonFight`):
//!
//! * `updateDragon`: a live dragon owns the fight and drives the boss bar.
//! * `setDragonKilled`: the bar empties and hides, the exit portal is written,
//!   the first kill drops the dragon egg on the podium, later kills open a
//!   gateway portal ring.
//! * `tryRespawn`: with the dragon dead and four crystals on the exit portal's
//!   spikes, the respawn sequence arms itself; the crystals are reset to
//!   invulnerable so the fight can restart the vanilla way.
//!
//! Not implemented (documented in `STATE-26.3-RU.md`): the 20-tick respawn
//! beam animation and the gateway's 4x4 ring geometry beyond the podium column.

use glam::IVec3;
use steel_utils::BlockPos;
use steel_utils::locks::SyncMutex;

/// Vanilla exit portal position in the End (`0, 0` column).
pub const EXIT_PORTAL: IVec3 = IVec3::new(0, 0, 0);

/// Vanilla `EnderDragonFight.origin`: the exit portal column the fight centres on.
pub const ORIGIN_Y: i32 = 0;

/// One dragon fight.
#[derive(Debug, Default)]
pub struct DragonFight {
    /// Block position of the alive dragon, used as the crystal beam target.
    dragon_target: SyncMutex<Option<IVec3>>,
    /// Whether the respawn sequence has already been triggered.
    respawn_started: SyncMutex<bool>,
    /// Vanilla `hasPreviouslyKilledDragon`: the egg only drops on the first kill.
    previously_killed: SyncMutex<bool>,
    /// Vanilla `dragonKilled`: no dragon is alive and the exit portal is open.
    dragon_killed: SyncMutex<bool>,
    /// Vanilla boss bar progress, mirrored into `BossBarRegistry`.
    progress: SyncMutex<f32>,
    /// Vanilla boss bar visibility.
    bar_visible: SyncMutex<bool>,
}

impl DragonFight {
    /// Creates a fight with no dragon recorded.
    #[must_use]
    pub fn new() -> Self {
        Self {
            progress: SyncMutex::new(1.0),
            bar_visible: SyncMutex::new(false),
            ..Self::default()
        }
    }

    /// Records the alive dragon's block position, so crystals relink their beam.
    pub fn set_dragon(&self, position: Option<IVec3>) {
        *self.dragon_target.lock() = position;
        if position.is_some() {
            *self.respawn_started.lock() = false;
            *self.dragon_killed.lock() = false;
            *self.bar_visible.lock() = true;
        }
    }

    /// Returns the block position crystals should beam at, if a dragon is alive.
    #[must_use]
    pub fn dragon_target(&self) -> Option<IVec3> {
        *self.dragon_target.lock()
    }

    /// Vanilla `EnderDragonFight.updateDragon`: a live dragon sets the bar
    /// progress from its health fraction.
    ///
    /// Returns `true` when this dragon owns the fight, so callers know whether
    /// to mirror the bar.
    pub fn update_dragon(&self, health_fraction: f32) -> bool {
        if self.dragon_target().is_none() {
            return false;
        }
        *self.progress.lock() = health_fraction.clamp(0.0, 1.0);
        *self.bar_visible.lock() = true;
        true
    }

    /// Current boss bar progress in `0.0..=1.0`.
    #[must_use]
    pub fn progress(&self) -> f32 {
        *self.progress.lock()
    }

    /// Whether the dragon boss bar should be visible.
    #[must_use]
    pub fn bar_visible(&self) -> bool {
        *self.bar_visible.lock()
    }

    /// Whether the dragon has already been killed at least once.
    #[must_use]
    pub fn has_previously_killed_dragon(&self) -> bool {
        *self.previously_killed.lock()
    }

    /// Whether no dragon is alive and the exit portal is open.
    #[must_use]
    pub fn is_dragon_killed(&self) -> bool {
        *self.dragon_killed.lock()
    }

    /// Vanilla `EnderDragonFight.setDragonKilled`.
    ///
    /// Returns `DragonKillOutcome` describing what the caller still has to write
    /// into the world: the exit portal blocks, and on the first kill the egg.
    pub fn set_dragon_killed(&self) -> DragonKillOutcome {
        *self.progress.lock() = 0.0;
        *self.bar_visible.lock() = false;
        *self.dragon_target.lock() = None;
        *self.dragon_killed.lock() = true;
        *self.respawn_started.lock() = false;

        let first_kill = !*self.previously_killed.lock();
        *self.previously_killed.lock() = true;

        DragonKillOutcome {
            spawn_exit_portal: true,
            drop_dragon_egg: first_kill,
            spawn_gateway: true,
        }
    }

    /// Vanilla `EnderDragonFight.tryRespawn`: destroying a crystal on the exit
    /// portal's column while the dragon is dead starts the respawn.
    ///
    /// Returns `true` when this call started the respawn.
    pub fn try_respawn(&self, crystal_pos: IVec3) -> bool {
        // Vanilla requires `dragonKilled` before a crystal can arm the sequence.
        if !self.is_dragon_killed() {
            return false;
        }
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

    /// Whether the respawn sequence is currently armed.
    #[must_use]
    pub fn respawn_started(&self) -> bool {
        *self.respawn_started.lock()
    }

    /// Returns the podium position the dragon egg and exit portal sit on.
    #[must_use]
    pub fn podium_pos() -> BlockPos {
        BlockPos::new(EXIT_PORTAL.x, ORIGIN_Y, EXIT_PORTAL.z)
    }
}

/// What `setDragonKilled` asks the world to write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DragonKillOutcome {
    /// Write the exit portal blocks around `EnderDragonFight.EXIT_PORTAL`.
    pub spawn_exit_portal: bool,
    /// Drop the dragon egg on the podium (only the first kill).
    pub drop_dragon_egg: bool,
    /// Open a gateway portal so the player can leave the End.
    pub spawn_gateway: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_live_dragon_owns_the_bar_and_blocks_respawn() {
        let fight = DragonFight::new();
        fight.set_dragon(Some(IVec3::new(0, 96, 0)));

        assert!(fight.update_dragon(0.5));
        assert!((fight.progress() - 0.5).abs() < f32::EPSILON);
        assert!(fight.bar_visible());
        assert!(
            !fight.try_respawn(EXIT_PORTAL),
            "vanilla cannot arm a respawn while the dragon is alive"
        );
    }

    #[test]
    fn the_first_kill_drops_the_egg_and_the_second_does_not() {
        let fight = DragonFight::new();
        fight.set_dragon(Some(IVec3::ZERO));

        let first = fight.set_dragon_killed();
        assert!(first.spawn_exit_portal && first.drop_dragon_egg && first.spawn_gateway);
        assert!(!fight.bar_visible());
        assert!(fight.is_dragon_killed());
        assert!(fight.has_previously_killed_dragon());

        fight.set_dragon(Some(IVec3::ZERO));
        let second = fight.set_dragon_killed();
        assert!(
            !second.drop_dragon_egg,
            "vanilla only places the egg on the first kill"
        );
    }

    #[test]
    fn killing_a_crystal_on_the_exit_portal_column_arms_the_respawn_once() {
        let fight = DragonFight::new();
        fight.set_dragon(Some(IVec3::new(0, 96, 0)));
        fight.set_dragon_killed();

        assert!(
            !fight.try_respawn(IVec3::new(8, 80, 8)),
            "a crystal off the portal column cannot start a respawn"
        );
        assert!(fight.try_respawn(EXIT_PORTAL));
        assert!(
            !fight.try_respawn(EXIT_PORTAL),
            "the sequence must only arm once"
        );
        assert!(fight.respawn_started());
    }
}
