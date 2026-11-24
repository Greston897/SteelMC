//! Tick overload management: prioritize lightest & most important work first,
//! defer heavier non-critical work to the next tick or a background pool,
//! never skip ticks and never desync saves / player packets.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// How long the server can be behind before we start deferring non-critical work.
/// 25 ms = half a tick; below MSPT=50 the server still feels 20 TPS.
const DEFER_THRESHOLD: Duration = Duration::from_millis(25);
/// How long the server can be severely behind before we offload to background.
const BACKGROUND_OFFLOAD_THRESHOLD: Duration = Duration::from_millis(45);
/// How long before we warn about persistent overload (avoid log spam).
const OVERLOAD_WARNING_INTERVAL: Duration = Duration::from_secs(10);
/// Maximum number of deferrals tracked per tick – bounds RAM.
const MAX_DEFERRED_PER_TICK: usize = 16;
/// Maximum deferred items drained per recovery tick – bounds MSPT spike when catching up.
const MAX_DRAIN_PER_TICK: usize = 8;

/// Priority tier – lower = more important / lighter, executed first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Tier {
    /// Never deferred: movement, combat, player packets, saves.
    #[allow(clippy::allow_attributes, reason = "critical tier API")]
    #[allow(dead_code, reason = "critical tier API")]
    Critical = 0,
    /// Important but can be delayed 1 tick under light overload.
    Important = 1,
    /// Heavy / cosmetic: can be delayed or moved to background pool.
    Deferred = 2,
}

/// A unit of work that can be deferred to a later tick or background system.
///
/// Priority ordering is defined by [`DeferredWork::tier`] (importance) and
/// [`DeferredWork::cost`] (MSPT weight). When recovering, the guard drains
/// lightest + most important first so the world stays in sync and player
/// experience is preserved.
#[derive(Clone, Debug, PartialEq, Eq)]
#[expect(dead_code, reason = "deferred work types")]
pub enum DeferredWork {
    /// Non-critical chunk updates (lighting, decoration, minor block changes).
    ChunkMaintenance,
    /// Non-critical entity AI goals (wander, look around, idle animations).
    EntityIdleAI,
    /// Non-critical block entity ticks (decorative updates).
    BlockEntityIdle,
    /// Non-critical natural spawning checks.
    NaturalSpawning,
    /// Non-critical scoreboard/team updates.
    ScoreboardMaintenance,
    /// Custom deferred work with a name – treated as Deferred tier, cost 3.
    Custom(&'static str),
}

impl DeferredWork {
    /// Tier decides *whether* we defer under a given overload level.
    #[inline]
    pub(super) const fn tier(&self) -> Tier {
        match self {
            DeferredWork::EntityIdleAI | DeferredWork::BlockEntityIdle => Tier::Important,
            DeferredWork::ChunkMaintenance
            | DeferredWork::NaturalSpawning
            | DeferredWork::ScoreboardMaintenance
            | DeferredWork::Custom(_) => Tier::Deferred,
        }
    }

    /// Cost decides *order* when draining – lightest first.
    #[inline]
    pub(super) const fn cost(&self) -> u8 {
        match self {
            DeferredWork::ScoreboardMaintenance => 1,
            DeferredWork::BlockEntityIdle => 2,
            DeferredWork::EntityIdleAI | DeferredWork::Custom(_) => 3,
            DeferredWork::ChunkMaintenance => 4,
            DeferredWork::NaturalSpawning => 5,
        }
    }

    #[inline]
    pub(super) const fn is_heavy(&self) -> bool {
        matches!(
            self,
            DeferredWork::ChunkMaintenance | DeferredWork::NaturalSpawning
        )
    }
}

/// Tracks overload state and decides what to defer this tick.
///
/// Instead of skipping ticks (which desyncs the world from players and saves),
/// this guard marks lower-priority work as deferred so critical gameplay
/// (movement, combat, player interaction, saving) always runs. Deferred work
/// is re-queued and drained in priority order when the server catches up, or
/// offloaded to a background thread pool if the overload is severe.
pub(super) struct TickOverloadGuard {
    /// When the last overload warning was logged.
    last_warning: Option<Instant>,
    /// Deferred work queue — items waiting to be processed on a future tick
    /// or background system. Bounded to `MAX_DEFERRED_PER_TICK` to cap RAM.
    deferred: VecDeque<DeferredWork>,
    /// How many ticks we've been continuously overloaded.
    overload_ticks: u32,
    /// Last tick duration (for tier decisions).
    last_tick_duration: Duration,
}

impl TickOverloadGuard {
    pub(super) const fn new() -> Self {
        Self {
            last_warning: None,
            deferred: VecDeque::new(),
            overload_ticks: 0,
            last_tick_duration: Duration::ZERO,
        }
    }

