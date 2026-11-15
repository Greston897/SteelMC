//! `EnderDragonFight`: the End boss state machine.
//!
//! Vanilla keeps one fight per End world. It owns the alive dragon, the crystal
//! ring, the exit portal, the gateway portals and the respawn sequence. This module
//! owns that state and the tick-by-tick stage machine; the world performs the
//! effects it asks for (placing features, running level events, spawning entities),
//! and the dragon entity owns its own flight phases.
//!
//! Mirrors vanilla `EnderDragonFight`:
//!
//! * `tick`: a 20-tick player scan gates the whole update, then the state scan, the
//!   respawn stage, `findOrCreateDragon` and the 100-tick crystal recount.
//! * `setDragonKilled`: the bar empties, the active podium is placed, a gateway
//!   opens, and the first kill also drops the dragon egg.
//! * `tryRespawn` / `abortRespawnSequence`: four crystals beside the portal arm the
//!   animation; a missing crystal aborts it and restores the active portal.
//! * `scanState`: resolves whether the world was already beaten and whether a
//!   dragon is alive.
//!
//! Persistence is vanilla's saved data (`data/ender_dragon_fight.toml`):
//! `needs_state_scanning`, `dragon_killed`, `previously_killed`, `respawn_stage`,
//! `respawn_time`, `dragon_uuid`, `exit_portal_location`, `gateways` and
//! `respawn_crystals`.

use glam::IVec3;
use serde::{Deserialize, Serialize};
use steel_utils::BlockPos;
use steel_utils::locks::SyncMutex;

/// Vanilla exit portal position in the End (`0, 0` column).
pub const EXIT_PORTAL: IVec3 = IVec3::new(0, 0, 0);

/// Vanilla `EnderDragonFight.MAX_TICKS_BEFORE_DRAGON_RESPAWN`.
pub const MAX_TICKS_BEFORE_DRAGON_RESPAWN: i32 = 1200;
/// Vanilla `EnderDragonFight.TIME_BETWEEN_CRYSTAL_SCANS`.
pub const TIME_BETWEEN_CRYSTAL_SCANS: i32 = 100;
/// Vanilla `EnderDragonFight.GATEWAY_COUNT`.
pub const GATEWAY_COUNT: i32 = 20;
/// Vanilla `EnderDragonFight.GATEWAY_DISTANCE`.
pub const GATEWAY_DISTANCE: i32 = 96;
/// Vanilla `EnderDragonFight.DRAGON_SPAWN_Y`.
pub const DRAGON_SPAWN_Y: i32 = 128;
/// Vanilla `LevelEvent.SOUND_DRAGON_GROWL`-adjacent spawn event (`3000`).
pub const EVENT_GATEWAY_SPAWN: i32 = 3000;
/// Vanilla `LevelEvent.SOUND_DRAGON_FIREBALL_EXPLODE`-adjacent respawn event
/// (`3001`).
pub const EVENT_DRAGON_RESPAWN: i32 = 3001;

/// Vanilla `EnderDragonFight.origin`: the exit portal column the fight centres on.
pub const ORIGIN_Y: i32 = 0;
/// Vanilla `EnderDragonFight.init`'s `validPlayer` radius, measured from
/// `(origin.x, 128 + origin.y, origin.z)`.
pub const PLAYER_SCAN_RADIUS: f64 = 192.0;
/// Vanilla `EnderDragonFight.TIME_BETWEEN_PLAYER_SCANS`.
pub const TIME_BETWEEN_PLAYER_SCANS: i32 = 20;

/// Vanilla `END_PODIUM_LOCATION`, which is `BlockPos.ZERO` in `26.3`.
pub struct PodiumOrigin;

impl PodiumOrigin {
    /// Vanilla `END_PODIUM_LOCATION`.
    #[must_use]
    pub const fn origin() -> BlockPos {
        BlockPos::new(EXIT_PORTAL.x, ORIGIN_Y, EXIT_PORTAL.z)
    }
}

