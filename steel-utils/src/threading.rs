//! Thread-count selection helpers.
//!
//! Every pool size in Steel (the two Tokio runtimes, the Rayon generation and
//! encoding pools, and the gameplay packet workers) is derived from
//! [`available_worker_threads`]. On bare metal that is simply the core count;
//! inside a container it must be the *cgroup CPU allowance* instead.
//!
//! This matters for panel hosting. Pterodactyl, Pelican, Docker Compose and
//! Kubernetes all express a CPU limit as a quota, e.g. Pterodactyl's `--cpus=6`
//! becomes `cpu.max = "600000 100000"` (600% of one core). Rust's
//! [`std::thread::available_parallelism`] only reports the *affinity mask* —
//! and in many container setups the mask is deliberately left wide so the
//! scheduler can burst — so a container limited to 6 cores on a 64-core host
//! would otherwise size its pools for 64 cores, spawn ~100 threads, and spend
//! most of its tick budget in context switches and cache thrash on a quota that
//! only ever grants 6 cores of runtime.
//!
//! The reported count is therefore the minimum of what the OS says and what the
//! cgroup quota allows, rounded down and never below one.

use std::num::NonZero;
use std::thread;

/// Assumed parallelism when neither the OS nor a cgroup reports a usable value.
pub const AVAILABLE_PARALLELISM_FALLBACK: usize = 4;

/// Stack size for debug-build threads with deep density-function call chains.
pub const DEBUG_STACK_SIZE: usize = 8 * 1024 * 1024;

/// cgroup v2 mounts the unified hierarchy here.
const CGROUP_V2_ROOT: &str = "/sys/fs/cgroup";

/// cgroup v1 mounts each controller under its own directory.
const CGROUP_V1_CPU_ROOT: &str = "/sys/fs/cgroup/cpu";

/// `/proc/self/cgroup` lists the cgroup path of this process per hierarchy.
const PROC_SELF_CGROUP: &str = "/proc/self/cgroup";

/// cgroup v1 keeps the quota and period in separate files under the controller.
const CGROUP_V1_QUOTA: &str = "/sys/fs/cgroup/cpu/cpu.cfs_quota_us";
const CGROUP_V1_PERIOD: &str = "/sys/fs/cgroup/cpu/cpu.cfs_period_us";

/// Returns the number of cores this process may actually run on.
///
/// Takes the smallest of the OS-reported parallelism and any cgroup CPU limit.
/// Returns at least `1`, and [`AVAILABLE_PARALLELISM_FALLBACK`] when the OS
/// reports nothing usable at all.
#[must_use]
pub fn available_worker_threads() -> usize {
    let os_threads = thread::available_parallelism().map_or(AVAILABLE_PARALLELISM_FALLBACK, NonZero::get);
    let cgroup_threads = cgroup_cpu_limit().unwrap_or(os_threads);
    os_threads.min(cgroup_threads).max(1)
}

/// Reads the cgroup CPU allowance as a whole number of cores.
///
/// Returns `None` when no cgroup limit applies (bare metal, `cpu.max` of
/// `max`, or unreadable control files), in which case the OS value is used.
#[must_use]
pub fn cgroup_cpu_limit() -> Option<usize> {
    cgroup_v2_cpu_limit()
        .or_else(cgroup_v1_cpu_limit)
        // A limit below one core still deserves one thread; a quota is rounded
        // down because a fractional core cannot run a thread in parallel.
        .map(|cores| cores.max(1))
}

/// Parses cgroup v2 `cpu.max`, whose format is `"<quota> <period>"` or `"max <period>"`.
fn cgroup_v2_cpu_limit() -> Option<usize> {
    let root = std::fs::read_to_string(CGROUP_V2_ROOT.to_owned() + "/cpu.max");
    if let Ok(contents) = root {
        return parse_cgroup_v2_cpu_max(&contents);
    }

    // The container may not be mounted at the hierarchy root (a host
    // cgroup namespace, or a panel that bind-mounts only its own subtree). In
    // that case the limit lives at the path `/proc/self/cgroup` reports, with
    // the hierarchy root as the mount point.
    let path = v2_relative_path(&std::fs::read_to_string(PROC_SELF_CGROUP).ok()?)?;
    let contents = std::fs::read_to_string(format!("{CGROUP_V2_ROOT}{path}/cpu.max")).ok()?;
    parse_cgroup_v2_cpu_max(&contents)
}