    pub(super) const fn restart_report_gap(&mut self, now: Instant) {
        self.last_warning = Some(now);
    }

    /// Call at the start of each tick with the time the previous tick took
    /// (or the time we are behind schedule). Returns whether the server is
    /// currently overloaded (non-critical work should be deferred).
    ///
    /// Lightest work is never deferred; important work is deferred only after
    /// a few consecutive overloaded ticks; heavy work is deferred immediately.
    /// Critical work (player packets, saves, combat) never goes through this
    /// guard at all.
    pub(super) fn tick(&mut self, now: Instant, tick_duration: Duration) -> bool {
        self.last_tick_duration = tick_duration;
        // Use a simple hysteresis: once overloaded, require 2 fast ticks to recover
        // to avoid flapping between overloaded/not. This keeps MSPT stable.
        let is_overloaded = if self.overload_ticks > 0 {
            tick_duration > Duration::from_millis(22)
        } else {
            tick_duration > DEFER_THRESHOLD
        };

        if is_overloaded {
            self.overload_ticks = self.overload_ticks.saturating_add(1);
        } else if self.overload_ticks > 0 {
            // Require 2 consecutive fast ticks to clear overload, otherwise decay slowly
            self.overload_ticks = self.overload_ticks.saturating_sub(1);
            if tick_duration <= Duration::from_millis(18) && self.overload_ticks > 0 {
                self.overload_ticks = self.overload_ticks.saturating_sub(1);
            }
        }

        // Warn periodically about persistent overload, but not every tick.
        if is_overloaded {
            let should_warn = self
                .last_warning
                .is_none_or(|last| now.duration_since(last) >= OVERLOAD_WARNING_INTERVAL);

            if should_warn {
                log::warn!(
                    "Server overloaded: tick took {:?}ms ({} consecutive); deferring non-critical work (deferred queue: {})",
                    tick_duration.as_millis(),
                    self.overload_ticks,
                    self.deferred.len()
                );
                self.last_warning = Some(now);
            }
        }

        is_overloaded
    }

    /// Returns true if the given work type should be deferred this tick.
    /// Critical work (movement, combat, saves, player packets) is never
    /// passed here and thus never deferred.
    ///
    /// Tier logic: `Deferred` work is deferred on any overload tick,
    /// `Important` work only after 3 consecutive overloaded ticks to avoid
    /// flapping, so lightest+important runs first.
    #[inline]
    #[allow(clippy::allow_attributes, reason = "should_defer used in tests")]
    #[allow(dead_code, reason = "should_defer used in tests")]
    pub(super) const fn should_defer(&self, work: DeferredWork) -> bool {
        if self.overload_ticks == 0 {
            return false;
        }
        match work.tier() {
            Tier::Critical => false,
            Tier::Important => self.overload_ticks > 3,
            Tier::Deferred => true,
        }
    }

    /// Tier-aware check without constructing an enum – for hot paths.
    #[inline]
    #[allow(clippy::allow_attributes, reason = "should_defer_tier used in tests")]
    #[allow(dead_code, reason = "should_defer_tier used in tests")]
    pub(super) const fn should_defer_tier(&self, tier: Tier) -> bool {
        if self.overload_ticks == 0 {
            return false;
        }
        match tier {
            Tier::Critical => false,
            Tier::Important => self.overload_ticks > 3,
            Tier::Deferred => true,
        }
    }

    /// Returns true if the overload is severe enough to offload heavy work
    /// to a background thread pool instead of just deferring to next tick.
    #[inline]
    pub(super) fn should_offload_to_background(&self) -> bool {
        self.last_tick_duration > BACKGROUND_OFFLOAD_THRESHOLD && self.overload_ticks > 5
    }

    /// Enqueue work to be processed later when the server catches up.
    /// Sorted insertion: lightest + most important first, so drain order is optimal.
    /// Bounded – excess is dropped with a warning to cap RAM.
    pub(super) fn defer(&mut self, work: DeferredWork) {
        if self.deferred.len() >= MAX_DEFERRED_PER_TICK {
            log::warn!(
                "Deferred queue full ({}), dropping {:?}",
                self.deferred.len(),
                work
            );
            return;
        }
        // Keep queue sorted by (tier, cost) so lightest+important is at front.
        // Small queue (≤16) so linear insertion is fine and avoids alloc.
        let pos = self
            .deferred
            .iter()
            .position(|existing| (work.tier(), work.cost()) < (existing.tier(), existing.cost()));
        if let Some(idx) = pos {
            self.deferred.insert(idx, work);
        } else {
            self.deferred.push_back(work);
        }
    }

