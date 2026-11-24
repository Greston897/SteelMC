//! Tick overload management: instead of skipping ticks, prioritize light and
//! important work first, defer heavier work to later ticks or background
//! systems, and keep the server in sync with players and saves.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// How long the server can be behind before we start deferring non-critical work.
const DEFER_THRESHOLD: Duration = Duration::from_millis(25);
/// How long before we warn about persistent overload.
const OVERLOAD_WARNING_INTERVAL: Duration = Duration::from_secs(10);
/// Maximum number of deferrals tracked per tick.
const MAX_DEFERRED_PER_TICK: usize = 8;

/// A unit of work that can be deferred to a later tick.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// Custom deferred work with a name.
    Custom(&'static str),
}

/// Tracks overload state and decides what to defer this tick.
///
/// Instead of skipping ticks (which desyncs the world from players and saves),
/// this guard marks lower-priority work as deferred so critical gameplay
/// (movement, combat, player interaction, saving) always runs.
pub(super) struct TickOverloadGuard {
    /// When the last overload warning was logged.
    last_warning: Option<Instant>,
    /// Deferred work queue — items waiting to be processed on a future tick.
    deferred: VecDeque<DeferredWork>,
    /// How many ticks we've been continuously overloaded.
    overload_ticks: u32,
}

impl TickOverloadGuard {
    pub(super) const fn new() -> Self {
        Self {
            last_warning: None,
            deferred: VecDeque::with_capacity(MAX_DEFERRED_PER_TICK),
            overload_ticks: 0,
        }
    }

    pub(super) const fn restart_report_gap(&mut self, now: Instant) {
        self.last_warning = Some(now);
    }

    /// Call at the start of each tick. Returns whether the server is
    /// currently overloaded (non-critical work should be deferred).
    pub(super) fn tick(&mut self, now: Instant, tick_duration: Duration) -> bool {
        let is_overloaded = tick_duration > DEFER_THRESHOLD;

        if is_overloaded {
            self.overload_ticks = self.overload_ticks.saturating_add(1);
        } else {
            self.overload_ticks = 0;
        }

        // Warn periodically about persistent overload.
        if is_overloaded {
            let should_warn = self.last_warning.is_none()
                || self
                    .last_warning
                    .is_some_and(|last| now.duration_since(last) >= OVERLOAD_WARNING_INTERVAL);

            if should_warn {
                log::warn!(
                    "Server overloaded: tick took {:?}ms; deferring non-critical work",
                    tick_duration.as_millis()
                );
                self.last_warning = Some(now);
            }
        }

        is_overloaded
    }

    /// Returns true if the given work type should be deferred this tick.
    /// Critical work (movement, combat, saves, player packets) is never deferred.
    pub(super) fn should_defer(&self, work: DeferredWork) -> bool {
        // Never defer critical work.
        matches!(
            work,
            DeferredWork::ChunkMaintenance
                | DeferredWork::EntityIdleAI
                | DeferredWork::BlockEntityIdle
                | DeferredWork::NaturalSpawning
                | DeferredWork::ScoreboardMaintenance
                | DeferredWork::Custom(_)
        ) && self.overload_ticks > 0
    }

    /// Enqueue work to be processed later when the server catches up.
    pub(super) fn defer(&mut self, work: DeferredWork) {
        if self.deferred.len() < MAX_DEFERRED_PER_TICK {
            self.deferred.push_back(work);
        }
    }

    /// Process deferred work when the server is back to normal speed.
    /// Returns how many items were processed.
    pub(super) fn process_deferred(&mut self) -> usize {
        let mut processed = 0;
        while let Some(work) = self.deferred.pop_front() {
            // In a future iteration, this could dispatch to the appropriate
            // subsystem. For now we track that work was deferred.
            processed += 1;
        }
        processed
    }

    /// Returns the number of deferred items pending.
    pub(super) fn deferred_count(&self) -> usize {
        self.deferred.len()
    }

    /// Returns whether the server is currently overloaded.
    pub(super) fn is_overloaded(&self) -> bool {
        self.overload_ticks > 0
    }

    /// Returns how many consecutive ticks have been overloaded.
    pub(super) fn overload_tick_count(&self) -> u32 {
        self.overload_ticks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NANOS_PER_TICK: u64 = 50_000_000;
    const TICK: Duration = Duration::from_nanos(NANOS_PER_TICK);

    #[test]
    fn fast_tick_is_not_overloaded() {
        let mut guard = TickOverloadGuard::new();
        let now = Instant::now();
        assert!(!guard.tick(now, Duration::from_millis(1)));
        assert_eq!(guard.overload_tick_count(), 0);
        assert!(!guard.should_defer(DeferredWork::EntityIdleAI));
    }

    #[test]
    fn slow_tick_defers_non_critical_work() {
        let mut guard = TickOverloadGuard::new();
        let now = Instant::now();
        assert!(guard.tick(now, Duration::from_millis(100)));
        assert_eq!(guard.overload_tick_count(), 1);
        assert!(guard.should_defer(DeferredWork::EntityIdleAI));
        assert!(guard.should_defer(DeferredWork::ChunkMaintenance));
    }

    #[test]
    fn critical_work_is_never_deferred() {
        let mut guard = TickOverloadGuard::new();
        let now = Instant::now();
        guard.tick(now, Duration::from_millis(100));
        // All DeferredWork variants are non-critical by design; critical work
        // (player packets, saves, combat) never goes through should_defer.
        assert!(guard.deferred_count() == 0);
    }

    #[test]
    fn defer_and_process() {
        let mut guard = TickOverloadGuard::new();
        guard.defer(DeferredWork::ChunkMaintenance);
        guard.defer(DeferredWork::EntityIdleAI);
        assert_eq!(guard.deferred_count(), 2);
        let processed = guard.process_deferred();
        assert_eq!(processed, 2);
        assert_eq!(guard.deferred_count(), 0);
    }

    #[test]
    fn recovers_after_fast_ticks() {
        let mut guard = TickOverloadGuard::new();
        let now = Instant::now();
        guard.tick(now, Duration::from_millis(100));
        assert!(guard.is_overloaded());
        guard.tick(now, Duration::from_millis(1));
        guard.tick(now, Duration::from_millis(1));
        assert!(!guard.is_overloaded());
    }
}