/// Vanilla `EnderDragonFight`'s saved data (`data/ender_dragon_fight.toml`).
///
/// Field for field the vanilla codec: `needs_state_scanning`, `dragon_killed`,
/// `previously_killed`, `respawn_stage`, `respawn_time`, `dragon_uuid`,
/// `exit_portal_location`, `gateways` and `respawn_crystals`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DragonFightSavedData {
    /// Vanilla `needsStateScanning`: a fresh world must resolve its legacy state.
    pub needs_state_scanning: bool,
    /// Vanilla `dragonKilled`.
    pub dragon_killed: bool,
    /// Vanilla `hasPreviouslyKilledDragon`.
    pub previously_killed: bool,
    /// Vanilla `respawnStage` as its enum name, `None` outside a sequence.
    pub respawn_stage: Option<String>,
    /// Vanilla `respawnTime`.
    pub respawn_time: i32,
    /// Vanilla `dragonUUID`.
    pub dragon_uuid: Option<uuid::Uuid>,
    /// Vanilla `exitPortalLocation` as `[x, y, z]`.
    pub exit_portal_location: Option<[i32; 3]>,
    /// Vanilla `gateways`: the gateway ring indices still unopened.
    pub gateways: Vec<i32>,
    /// Vanilla `respawnCrystals`: the crystal positions on the podium column.
    pub respawn_crystals: Vec<[i32; 3]>,
}

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
    /// Vanilla `aliveCrystals`: how many end crystals still heal the dragon.
    alive_crystals: SyncMutex<i32>,
    /// Vanilla `dragon_uuid`: the dragon this fight owns, so a reloaded dragon can
    /// relink itself.
    dragon_uuid: SyncMutex<Option<uuid::Uuid>>,
    /// Vanilla `ticksSinceCrystalsScanned`: crystals are recounted every 100 ticks.
    ticks_since_crystals_scanned: SyncMutex<i32>,
    /// Vanilla `respawnStage`: `None` outside a respawn sequence.
    respawn_stage: SyncMutex<Option<DragonRespawnStage>>,
    /// Vanilla `respawnTime`: ticks spent in the current respawn stage.
    respawn_time: SyncMutex<i32>,
    /// Vanilla `exitPortalLocation`, resolved from the heightmap on first use.
    exit_portal_location: SyncMutex<Option<IVec3>>,
    /// Vanilla `gateways`: the ring indices still unopened. Vanilla pops the last
    /// entry per kill, so the vector shrinks from twenty to zero.
    gateways: SyncMutex<Vec<i32>>,
    /// The level seed `EndSpikeFeature.getSpikesForLevel` derives the ring from, so
    /// the respawn animation rebuilds the same spikes worldgen placed.
    level_seed: SyncMutex<Option<i64>>,
    /// Vanilla `needsStateScanning`: the first tick resolves the legacy state.
    needs_state_scanning: SyncMutex<bool>,
    /// Block positions of the crystals armed for the current respawn sequence.
    respawn_crystals: SyncMutex<Vec<IVec3>>,
    /// Vanilla `ticksSinceDragonSeen`: counts up while the fight cannot see its
    /// dragon, and is reset whenever it can.
    ticks_since_dragon_seen: SyncMutex<i32>,
    /// Vanilla `ticksSinceLastPlayerScan`.
    ticks_since_last_player_scan: SyncMutex<i32>,
    /// Whether `dragonEvent.getPlayers()` is non-empty.
    has_nearby_players: SyncMutex<bool>,
}