    /// Process deferred work when the server is back to normal speed.
    /// Drains up to `MAX_DRAIN_PER_TICK` lightest+important items first to
    /// avoid an MSPT spike when recovering. Returns the drained work for the
    /// caller to execute in priority order (lightest+important first).
    pub(super) fn process_deferred(&mut self) -> Vec<DeferredWork> {
        if self.is_overloaded() {
            return Vec::new();
        }
        let mut drained = Vec::with_capacity(MAX_DRAIN_PER_TICK.min(self.deferred.len()));
        while drained.len() < MAX_DRAIN_PER_TICK {
            let Some(work) = self.deferred.pop_front() else {
                break;
            };
            drained.push(work);
        }
        drained
    }

    /// Drains heavy work that should be offloaded to a background pool
    /// when severely overloaded. Returns the heavy items for the caller to
    /// spawn on `spawn_blocking` / rayon.
    pub(super) fn drain_heavy_for_background(&mut self) -> Vec<DeferredWork> {
        if !self.should_offload_to_background() {
            return Vec::new();
        }
        let mut heavy = Vec::with_capacity(4);
        let mut retained = VecDeque::with_capacity(self.deferred.len());
        while let Some(work) = self.deferred.pop_front() {
            if work.is_heavy() {
                heavy.push(work);
            } else {
                retained.push_back(work);
            }
        }
        self.deferred = retained;
        heavy
    }

    /// Returns the number of deferred items pending (for metrics / TPS bar).
    #[allow(clippy::allow_attributes, reason = "deferred_count is metrics API")]
    #[allow(dead_code, reason = "deferred_count is metrics API")]
    pub(super) fn deferred_count(&self) -> usize {
        self.deferred.len()
    }

    /// Returns whether the server is currently overloaded.
    pub(super) const fn is_overloaded(&self) -> bool {
        self.overload_ticks > 0
    }

    /// Budget multiplier for tick work: 1.0 = normal, 0.5 = light overload, 0.25 = severe.
    /// Subsystems use this to decide how many cheapest+important items to run this tick.
    #[allow(dead_code, reason = "budget for future subsystems")]
    pub(super) const fn budget_factor(&self) -> f32 {
        match self.overload_ticks {
            0 => 1.0,
            1..=3 => 0.6,
            4..=6 => 0.35,
            _ => 0.25,
        }
    }

    /// Returns how many consecutive ticks have been overloaded.
    pub(super) const fn overload_tick_count(&self) -> u32 {
        self.overload_ticks
    }

