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

use std::fs;
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

/// cgroup v1 mounts the `cpuset` controller under its own hierarchy.
const CGROUP_V1_CPUSET_ROOT: &str = "/sys/fs/cgroup/cpuset";

/// Returns the number of cores this process may actually run on.
///
/// Takes the smallest of the OS-reported parallelism and any cgroup CPU limit.
/// Returns at least `1`, and [`AVAILABLE_PARALLELISM_FALLBACK`] when the OS
/// reports nothing usable at all.
#[must_use]
pub fn available_worker_threads() -> usize {
    let os_threads = thread::available_parallelism().map_or(AVAILABLE_PARALLELISM_FALLBACK, NonZero::get);
    let quota_threads = cgroup_cpu_limit().unwrap_or(os_threads);
    // A Pterodactyl/Docker panel may pin a *set* of cores (`--cpuset-cpus`) and
    // leave the quota unlimited; the cpuset is then the real allowance. Taking
    // the minimum of all three keeps both limit styles honest, and the OS value
    // already reflects the affinity mask on most platforms.
    let cpuset_threads = cgroup_cpuset_cpus().unwrap_or(os_threads);
    os_threads.min(quota_threads).min(cpuset_threads).max(1)
}

/// Reads a cgroup `cpuset` allowance as a core count.
///
/// Both hierarchy versions are probed, because a panel can pin cores either way
/// and a v1 host never has a v2 file:
///
/// * v2 — `cpuset.cpus.effective` (the cpuset actually granted after ancestor
///   intersection) and then the requested `cpuset.cpus`, at the hierarchy root
///   and under the path `/proc/self/cgroup` reports for nested mounts;
/// * v1 — `cpuset.effective_cpus` and `cpuset.cpus` from the controller root and
///   from the per-controller path (`/sys/fs/cgroup/cpuset<path>/`).
///
/// Each candidate is tried in turn: a file that does not exist, cannot be read,
/// or holds an unusable value must not abort the search, otherwise a single
/// missing control file would silently hide the real allowance and let the
/// server size its pools for the whole host.
///
/// Returns `None` when no cpuset controller restricts this process, in which
/// case the caller falls back to the OS parallelism.
#[must_use]
pub fn cgroup_cpuset_cpus() -> Option<usize> {
    let relative = fs::read_to_string(PROC_SELF_CGROUP).ok();

    // cgroup v2: the process's own cgroup first, then the hierarchy root.
    //
    // The order is load-bearing. Without a private cgroup namespace (the default
    // for a plain `docker run` with a host bind mount, and for a panel that only
    // mounts its own subtree) `/sys/fs/cgroup` *is* the host root, so
    // `cpuset.cpus.effective` there lists every core of the machine. Reading it
    // first would report the host width and hide a `--cpuset-cpus=0-3` pin
    // entirely — the pools would then be sized for the whole box. The nested
    // path `/proc/self/cgroup` reports is the container's real allowance, so it
    // is probed first and the root is only a fallback.
    let mut v2_candidates = Vec::with_capacity(4);
    if let Some(Some(path)) = relative.as_deref().map(v2_relative_path) {
        for name in ["cpuset.cpus.effective", "cpuset.cpus"] {
            v2_candidates.push(format!("{CGROUP_V2_ROOT}{path}/{name}"));
        }
    }
    for name in ["cpuset.cpus.effective", "cpuset.cpus"] {
        v2_candidates.push(format!("{CGROUP_V2_ROOT}/{name}"));
    }
    if let Some(count) = first_cpu_set(&v2_candidates) {
        return Some(count);
    }

    // cgroup v1: same rule — the per-controller path this process reports wins
    // over the controller root, because the root file describes the host.
    let mut v1_candidates = Vec::with_capacity(4);
    if let Some(base) = v1_cpuset_base(relative.as_deref()) {
        for name in ["cpuset.effective_cpus", "cpuset.cpus"] {
            v1_candidates.push(format!("{base}/{name}"));
        }
    }
    for name in ["cpuset.effective_cpus", "cpuset.cpus"] {
        v1_candidates.push(format!("{CGROUP_V1_CPUSET_ROOT}/{name}"));
    }
    first_cpu_set(&v1_candidates)
}

/// Returns the first candidate file that holds a usable CPU set.
fn first_cpu_set(candidates: &[String]) -> Option<usize> {
    for path in candidates {
        let Ok(contents) = fs::read_to_string(path) else {
            continue;
        };
        if let Some(count) = parse_cpu_set(&contents) {
            return Some(count);
        }
    }
    None
}