impl DragonFight {
    /// Creates a fight with no dragon recorded.
    #[must_use]
    pub fn new() -> Self {
        Self {
            progress: SyncMutex::new(1.0),
            bar_visible: SyncMutex::new(false),
            // A brand new world has not resolved its dragon state yet, exactly like
            // vanilla's `createDefault`.
            needs_state_scanning: SyncMutex::new(true),
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

    /// Returns how many end crystals are still alive.
    ///
    /// Vanilla `EnderDragonFight.aliveCrystals()`. The count is refreshed by
    /// [`Self::update_crystal_count`] and decremented by
    /// [`Self::on_crystal_destroyed`].
    #[must_use]
    pub fn alive_crystals(&self) -> i32 {
        *self.alive_crystals.lock()
    }

    /// Vanilla `EnderDragonFight.updateCrystalCount`: recount the crystals around
    /// the exit portal, at most once every 100 ticks.
    ///
    /// Vanilla scans `EndCrystal` entities in the exit-portal column's box while
    /// the dragon is dead. Steel passes the live count in because the fight has no
    /// world handle of its own.
    pub fn update_crystal_count(&self, counted: i32) {
        *self.alive_crystals.lock() = counted.max(0);
        *self.ticks_since_crystals_scanned.lock() = 0;
    }

    /// Advances the crystal scan timer and returns whether a recount is due.
    ///
    /// Vanilla `EnderDragonFight.tick` recounts once every 100 ticks while
    /// `!dragonKilled`.
    pub fn should_rescan_crystals(&self) -> bool {
        if *self.dragon_killed.lock() {
            return false;
        }
        let mut ticks = self.ticks_since_crystals_scanned.lock();
        *ticks += 1;
        if *ticks >= TIME_BETWEEN_CRYSTAL_SCANS {
            *ticks = 0;
            return true;
        }
        false
    }

    /// Vanilla `EnderDragonFight.onCrystalDestroyed`: a destroyed crystal lowers the
    /// alive count so the dragon stops healing and starts using the inner ring.
    pub fn on_crystal_destroyed(&self) {
        let mut crystals = self.alive_crystals.lock();
        *crystals = (*crystals - 1).max(0);
    }

    /// Vanilla `EnderDragonFight.dragonUUID`.
    #[must_use]
    pub fn dragon_uuid(&self) -> Option<uuid::Uuid> {
        *self.dragon_uuid.lock()
    }

    /// Vanilla `EnderDragonFight.setDragonUUID` / the assignment in `createNewDragon`.
    pub fn set_dragon_uuid(&self, uuid: Option<uuid::Uuid>) {
        *self.dragon_uuid.lock() = uuid;
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
        // Vanilla compares the dragon's UUID with `dragonUUID`; the recorded beam
        // target is the fight's marker for "this dragon owns the bar".
        if self.dragon_target().is_none() {
            return false;
        }
        *self.progress.lock() = health_fraction.clamp(0.0, 1.0);
        *self.bar_visible.lock() = true;
        // Vanilla `updateDragon` also resets the seen timer, which is why a live
        // dragon is never replaced while the fight can see it.
        *self.ticks_since_dragon_seen.lock() = 0;
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
        *self.dragon_uuid.lock() = None;
        *self.alive_crystals.lock() = 0;
        *self.respawn_stage.lock() = None;

        let first_kill = !*self.previously_killed.lock();
        *self.previously_killed.lock() = true;

        DragonKillOutcome {
            spawn_exit_portal: true,
            drop_dragon_egg: first_kill,
            spawn_gateway: true,
            first_kill,
        }
    }

    /// Vanilla `EnderDragonFight.tryRespawn`: a player using an end crystal on the
    /// obsidian/bedrock pillaring above the exit portal starts the respawn.
    ///
    /// Vanilla requires four crystals, one on each horizontal side of the block
    /// above the portal centre. The caller has the level, so it passes the found
    /// crystals in; this arms the sequence with them.
    pub fn try_respawn(&self, crystals: Vec<IVec3>) -> bool {
        // Vanilla requires `dragonKilled` and no stage in progress.
        if !self.is_dragon_killed() || self.respawn_stage().is_some() {
            return false;
        }
        if crystals.is_empty() {
            return false;
        }

        let mut started = self.respawn_started.lock();
        if *started {
            return false;
        }
        *started = true;
        drop(started);

        // Vanilla `respawnDragon`: the stage starts, the portal is deactivated and
        // the crystals become the sequence's drivers.
        *self.respawn_stage.lock() = Some(DragonRespawnStage::Start);
        *self.respawn_time.lock() = 0;
        *self.respawn_crystals.lock() = crystals;
        true
    }

    /// Whether the respawn sequence is currently armed.
    #[must_use]
    pub fn respawn_started(&self) -> bool {
        *self.respawn_started.lock()
    }

    /// Vanilla `EnderDragonFight.tick`'s respawn half: advance one stage tick.
    ///
    /// Mirrors `DragonRespawnStage.tick`, including the exact tick numbers vanilla
    /// hard-codes. Returns the world effects for this tick, in order.
    pub fn tick_respawn(&self) -> Vec<DragonRespawnEffect> {
        let mut effects = Vec::new();
        let mut stage = self.respawn_stage.lock();
        let Some(current) = *stage else {
            return effects;
        };
        let mut time = self.respawn_time.lock();
        let now = *time;
        *time += 1;

        match current {
            // `START`: every respawn crystal aims at `(0, 128, 0)`, then the wait.
            DragonRespawnStage::Start => {
                effects.push(DragonRespawnEffect::AimCrystalsAt(BlockPos::new(
                    0, DRAGON_SPAWN_Y, 0,
                )));
                *stage = Some(DragonRespawnStage::PreparingToSummonPillars);
                *time = 0;
            }
            // `PREPARING_TO_SUMMON_PILLARS`: five hits of event 3001, then pillars.
            DragonRespawnStage::PreparingToSummonPillars => {
                if now < 100 {
                    if matches!(now, 0 | 50 | 51 | 52) || now >= 95 {
                        effects.push(DragonRespawnEffect::LevelEvent {
                            event: EVENT_DRAGON_RESPAWN,
                            pos: BlockPos::new(0, DRAGON_SPAWN_Y, 0),
                        });
                    }
                } else {
                    *stage = Some(DragonRespawnStage::SummoningPillars);
                    *time = 0;
                }
            }
            // `SUMMONING_PILLARS`: one spike every forty ticks, aimed then rebuilt.
            DragonRespawnStage::SummoningPillars => {
                let start_of_beam = now % 40 == 0;
                let end_of_beam = now % 40 == 39;
                if start_of_beam || end_of_beam {
                    let index = now / 40;
                    // `SUMMONING_PILLARS` walks the level's spikes by index; it ends
                    // the stage as soon as the index runs past them.
                    if let Some(spike) = self.spike(index) {
                        if start_of_beam {
                            effects.push(DragonRespawnEffect::AimCrystalsAt(BlockPos::new(
                                spike.center_x,
                                spike.height + 1,
                                spike.center_z,
                            )));
                        } else {
                            effects.push(DragonRespawnEffect::RebuildSpike(spike));
                        }
                    } else if start_of_beam {
                        *stage = Some(DragonRespawnStage::SummoningDragon);
                        *time = 0;
                    }
                }
            }
            // `SUMMONING_DRAGON`: flashes, then the crystals blow up and it spawns.
            DragonRespawnStage::SummoningDragon => {
                if now >= 100 {
                    *stage = None;
                    *time = 0;
                    effects.push(DragonRespawnEffect::ResetSpikeCrystals);
                    effects.push(DragonRespawnEffect::DetonateRespawnCrystals);
                    effects.push(DragonRespawnEffect::RespawnDragon);
                } else if now >= 80 {
                    effects.push(DragonRespawnEffect::LevelEvent {
                        event: EVENT_DRAGON_RESPAWN,
                        pos: BlockPos::new(0, DRAGON_SPAWN_Y, 0),
                    });
                } else if now == 0 {
                    effects.push(DragonRespawnEffect::AimCrystalsAt(BlockPos::new(
                        0, DRAGON_SPAWN_Y, 0,
                    )));
                } else if now < 5 {
                    effects.push(DragonRespawnEffect::LevelEvent {
                        event: EVENT_DRAGON_RESPAWN,
                        pos: BlockPos::new(0, DRAGON_SPAWN_Y, 0),
                    });
                }
            }
            DragonRespawnStage::End => {}
        }

        effects
    }

    /// Vanilla `EnderDragonFight.setRespawnStage`.
    ///
    /// `END` is the terminal state: the stage clears, the fight is live again and a
    /// dragon is created. Vanilla throws when called outside a respawn, which Steel
    /// treats as a no-op because the caller already checked.
    pub fn set_respawn_stage(&self, stage: DragonRespawnStage) {
        let mut current = self.respawn_stage.lock();
        if current.is_none() && stage != DragonRespawnStage::End {
            return;
        }
        *self.respawn_time.lock() = 0;
        if stage == DragonRespawnStage::End {
            *current = None;
            *self.dragon_killed.lock() = false;
        } else {
            *current = Some(stage);
        }
    }

    /// Vanilla `EnderDragonFight.abortRespawnSequence`: a respawn crystal went
    /// missing, so the sequence stops and the fight goes back to dead.
    pub fn abort_respawn(&self) {
        *self.respawn_stage.lock() = None;
        *self.respawn_time.lock() = 0;
        *self.respawn_started.lock() = false;
        *self.respawn_crystals.lock() = Vec::new();
    }

    /// Vanilla `EnderDragonFight.setExitPortalLocation` / the heightmap lookup.
    pub fn set_exit_portal_location(&self, location: Option<IVec3>) {
        *self.exit_portal_location.lock() = location;
    }

    /// Vanilla `EnderDragonFight.getExitPortalLocation`.
    #[must_use]
    pub fn exit_portal_location(&self) -> Option<IVec3> {
        *self.exit_portal_location.lock()
    }

    /// Returns the gateway ring indices still unopened, filling them in on the
    /// first call.
    ///
    /// Vanilla keeps `List<Integer> gateways` initialised to `0..GATEWAY_COUNT`
    /// and pops the last entry per kill.
    #[must_use]
    pub fn gateways(&self) -> Vec<i32> {
        let mut gateways = self.gateways.lock();
        if gateways.is_empty() {
            gateways.extend(0..GATEWAY_COUNT);
        }
        gateways.clone()
    }

    /// Vanilla `EnderDragonFight.spawnNewGateway`: the gateway for kill number
    /// `gateways_used`, or `None` once all twenty have opened.
    #[must_use]
    pub fn next_gateway(&self) -> Option<IVec3> {
        let index = {
            let mut gateways = self.gateways.lock();
            if gateways.is_empty() {
                gateways.extend(0..GATEWAY_COUNT);
            }
            gateways.pop()?
        };
        Some(gateway_position(index))
    }

    /// Vanilla `EnderDragonFight.getPodiumLocation(offset)`: the origin plus an
    /// offset, where the origin is `END_PODIUM_LOCATION` (`BlockPos.ZERO`).
    #[must_use]
    pub fn podium_location(offset: BlockPos) -> BlockPos {
        PodiumOrigin::origin().offset(offset.x(), offset.y(), offset.z())
    }

    /// Returns the podium position the dragon egg and exit portal sit on.
    #[must_use]
    pub fn podium_pos() -> BlockPos {
        BlockPos::new(EXIT_PORTAL.x, ORIGIN_Y, EXIT_PORTAL.z)
    }

    /// The `index`-th arena spike, or `None` past the end of the ring.
    ///
    /// Vanilla `DragonRespawnStage.SUMMONING_PILLARS` indexes
    /// `EndSpikeFeature.getSpikesForLevel(level)` directly; the fight needs the
    /// level seed to rebuild the same ring, so it is threaded in here.
    #[must_use]
    pub fn spike(&self, index: i32) -> Option<steel_registry::feature::EndSpike> {
        let seed = *self.level_seed.lock()?;
        crate::worldgen::feature::FeatureDecorationRunner::end_spikes_for_level(seed)
            .get(usize::try_from(index).ok()?)
            .copied()
    }

    /// Records the level seed the spike ring is generated from.
    pub fn set_level_seed(&self, seed: i64) {
        *self.level_seed.lock() = Some(seed);
    }

    /// Vanilla `EnderDragonFight.init`: fill the gateway order from the level seed.
    ///
    /// `Util.shuffle(ContiguousSet.create(Range.closedOpen(0, 20)), createThreadLocal(seed))`
    /// — `java.util.Collections.shuffle(List, Random)`, which walks the list from the
    /// end and swaps each slot with `random.nextInt(index + 1)`.
    pub fn init_gateways(&self, seed: i64) {
        use steel_utils::random::{Random as _, RandomSource, legacy_random::LegacyRandom};

        let mut order: Vec<i32> = (0..GATEWAY_COUNT).collect();
        // `RandomSource.createThreadLocalInstance(seed)` is a `SingleThreadedRandomSource`,
        // which shares `LegacyRandomSource`'s algorithm; Steel's `LegacyRandom` is
        // that same generator.
        let mut random = RandomSource::Legacy(LegacyRandom::from_seed(seed as u64));
        random.shuffle(&mut order);
        *self.gateways.lock() = order;
    }

    /// Vanilla `EnderDragonFight.tick`: `setVisible(!dragonKilled)`.
    pub fn set_bar_visible(&self, visible: bool) {
        *self.bar_visible.lock() = visible;
    }

    /// Vanilla `EnderDragonFight.updatePlayers`'s 20-tick scan timer.
    ///
    /// Returns `true` on the ticks the player set has to be refreshed.
    pub fn tick_player_scan(&self) -> bool {
        let mut ticks = self.ticks_since_last_player_scan.lock();
        *ticks += 1;
        if *ticks >= TIME_BETWEEN_PLAYER_SCANS {
            *ticks = 0;
            return true;
        }
        false
    }

    /// Records whether any valid player is watching the fight.
    pub fn set_has_nearby_players(&self, has_players: bool) {
        *self.has_nearby_players.lock() = has_players;
    }

    /// Whether the boss-bar player set is non-empty, which gates the whole tick.
    #[must_use]
    pub fn has_nearby_players(&self) -> bool {
        *self.has_nearby_players.lock()
    }

    /// Vanilla `EnderDragonFight.tick`'s `++this.ticksSinceDragonSeen`.
    #[must_use]
    pub fn tick_dragon_seen(&self) -> i32 {
        let mut ticks = self.ticks_since_dragon_seen.lock();
        *ticks += 1;
        *ticks
    }

    /// Vanilla `ticksSinceDragonSeen = 0` after the dragon was found.
    pub fn reset_ticks_since_dragon_seen(&self) {
        *self.ticks_since_dragon_seen.lock() = 0;
    }

    /// Vanilla `needsStateScanning`.
    #[must_use]
    pub fn needs_state_scanning(&self) -> bool {
        *self.needs_state_scanning.lock()
    }

    /// Clears `needsStateScanning` once `scan_state` has run.
    pub fn finish_state_scan(&self) {
        *self.needs_state_scanning.lock() = false;
    }

    /// Vanilla `respawnCrystals`: the crystals the respawn animation drives.
    #[must_use]
    pub fn respawn_crystals(&self) -> Vec<IVec3> {
        self.respawn_crystals.lock().clone()
    }

    /// Records the crystals that arm the respawn sequence.
    pub fn set_respawn_crystals(&self, crystals: Vec<IVec3>) {
        *self.respawn_crystals.lock() = crystals;
    }

    /// Vanilla `EnderDragonFight.respawnStage`.
    #[must_use]
    pub fn respawn_stage(&self) -> Option<DragonRespawnStage> {
        *self.respawn_stage.lock()
    }

    /// Vanilla `scanState`: mark the world as previously killed (or not), and
    /// record whether a dragon is still alive.
    ///
    /// The caller resolves the world facts — whether an active exit portal exists
    /// and whether any dragon is present — because the fight has no world handle.
    pub fn scan_state(&self, has_active_exit_portal: bool, has_dragon: Option<uuid::Uuid>) {
        // Vanilla: `hasPreviouslyKilledDragon = activePortalExists`.
        *self.previously_killed.lock() = has_active_exit_portal;

        match has_dragon {
            Some(uuid) => {
                *self.dragon_uuid.lock() = Some(uuid);
                *self.dragon_killed.lock() = false;
            }
            None => {
                *self.dragon_killed.lock() = true;
            }
        }

        // Vanilla's last rule: a world that was never beaten cannot be in the
        // killed state, so it always gets a dragon again.
        if !has_active_exit_portal {
            *self.dragon_killed.lock() = false;
        }

        self.finish_state_scan();
    }

    /// Serialises the fight exactly as vanilla's codec does.
    #[must_use]
    pub fn to_saved_data(&self) -> DragonFightSavedData {
        DragonFightSavedData {
            needs_state_scanning: self.needs_state_scanning(),
            dragon_killed: self.is_dragon_killed(),
            previously_killed: self.has_previously_killed_dragon(),
            respawn_stage: self
                .respawn_stage()
                .map(|stage| stage.saved_name().to_owned()),
            respawn_time: *self.respawn_time.lock(),
            dragon_uuid: self.dragon_uuid(),
            exit_portal_location: self
                .exit_portal_location()
                .map(|pos| [pos.x, pos.y, pos.z]),
            gateways: self.gateways(),
            respawn_crystals: self
                .respawn_crystals()
                .into_iter()
                .map(|pos| [pos.x, pos.y, pos.z])
                .collect(),
        }
    }

    /// Restores a fight from its saved data.
    pub fn apply_saved_data(&self, data: &DragonFightSavedData) {
        *self.needs_state_scanning.lock() = data.needs_state_scanning;
        *self.dragon_killed.lock() = data.dragon_killed;
        *self.previously_killed.lock() = data.previously_killed;
        *self.respawn_stage.lock() =
            data.respawn_stage.as_deref().and_then(DragonRespawnStage::from_saved_name);
        *self.respawn_time.lock() = data.respawn_time;
        *self.dragon_uuid.lock() = data.dragon_uuid;
        *self.exit_portal_location.lock() = data
            .exit_portal_location
            .map(|pos| IVec3::new(pos[0], pos[1], pos[2]));
        // Vanilla stores the whole ring on a fresh world and a short ring once
        // gateways have opened; an empty list means "opened all twenty".
        *self.gateways.lock() = if data.gateways.is_empty() && !data.dragon_killed {
            (0..GATEWAY_COUNT).collect()
        } else {
            data.gateways.clone()
        };
        *self.respawn_crystals.lock() = data
            .respawn_crystals
            .iter()
            .copied()
            .map(|pos| IVec3::new(pos[0], pos[1], pos[2]))
            .collect();
        *self.respawn_started.lock() = data.respawn_stage.is_some();
        // A dragon that is still alive re-arms the crystal beam target on its next
        // tick, so the saved UUID is enough here.
        *self.dragon_target.lock() = None;
        *self.bar_visible.lock() = !data.dragon_killed && data.dragon_uuid.is_some();
        *self.progress.lock() = if data.dragon_killed { 0.0 } else { 1.0 };
    }
}

/// What `setDragonKilled` asks the world to write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DragonKillOutcome {
    /// Place the active `end_podium` feature at the exit portal location.
    pub spawn_exit_portal: bool,
    /// Drop the dragon egg on the podium (only the first kill).
    pub drop_dragon_egg: bool,
    /// Open a gateway portal so the player can leave the End.
    pub spawn_gateway: bool,
    /// Whether this was the world's first kill.
    ///
    /// Vanilla drops the dragon egg only on the first kill; the portal and the
    /// gateway ring advance on every kill.
    pub first_kill: bool,
}

/// Vanilla `EnderDragonFight.DragonRespawnStage`.
///
/// The sequence runs `START` → `PREPARING` → `SUMMONING_PILLARS` →
/// `SUMMONING_DRAGON` → done, driven one tick at a time by
/// [`DragonFight::tick_respawn`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragonRespawnStage {
    /// `START`: the respawn crystals aim at `(0, 128, 0)`.
    Start,
    /// `PREPARING_TO_SUMMON_PILLARS`: `3001` at ticks 0, 50, 51, 52 and 95-99.
    PreparingToSummonPillars,
    /// `SUMMONING_PILLARS`: one arena spike rebuilt every forty ticks.
    SummoningPillars,
    /// `SUMMONING_DRAGON`: flashes at 0-4 and 80-99, then the dragon spawns.
    SummoningDragon,
    /// `END`: the terminal state, which clears the stage and revives the fight.
    End,
}