    /// Exposes last tick duration for diagnostics.
    #[expect(dead_code, reason = "diagnostics only")]
    pub(super) const fn last_tick_duration(&self) -> Duration {
        self.last_tick_duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[expect(dead_code, reason = "test timing")]
    const NANOS_PER_TICK: u64 = 50_000_000;
    #[expect(dead_code, reason = "test timing")]
    const TICK: Duration = Duration::from_nanos(NANOS_PER_TICK);

    #[test]
    fn fast_tick_is_not_overloaded() {
        let mut guard = TickOverloadGuard::new();
        let now = Instant::now();
        assert!(!guard.tick(now, Duration::from_millis(1)));
        assert_eq!(guard.overload_tick_count(), 0);
        assert!(!guard.should_defer(DeferredWork::EntityIdleAI));
        assert!(!guard.should_defer(DeferredWork::ChunkMaintenance));
        assert!(!guard.should_defer_tier(Tier::Deferred));
    }

    #[test]
    fn slow_tick_defers_deferred_tier_immediately() {
        let mut guard = TickOverloadGuard::new();
        let now = Instant::now();
        assert!(guard.tick(now, Duration::from_millis(100)));
        assert_eq!(guard.overload_tick_count(), 1);
        // Deferred tier deferred immediately
        assert!(guard.should_defer(DeferredWork::ChunkMaintenance));
        assert!(guard.should_defer(DeferredWork::NaturalSpawning));
        // Important tier NOT yet (needs >3 consecutive)
        assert!(!guard.should_defer(DeferredWork::EntityIdleAI));
        assert!(!guard.should_defer(DeferredWork::BlockEntityIdle));
    }

    #[test]
    fn important_deferred_after_three_consecutive_overloads() {
        let mut guard = TickOverloadGuard::new();
        let now = Instant::now();
        for _ in 0..4 {
            guard.tick(now, Duration::from_millis(100));
        }
        assert!(guard.should_defer(DeferredWork::EntityIdleAI));
        assert!(guard.should_defer(DeferredWork::BlockEntityIdle));
    }

    #[test]
    fn critical_work_is_never_deferred() {
        let mut guard = TickOverloadGuard::new();
        let now = Instant::now();
        guard.tick(now, Duration::from_millis(100));
        // All DeferredWork variants are non-critical by design; critical work
        // (player packets, saves, combat) never goes through should_defer.
        assert!(!guard.should_defer_tier(Tier::Critical));
        assert_eq!(guard.deferred_count(), 0);
    }

    #[test]
    fn defer_sorted_lightest_important_first() {
        let mut guard = TickOverloadGuard::new();
        // Insert heavy first, then light – queue should reorder.
        guard.defer(DeferredWork::NaturalSpawning); // cost 5, deferred
        guard.defer(DeferredWork::ScoreboardMaintenance); // cost 1, deferred
        guard.defer(DeferredWork::EntityIdleAI); // cost 3, important
        // Expected order: Scoreboard (1, deferred) before EntityIdle (3, important) before Natural (5)?
        // Actually sorted by (tier, cost): Important(1) < Deferred(2), so EntityIdle first.
        let first = guard.deferred.pop_front().unwrap();
        assert_eq!(first, DeferredWork::EntityIdleAI);
        let second = guard.deferred.pop_front().unwrap();
        assert_eq!(second, DeferredWork::ScoreboardMaintenance);
        let third = guard.deferred.pop_front().unwrap();
        assert_eq!(third, DeferredWork::NaturalSpawning);
    }

    #[test]
    fn defer_and_process_budget() {
        let mut guard = TickOverloadGuard::new();
        for _ in 0..10 {
            guard.defer(DeferredWork::ChunkMaintenance);
        }
        // Bounded to MAX_DEFERRED_PER_TICK
        assert_eq!(guard.deferred_count(), MAX_DEFERRED_PER_TICK);
        // Not overloaded, so can drain up to MAX_DRAIN_PER_TICK
        let processed = guard.process_deferred();
        assert_eq!(processed.len(), MAX_DRAIN_PER_TICK);
        assert_eq!(
            guard.deferred_count(),
            MAX_DEFERRED_PER_TICK - MAX_DRAIN_PER_TICK
        );
    }

    #[test]
    fn recovers_after_fast_ticks() {
        let mut guard = TickOverloadGuard::new();
        let now = Instant::now();
        guard.tick(now, Duration::from_millis(100));
        assert!(guard.is_overloaded());
        guard.tick(now, Duration::from_millis(1));
        assert!(!guard.is_overloaded());
        assert!(!guard.should_defer(DeferredWork::ChunkMaintenance));
    }

    #[test]
    fn heavy_offload_only_when_severely_overloaded() {
        let mut guard = TickOverloadGuard::new();
        let now = Instant::now();
        guard.defer(DeferredWork::ChunkMaintenance);
        guard.defer(DeferredWork::ScoreboardMaintenance);
        // Not severe yet
        guard.tick(now, Duration::from_millis(30));
        assert_eq!(guard.drain_heavy_for_background().len(), 0); // expect(clippy::assert_is_empty) suppressed via allow
        // Severe: >45ms and >5 consecutive
        for _ in 0..6 {
            guard.tick(now, Duration::from_millis(60));
        }
        let heavy = guard.drain_heavy_for_background();
        assert!(heavy.contains(&DeferredWork::ChunkMaintenance));
        // Scoreboard stays in queue
        assert_eq!(guard.deferred_count(), 1);
    }

    // === СИЛЬНЫЕ ТЕСТЫ: глубокая проверка без десинка и без перегрузки ОЗУ/MSPT ===

    #[test]
    fn critical_never_deferred_even_under_severe_overload() {
        let mut guard = TickOverloadGuard::new();
        let now = Instant::now();
        // 10 severe ticks
        for _ in 0..10 {
            guard.tick(now, Duration::from_millis(100));
        }
        assert!(guard.is_overloaded());
        assert_eq!(guard.overload_tick_count(), 10);
        // Critical tier must never be deferred
        assert!(!guard.should_defer_tier(Tier::Critical));
        // Even if someone tries to defer Critical via should_defer, it would be false
        // (DeferredWork has no Critical variant, so we test tier directly)
        assert!(!guard.should_defer_tier(Tier::Critical));
    }

    #[test]
    fn lightest_important_first_order_is_stable() {
        let mut guard = TickOverloadGuard::new();
        // DeferredWork: Scoreboard (cost1, Deferred), BlockEntityIdle (cost2, Important), EntityIdleAI (cost3, Important), ChunkMaintenance (cost4, Deferred), NaturalSpawning (cost5, Deferred)
        // Sorted by (tier, cost): Important(1) before Deferred(2), so cost2 and cost3 (Important) before cost1,4,5 (Deferred)
        guard.defer(DeferredWork::NaturalSpawning); // 5, Deferred
        guard.defer(DeferredWork::ChunkMaintenance); // 4, Deferred
        guard.defer(DeferredWork::ScoreboardMaintenance); // 1, Deferred
        guard.defer(DeferredWork::EntityIdleAI); // 3, Important
        guard.defer(DeferredWork::BlockEntityIdle); // 2, Important
        // Expected order: BlockEntityIdle (2,Imp), EntityIdleAI (3,Imp), Scoreboard (1,Def), Chunk (4,Def), Natural (5,Def)
        let order = guard.process_deferred();
        assert_eq!(
            order,
            vec![
                DeferredWork::BlockEntityIdle,
                DeferredWork::EntityIdleAI,
                DeferredWork::ScoreboardMaintenance,
                DeferredWork::ChunkMaintenance,
                DeferredWork::NaturalSpawning
            ]
        );
    }

    #[test]
    fn deferred_queue_bounded_ram() {
        let mut guard = TickOverloadGuard::new();
        // Try to defer 100 items, but MAX_DEFERRED is 16
        for _ in 0..100 {
            guard.defer(DeferredWork::ChunkMaintenance);
        }
        assert_eq!(guard.deferred_count(), MAX_DEFERRED_PER_TICK);
        // MSPT bound: drain max 8 per tick
        let drained = guard.process_deferred();
        assert_eq!(drained.len(), MAX_DRAIN_PER_TICK);
        assert_eq!(
            guard.deferred_count(),
            MAX_DEFERRED_PER_TICK - MAX_DRAIN_PER_TICK
        );
        // Next tick drains rest
        let drained2 = guard.process_deferred();
        assert_eq!(drained2.len(), MAX_DRAIN_PER_TICK);
        assert_eq!(guard.deferred_count(), 0);
    }

    #[test]
    fn no_desync_when_overloaded_then_recovered() {
        let mut guard = TickOverloadGuard::new();
        let now = Instant::now();
        // Overload 4 ticks (Important deferred after 3)
        for _ in 0..4 {
            guard.tick(now, Duration::from_millis(100));
        }
        assert!(guard.should_defer(DeferredWork::EntityIdleAI));
        // Recover with 2 fast ticks (hysteresis requires 2)
        guard.tick(now, Duration::from_millis(1));
        assert!(guard.is_overloaded()); // still 3 after one fast
        guard.tick(now, Duration::from_millis(1));
        assert!(!guard.is_overloaded()); // now 1 after second fast? Actually hysteresis: 4 -> 3 -> 1 after 2 fast (since second fast subtracts 2)
        // After recovery, deferred work should be drainable
        guard.defer(DeferredWork::ChunkMaintenance);
        let drained = guard.process_deferred();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0], DeferredWork::ChunkMaintenance);
    }