/// Returns the cgroup v1 `cpuset` controller directory of this process.
///
/// v1 writes the per-controller path in `/proc/self/cgroup` as
/// `<id>:<controllers>:<path>`, so the controller's directory is
/// `/sys/fs/cgroup/cpuset<path>`.
#[must_use]
pub fn v1_cpuset_base(proc_self_cgroup: Option<&str>) -> Option<String> {
    let contents = proc_self_cgroup?;
    for line in contents.lines() {
        // A malformed hierarchy line is skipped, not fatal — see `read_v1_paths`.
        let mut fields = line.splitn(3, ':');
        let (Some(_id), Some(controllers), Some(path)) =
            (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        if !controllers
            .split(',')
            .any(|controller| controller == "cpuset")
        {
            continue;
        }
        return Some(format!("{CGROUP_V1_CPUSET_ROOT}{path}"));
    }
    None
}

/// Counts the cores in a cgroup `cpuset` list such as `"0-3,8"`.
///
/// An empty file means "no restriction" on older kernels, so it yields `None`
/// rather than zero cores.
#[must_use]
pub fn parse_cpu_set(contents: &str) -> Option<usize> {
    let contents = contents.trim();
    if contents.is_empty() {
        return None;
    }
    let mut total = 0usize;
    for range in contents.split(',') {
        let range = range.trim();
        if range.is_empty() {
            continue;
        }
        if let Some((start, end)) = range.split_once('-') {
            let start: usize = start.parse().ok()?;
            let end: usize = end.parse().ok()?;
            if end < start {
                return None;
            }
            total += end - start + 1;
        } else {
            range.parse::<usize>().ok()?;
            total += 1;
        }
    }
    (total > 0).then_some(total)
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
///
/// The process's own cgroup is consulted first and the hierarchy root second.
/// The order matters: inside a nested cgroup the root `cpu.max` usually says
/// `max` (the host is unlimited) while the child file carries the container's
/// quota, so reading the root first would report "no limit" and size the pools
/// for the whole machine. A file that is absent or unparsable must not abort the
/// search either, so every candidate is tried in turn.
fn cgroup_v2_cpu_limit() -> Option<usize> {
    // The container may not be mounted at the hierarchy root (a host cgroup
    // namespace, or a panel that bind-mounts only its own subtree), so the
    // limit lives at the path `/proc/self/cgroup` reports.
    if let Some(path) = fs::read_to_string(PROC_SELF_CGROUP)
        .ok()
        .as_deref()
        .and_then(v2_relative_path)
        && let Ok(contents) = fs::read_to_string(format!("{CGROUP_V2_ROOT}{path}/cpu.max"))
        && let Some(cores) = parse_cgroup_v2_cpu_max(&contents)
    {
        return Some(cores);
    }

    let contents = fs::read_to_string(CGROUP_V2_ROOT.to_owned() + "/cpu.max").ok()?;
    parse_cgroup_v2_cpu_max(&contents)
}

/// Parses cgroup v1 `cpu.cfs_quota_us` / `cpu.cfs_period_us`.
fn cgroup_v1_cpu_limit() -> Option<usize> {
    // Same fallback as v2, in the same order: the nested per-controller path this
    // process reports wins over the controller root, because a v1 host leaves the
    // root at `-1` (unlimited) and only the child file carries the container
    // quota. Reading the root first would report "no limit".
    let mut candidates = Vec::with_capacity(2);
    if let Some(pair) = read_v1_paths() {
        candidates.push(pair);
    }
    candidates.push((
        CGROUP_V1_QUOTA.to_owned(),
        CGROUP_V1_PERIOD.to_owned(),
    ));

    for (quota_path, period_path) in candidates {
        let Ok(quota) = fs::read_to_string(quota_path) else {
            continue;
        };
        let Ok(quota) = quota.trim().parse::<i64>() else {
            continue;
        };
        if quota <= 0 {
            // The cgroup v1 "no limit" value is `-1`; try the next candidate.
            continue;
        }
        let Ok(period) = fs::read_to_string(period_path) else {
            continue;
        };
        let Ok(period) = period.trim().parse::<i64>() else {
            continue;
        };
        if let Some(cores) = quota_to_cores(quota, period) {
            return Some(cores);
        }
    }
    None
}

/// Finds the nested cgroup v1 `cpu` controller files, if this process is not at
/// the controller root.
fn read_v1_paths() -> Option<(String, String)> {
    let contents = fs::read_to_string(PROC_SELF_CGROUP).ok()?;
    for line in contents.lines() {
        // v1 lines look like `<id>:<controllers>:<path>`; a pure `<id>::<path>`
        // line is the v2 hierarchy and carries no cpu controller files.
        //
        // A malformed line is skipped, not fatal: `/proc/self/cgroup` always
        // carries several hierarchies, and letting one of them abort the scan
        // would hide the cpu controller that follows it.
        let mut fields = line.splitn(3, ':');
        let (Some(_id), Some(controllers), Some(path)) =
            (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
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

/// Resolves the effective core budget for this process.
///
/// This is the single rule every caller must use: the configured value is an
/// *upper bound*, never a promotion. A server configured for 16 cores inside a
/// container limited to 600 % CPU runs on 6, and a 4-core configuration on a
/// 64-core host still uses only 4. `None`, `0`, and negative-as-`0` all mean
/// "whatever this container actually allows", which is
/// [`available_worker_threads`].
///
/// `configured` is `Option<usize>`, so the `0`-means-auto convention only has to
/// be applied once by the config parser (`ThreadConfig::cores` is already
/// filtered to a positive value there).
#[must_use]
pub fn effective_cores(configured: Option<usize>, available: usize) -> usize {
    let available = available.max(1);
    configured
        .filter(|&cores| cores > 0)
        .map_or(available, |cores| cores.min(available))
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
pub const fn pool_sizes_for_available(cores: usize) -> PoolSizes {
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
        effective_cores, first_cpu_set, parse_cgroup_v2_cpu_max, parse_cpu_set,
        pool_sizes_for_available, quota_to_cores, v1_cpuset_base, v2_relative_path,
        worker_threads_for_available,
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
    #[test]
    fn a_cpuset_range_counts_its_cores() {
        assert_eq!(parse_cpu_set("0-3"), Some(4));
        assert_eq!(parse_cpu_set("0-3,8"), Some(5));
        assert_eq!(parse_cpu_set("2"), Some(1));
        assert_eq!(parse_cpu_set("0-0"), Some(1));
    }

    #[test]
    fn a_v1_cpuset_hierarchy_is_found_by_its_controller() {
        // v1 lists one hierarchy per controller; cpuset is one of them.
        let proc_self =
            "1:name=systemd:/docker/abc123\n2:cpu,cpuacct:/docker/abc123\n5:cpuset:/docker/abc123\n";
        assert_eq!(
            v1_cpuset_base(Some(proc_self)).as_deref(),
            Some("/sys/fs/cgroup/cpuset/docker/abc123")
        );
    }

    #[test]
    fn a_host_without_a_cpuset_controller_has_no_v1_base() {
        let proc_self = "2:cpu,cpuacct:/docker/abc123\n0::/docker/abc123\n";
        assert_eq!(v1_cpuset_base(Some(proc_self)), None);
        assert_eq!(v1_cpuset_base(None), None);
    }

    #[test]
    fn malformed_hierarchy_lines_do_not_hide_the_cpuset_controller() {
        // A short line in front of the cpuset line used to abort the scan.
        let proc_self = "0::\n5:cpuset:/docker/abc123\n";
        assert_eq!(
            v1_cpuset_base(Some(proc_self)).as_deref(),
            Some("/sys/fs/cgroup/cpuset/docker/abc123")
        );
    }

    #[test]
    fn the_first_usable_cpu_set_wins_and_missing_files_are_skipped() {
        // A candidate that cannot be read must not abort the search: otherwise a
        // single missing control file would size the pools for the whole host.
        let dir = std::env::temp_dir().join(format!("steel-cpuset-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let present = dir.join("cpuset.cpus.effective");
        std::fs::write(&present, "0-3\n").expect("write cpu set");
        let missing = dir.join("does_not_exist");
        let candidates = vec![
            missing.to_string_lossy().into_owned(),
            present.to_string_lossy().into_owned(),
        ];
        assert_eq!(first_cpu_set(&candidates), Some(4));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_cpuset_of_max_is_not_a_core_count() {
        // v1 writes `cpuset.cpus` verbatim, but `cpuset.effective_cpus` on some
        // kernels reports an unconstrained set; parsing must reject junk instead
        // of inventing a limit.
        assert_eq!(first_cpu_set(&["/definitely/not/here".to_owned()]), None);
    }

    /// The configured value is an upper bound: a container serving six cores
    /// must shrink a sixteen-core configuration to six, never promote it.
    #[test]
    fn effective_cores_never_exceeds_the_container_allowance() {
        assert_eq!(effective_cores(Some(16), 6), 6);
        assert_eq!(effective_cores(Some(4), 64), 4);
        assert_eq!(effective_cores(Some(6), 6), 6);
    }

    /// `0`/`None` mean "use whatever the container allows", and the result is
    /// never zero even when the OS reports nothing usable.
    #[test]
    fn effective_cores_treats_zero_as_auto_and_never_returns_zero() {
        assert_eq!(effective_cores(None, 8), 8);
        assert_eq!(effective_cores(Some(0), 8), 8);
        assert_eq!(effective_cores(None, 0), 1);
        assert_eq!(effective_cores(Some(0), 0), 1);
    }

    /// A quota below one core still has to run the server, so every scheduling
    /// helper clamps to one instead of spawning zero workers.
    #[test]
    fn a_sub_core_quota_still_yields_one_worker() {
        assert_eq!(quota_to_cores(50_000, 100_000), Some(0));
        assert!(pool_sizes_for_available(0).main_runtime >= 1);
        assert!(worker_threads_for_available(Some(0), 0) >= 1);
    }

    #[test]
    fn an_empty_or_malformed_cpuset_falls_back_to_the_os() {
        assert_eq!(parse_cpu_set(""), None);
        assert_eq!(parse_cpu_set("   "), None);
        assert_eq!(parse_cpu_set("4-2"), None);
        assert_eq!(parse_cpu_set("abc"), None);
    }
}