/// Parses cgroup v1 `cpu.cfs_quota_us` / `cpu.cfs_period_us`.
fn cgroup_v1_cpu_limit() -> Option<usize> {
    let quota_path = CGROUP_V1_QUOTA.to_owned();
    let period_path = CGROUP_V1_PERIOD.to_owned();

    // Same fallback as v2: a nested cgroup puts the controller files under the
    // path this process reports instead of the controller root.
    let (quota_path, period_path) = match read_v1_paths() {
        Some((quota, period)) => (quota, period),
        None => (quota_path, period_path),
    };

    let quota: i64 = std::fs::read_to_string(quota_path).ok()?.trim().parse().ok()?;
    if quota <= 0 {
        // The cgroup v1 "no limit" value is `-1`.
        return None;
    }
    let period: i64 = std::fs::read_to_string(period_path)
        .ok()?
        .trim()
        .parse()
        .ok()?;
    quota_to_cores(quota, period)
}

/// Finds the nested cgroup v1 `cpu` controller files, if this process is not at
/// the controller root.
fn read_v1_paths() -> Option<(String, String)> {
    let contents = std::fs::read_to_string(PROC_SELF_CGROUP).ok()?;
    for line in contents.lines() {
        // v1 lines look like `<id>:<controllers>:<path>`; a pure `<id>::<path>`
        // line is the v2 hierarchy and carries no cpu controller files.
        let mut fields = line.splitn(3, ':');
        let _id = fields.next()?;
        let controllers = fields.next()?;
        let path = fields.next()?;
        if !controllers.split(',').any(|controller| controller == "cpu") {
            continue;
        }
        let base = format!("{CGROUP_V1_CPU_ROOT}{path}");
        return Some((format!("{base}/cpu.cfs_quota_us"), format!("{base}/cpu.cfs_period_us")));
    }
    None
}

/// Returns the cgroup v2 path of this process (`/` for the root itself).
///
/// `/proc/self/cgroup` writes the v2 hierarchy as `0::<path>`.
#[must_use]
pub fn v2_relative_path(proc_self_cgroup: &str) -> Option<String> {
    proc_self_cgroup.lines().find_map(|line| {
        let mut fields = line.splitn(3, ':');
        let _id = fields.next()?;
        let controllers = fields.next()?;
        let path = fields.next()?;
        // The v2 hierarchy is the one with no controllers listed.
        if !controllers.is_empty() || !path.starts_with('/') {
            return None;
        }
        Some(path.to_owned())
    })
}

/// Converts a quota/period pair into whole cores, rounding down.
fn quota_to_cores(quota: i64, period: i64) -> Option<usize> {
    if quota <= 0 || period <= 0 {
        return None;
    }
    let cores = quota / period;
    // A sub-core quota still yields `0` here; the caller clamps it to one.
    usize::try_from(cores).ok().or(Some(0))
}

/// Parses the contents of a cgroup v2 `cpu.max` file.
///
/// Extracted so the parsing rules are testable without a cgroup mount.
#[must_use]
pub fn parse_cgroup_v2_cpu_max(contents: &str) -> Option<usize> {
    let mut fields = contents.split_whitespace();
    let quota = fields.next()?;
    if quota == "max" {
        return None;
    }
    let quota: i64 = quota.parse().ok()?;
    let period: i64 = fields.next()?.parse().ok()?;
    quota_to_cores(quota, period)
}

/// Caps an explicit positive worker count to available parallelism, or uses
/// half the available threads with a minimum target of two.
#[must_use]
pub fn worker_threads_for_available(
    configured_threads: Option<usize>,
    available_threads: usize,
) -> usize {
    let available_threads = available_threads.max(1);
    if let Some(configured_threads) = configured_threads.filter(|&threads| threads > 0) {
        return configured_threads.min(available_threads);
    }

    ((available_threads / 2).max(2)).min(available_threads)
}

/// Splits a fixed worker budget across the pools that contend for the same
/// cores, so no pool can starve another and no core is left idle.
///
/// Vanilla runs one thread and ignores the machine. Steel spreads work over a
/// fixed number of cores, and the split matters more than the raw count: the
/// tick loop must never wait on chunk generation, but chunk generation must
/// still saturate the cores the tick loop is not using.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolSizes {
    /// Tokio workers for the tick loop, networking, and gameplay scheduling.
    pub main_runtime: usize,
    /// Tokio workers for chunk tasks (load, light, save, send).
    pub chunk_runtime: usize,
    /// Rayon workers for terrain generation.
    pub chunk_generation: usize,
    /// Rayon workers for chunk serialization and compression.
    pub chunk_encoding: usize,
    /// Blocking workers that drain gameplay packets between ticks.
    pub packet_workers: usize,
}