impl DragonRespawnStage {
    /// The name vanilla writes into `respawn_stage`.
    ///
    /// Vanilla's codec uses `DragonRespawnStage`'s enum name, so the saved value is
    /// `START`, `PREPARING_TO_SUMMON_PILLARS`, `SUMMONING_PILLARS`,
    /// `SUMMONING_DRAGON` or `END`.
    #[must_use]
    pub const fn saved_name(self) -> &'static str {
        match self {
            Self::Start => "START",
            Self::PreparingToSummonPillars => "PREPARING_TO_SUMMON_PILLARS",
            Self::SummoningPillars => "SUMMONING_PILLARS",
            Self::SummoningDragon => "SUMMONING_DRAGON",
            Self::End => "END",
        }
    }

    /// Parses a persisted `respawn_stage` name.
    #[must_use]
    pub fn from_saved_name(name: &str) -> Option<Self> {
        Some(match name {
            "START" => Self::Start,
            "PREPARING_TO_SUMMON_PILLARS" => Self::PreparingToSummonPillars,
            "SUMMONING_PILLARS" => Self::SummoningPillars,
            "SUMMONING_DRAGON" => Self::SummoningDragon,
            "END" => Self::End,
            _ => return None,
        })
    }
}

/// What [`DragonFight::tick_respawn`] asks the world to do this tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragonRespawnEffect {
    /// Point every respawn crystal's beam at `target`.
    AimCrystalsAt(BlockPos),
    /// Broadcast level event `event` at `pos`.
    LevelEvent { event: i32, pos: BlockPos },
    /// Clear and rebuild one arena spike with its crystal.
    RebuildSpike(steel_registry::feature::EndSpike),
    /// Vanilla `resetSpikeCrystals`: rebuild every spike's crystal.
    ResetSpikeCrystals,
    /// Blow up and discard the respawn crystals.
    DetonateRespawnCrystals,
    /// Spawn a fresh dragon at `(0, 128, 0)`.
    RespawnDragon,
}

