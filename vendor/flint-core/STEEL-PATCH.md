# Narrow Steel integration patch

Upstream: FlintTestMC/flint-core v1.2.2,
commit 1bcea1099cde5a90dbbeda65eff0584206efcb90, MIT (see LICENSE).

Only traits.rs and runner.rs are intentionally patched: the runner must apply
WorldConfig.time and WorldConfig.weather before executing the timeline. Upstream
parsed both fields but only applied gamerules. The new configure_world hook fails
explicitly on unsupported adapters instead of silently accepting ignored setup.
Steel's adapter calls production clock-marker and weather-state APIs.

No benchmark specs, expected values, assertion rules, or skipping/filtering rules
are changed. Results after this patch must be reported separately from the old
runner: environments are now actually initialized as the benchmarks request.