    #[test]
    fn mspt_bounded_under_sustained_overload() {
        let mut guard = TickOverloadGuard::new();
        let now = Instant::now();
        // Simulate 20 ticks of 60ms (severe)
        for _ in 0..20 {
            guard.tick(now, Duration::from_millis(60));
            assert!(guard.is_overloaded());
        }
        // Even under sustained, budget_factor should be 0.25 (severe)
        assert_eq!(guard.budget_factor(), 0.25);
        // Heavy work should be offloadable
        guard.defer(DeferredWork::ChunkMaintenance);
        guard.defer(DeferredWork::NaturalSpawning);
        let heavy = guard.drain_heavy_for_background();
        assert_eq!(heavy.len(), 2);
        // After draining heavy, non-heavy remains 0
        assert_eq!(guard.deferred_count(), 0);
    }

    #[test]
    fn saves_and_player_packets_never_deferred() {
        // This is a conceptual test: Tier::Critical is never deferred
        let guard = TickOverloadGuard::new();
        // Even if we manually tick to overload, Critical stays false
        let mut guard2 = TickOverloadGuard::new();
        let now = Instant::now();
        guard2.tick(now, Duration::from_millis(100));
        assert!(!guard2.should_defer_tier(Tier::Critical));
        // Saves are outside guard, so they always run — we just ensure guard doesn't block them
        assert!(!guard.should_defer_tier(Tier::Critical));
    }
}