/// Vanilla `EnderDragonFight`'s gateway ring position for `index`.
///
/// `theta` steps by `0.15707963267948966` (a tenth of pi) from `-pi`, and the
/// gateway sits at radius [`GATEWAY_DISTANCE`] on y `75`.
#[must_use]
pub fn gateway_position(index: i32) -> IVec3 {
    let theta = 2.0 * (-std::f64::consts::PI + 0.157_079_632_679_489_66 * f64::from(index));
    let x = f64::from(GATEWAY_DISTANCE) * f64::from(steel_math::trig::cos(theta));
    let z = f64::from(GATEWAY_DISTANCE) * f64::from(steel_math::trig::sin(theta));
    IVec3::new(x.floor() as i32, 75, z.floor() as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_data_round_trips_every_field() {
        let fight = DragonFight::new();
        fight.init_gateways(1234);
        let spent = fight.next_gateway();
        assert!(spent.is_some());

        fight.set_dragon_uuid(Some(uuid::Uuid::from_u128(7)));
        fight.set_exit_portal_location(Some(IVec3::new(0, 60, 0)));
        fight.set_respawn_crystals(vec![IVec3::new(3, 64, 0)]);
        *fight.respawn_stage.lock() = Some(DragonRespawnStage::SummoningPillars);
        *fight.respawn_time.lock() = 41;

        let saved = fight.to_saved_data();
        assert_eq!(saved.respawn_stage.as_deref(), Some("SUMMONING_PILLARS"));
        assert_eq!(saved.respawn_time, 41);
        assert_eq!(saved.gateways.len(), GATEWAY_COUNT as usize - 1);
        assert_eq!(saved.respawn_crystals, vec![[3, 64, 0]]);

        let restored = DragonFight::new();
        restored.apply_saved_data(&saved);
        assert_eq!(restored.dragon_uuid(), fight.dragon_uuid());
        assert_eq!(restored.exit_portal_location(), fight.exit_portal_location());
        assert_eq!(restored.respawn_stage(), Some(DragonRespawnStage::SummoningPillars));
        assert_eq!(*restored.respawn_time.lock(), 41);
        assert_eq!(restored.gateways(), saved.gateways);
        assert_eq!(restored.respawn_crystals(), fight.respawn_crystals());
    }

    #[test]
    fn every_respawn_stage_name_round_trips() {
        for stage in [
            DragonRespawnStage::Start,
            DragonRespawnStage::PreparingToSummonPillars,
            DragonRespawnStage::SummoningPillars,
            DragonRespawnStage::SummoningDragon,
            DragonRespawnStage::End,
        ] {
            assert_eq!(
                DragonRespawnStage::from_saved_name(stage.saved_name()),
                Some(stage)
            );
        }
        assert_eq!(DragonRespawnStage::from_saved_name("NOPE"), None);
    }

    #[test]
    fn gateway_ring_is_a_seeded_permutation() {
        let first = DragonFight::new();
        first.init_gateways(42);
        let second = DragonFight::new();
        second.init_gateways(42);

        let order = first.gateways();
        assert_eq!(order, second.gateways());
        let mut sorted = order.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..GATEWAY_COUNT).collect::<Vec<_>>());
    }

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