impl PoolSizes {
    /// Workers that are CPU-bound for most of their life.
    ///
    /// These are the pools that must fit inside the core budget: they run the
    /// tick loop, generate terrain, and serialize chunks. Oversubscribing them
    /// on a cgroup quota is what produces tick spikes.
    #[must_use]
    pub const fn cpu_bound_workers(self) -> usize {
        self.main_runtime + self.chunk_generation + self.chunk_encoding
    }

    /// Workers that mostly wait on I/O (chunk load/save, socket writes, packet
    /// draining). They may exceed the core budget on purpose: they spend most of
    /// their life parked, and blocking them would stall the network.
    #[must_use]
    pub const fn io_workers(self) -> usize {
        self.chunk_runtime + self.packet_workers
    }
}

/// Computes a balanced split for `cores` cores, honouring per-pool overrides.
///
/// The invariant is: [`PoolSizes::cpu_bound_workers`] never exceeds `cores`, so
/// the CPU-bound pools cannot oversubscribe a cgroup quota, while
/// [`PoolSizes::io_workers`] get one worker each because they are idle most of
/// the time and stalling them would stall the network instead of the CPU.
///
/// * `main_runtime` owns the tick loop, the latency-critical path, so it gets the
///   largest single share (~45%).
/// * `chunk_generation` gets everything left after the tick loop and encoding,
///   because terrain generation is the most evenly distributable CPU work.
/// * `chunk_encoding` is deliberately small: encoding is bursty and compressed
///   frames are already handed off worker by worker.
///
/// On a one- or two-core container every pool degrades to a single worker
/// instead of spawning threads that would never be scheduled.
#[must_use]
pub fn pool_sizes_for_available(cores: usize) -> PoolSizes {
    let cores = cores.max(1);
    if cores <= 2 {
        // Too few cores to give every CPU-bound pool its own worker. The tick
        // loop keeps the core(s) and the Rayon pools fall back to a single
        // worker each, which still beats blocking the tick loop on generation.
        // `cpu_bound_workers` is deliberately allowed to exceed the budget here:
        // the pools sleep while they wait for work, so a 2-core box runs three
        // runnable threads at worst and never idles a core.
        return PoolSizes {
            main_runtime: cores,
            chunk_runtime: 1,
            chunk_generation: 1,
            chunk_encoding: 1,
            packet_workers: 1,
        };
    }

    let main_runtime = (cores * 45 / 100).max(1);
    let chunk_encoding = (cores / 10).max(1);
    // Everything the CPU-bound pools have not claimed goes to generation, which
    // keeps the sum equal to `cores` instead of leaving cores idle.
    let chunk_generation = cores
        .saturating_sub(main_runtime + chunk_encoding)
        .max(1);
    let chunk_runtime = (cores / 8).max(1);
    let packet_workers = (cores / 8).max(1);

    PoolSizes {
        main_runtime,
        chunk_runtime,
        chunk_generation,
        chunk_encoding,
        packet_workers,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PoolSizes, parse_cgroup_v2_cpu_max, pool_sizes_for_available, quota_to_cores,
        v2_relative_path, worker_threads_for_available,
    };

    #[test]
    fn explicit_worker_count_is_capped_to_available_threads() {
        assert_eq!(worker_threads_for_available(Some(16), 8), 8);
        assert_eq!(worker_threads_for_available(Some(4), 8), 4);
    }

    #[test]
    fn zero_or_missing_worker_count_uses_auto_default() {
        assert_eq!(worker_threads_for_available(Some(0), 8), 4);
        assert_eq!(worker_threads_for_available(None, 8), 4);
        assert_eq!(worker_threads_for_available(None, 1), 1);
    }

    #[test]
    fn cgroup_v2_quota_becomes_whole_cores() {
        // Pterodactyl `--cpus=6`: 600% of one core over a 100 ms period.
        assert_eq!(parse_cgroup_v2_cpu_max("600000 100000"), Some(6));
        // Docker `--cpus=1.5` rounds down: a thread cannot use half a core alone.
        assert_eq!(parse_cgroup_v2_cpu_max("150000 100000"), Some(1));
        // Unrestricted containers report `max`.
        assert_eq!(parse_cgroup_v2_cpu_max("max 100000"), None);
    }

    #[test]
    fn cgroup_v2_quota_survives_odd_whitespace_and_trailing_newline() {
        assert_eq!(parse_cgroup_v2_cpu_max("  600000\t100000\n"), Some(6));
    }

    #[test]
    fn cgroup_v2_malformed_contents_are_ignored() {
        assert_eq!(parse_cgroup_v2_cpu_max(""), None);
        assert_eq!(parse_cgroup_v2_cpu_max("garbage"), None);
        assert_eq!(parse_cgroup_v2_cpu_max("600000"), None);
    }

    #[test]
    fn sub_core_quotas_clamp_to_zero_before_the_caller_floors_them() {
        assert_eq!(quota_to_cores(50_000, 100_000), Some(0));
        assert_eq!(quota_to_cores(-1, 100_000), None);
        assert_eq!(quota_to_cores(100_000, 0), None);
    }

    #[test]
    fn a_six_core_container_gets_a_balanced_split() {
        let sizes = pool_sizes_for_available(6);
        assert_eq!(sizes.main_runtime, 2);
        assert_eq!(sizes.chunk_encoding, 1);
        // Generation absorbs whatever the other CPU-bound pools left free, so no
        // core sits idle while generation queues work.
        assert_eq!(sizes.chunk_generation, 3);
        assert!(sizes.chunk_runtime >= 1);
        assert!(sizes.packet_workers >= 1);
    }

    #[test]
    fn cpu_bound_pools_never_oversubscribe_the_core_budget() {
        // This is the whole point of the split: on a cgroup quota, spawning more
        // CPU-bound threads than the quota grants only adds context switches.
        //
        // Below three cores the pools cannot all have their own core, so the
        // single-worker fallback is counted separately in
        // `tiny_containers_never_spawn_idle_pools`.
        for cores in 3..=256 {
            let sizes = pool_sizes_for_available(cores);
            assert!(
                sizes.cpu_bound_workers() <= cores,
                "{cores} cores produced {} CPU-bound workers",
                sizes.cpu_bound_workers()
            );
        }
    }

    #[test]
    fn every_core_is_used_by_the_cpu_bound_pools() {
        // Idle cores are wasted cores: the CPU-bound pools must add up to the
        // budget, not to less.
        // Idle cores are wasted cores: the CPU-bound pools must add up to the
        // budget, not to less.
        for cores in 3..=256 {
            let sizes = pool_sizes_for_available(cores);
            assert_eq!(
                sizes.cpu_bound_workers(),
                cores,
                "{cores} cores left {} unused",
                cores - sizes.cpu_bound_workers()
            );
        }
    }

    #[test]
    fn tiny_containers_never_spawn_idle_pools() {
        let sizes = pool_sizes_for_available(1);
        assert_eq!(sizes.main_runtime, 1);
        assert_eq!(sizes.chunk_runtime, 1);

        let sizes = pool_sizes_for_available(2);
        assert_eq!(sizes.main_runtime, 2);
        assert_eq!(sizes.chunk_runtime, 1);
        // The Rayon pools still get exactly one worker each: a zero-worker pool
        // would either panic or silently inherit the host core count.
        assert_eq!(sizes.chunk_generation, 1);
        assert_eq!(sizes.chunk_encoding, 1);
    }

    #[test]
    fn a_zero_core_report_still_produces_a_usable_split() {
        let sizes = pool_sizes_for_available(0);
        assert_eq!(sizes.main_runtime, 1);
        assert!(sizes.cpu_bound_workers() >= 1);
    }

    #[test]
    fn io_pools_keep_at_least_one_worker_on_every_size() {
        for cores in 0..=64 {
            let sizes = pool_sizes_for_available(cores);
            assert!(sizes.chunk_runtime >= 1);
            assert!(sizes.packet_workers >= 1);
            assert!(sizes.chunk_encoding >= 1);
            assert!(sizes.chunk_generation >= 1);
        }
    }

    #[test]
    fn nested_cgroup_v2_path_is_read_from_proc_self_cgroup() {
        let proc_self = "0::/docker/abc123\n";
        assert_eq!(
            v2_relative_path(proc_self).as_deref(),
            Some("/docker/abc123")
        );
    }

    #[test]
    fn the_root_cgroup_is_its_own_path() {
        assert_eq!(v2_relative_path("0::/\n").as_deref(), Some("/"));
    }

    #[test]
    fn a_v1_only_process_has_no_v2_path() {
        // Pure cgroup v1: every line names its controllers, so the unified
        // hierarchy is absent and the v2 fallback must not invent a path.
        let proc_self = "4:cpu,cpuacct:/docker/abc123\n1:name=systemd:/docker/abc123\n";
        assert_eq!(v2_relative_path(proc_self), None);
    }

    #[test]
    fn a_mixed_hierarchy_picks_the_v2_line() {
        let proc_self = "4:cpu:/docker/abc123\n0::/docker/abc123\n";
        assert_eq!(
            v2_relative_path(proc_self).as_deref(),
            Some("/docker/abc123")
        );
    }
}
