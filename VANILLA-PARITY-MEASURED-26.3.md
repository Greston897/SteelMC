# Measured implementation audit against vanilla 26.3

This report is recomputed from source files. It measures **implementation presence**, not correctness: a Rust method existing does not prove it matches vanilla. It never prints a global parity percentage, because no complete weighted list of vanilla mechanics exists. Unknowns stay unknown.

Vanilla source snapshot: `minecraft-src`.

| Domain | Registry entries | Vanilla classes | Steel behavior classes | Missing classes | Explicit methods | Defaulted | Not explicit | Unknown | Client-only |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| blocks | 1286 | 268 | 267 | 0 | 734 | 447 | 0 | 289 | 346 |
| items | 1658 | 66 | 65 | 0 | 54 | 30 | 0 | 66 | 7 |
| entities | 161 | 143 | 144 | 0 | 273 | 0 | 2429 | 2 | 28 |
- **blocks**: 734/1181 mapped server-relevant vanilla overrides are explicitly overridden in Steel (62.15%); 447 rely on a trait default body.
- **items**: 54/84 mapped server-relevant vanilla overrides are explicitly overridden in Steel (64.29%); 30 rely on a trait default body.
- **entities**: no method percentage (entity trait hierarchy; see JSON for per-class counts).

### Most common unmapped vanilla methods

These have no Steel trait method with the mapped name. They are excluded from the denominator; many are client-only or handled by generic world/registry code, but each one is unverified by this audit.

- **blocks**: `get_fluid_state` (40), `get_menu_provider` (10), `get_block_support_shape` (9), `get_shade_brightness` (7), `skip_rendering` (6), `get_shape` (5), `get_dust_color` (4), `get_base_seed_id` (4), `get_type` (4), `get_blocks_to_grow_when_bonemealed` (4), `can_grow_into` (4), `get_body_block` (4)
- **items**: `use` (26), `try_apply_to_sign` (4), `get_name` (2), `as_projectile` (2), `check_extra_content` (2), `append_hover_text` (2), `shoot` (2), `get_placement_state` (2), `override_stacked_on_other` (1), `override_other_stacked_on_me` (1), `is_bar_visible` (1), `get_bar_width` (1)
- **entities**: `ride_height` (2)

## Reading the numbers

- `Missing classes` are vanilla classes with no Steel behavior struct at all. This is a real gap in the registered behavior map, not a guess.
- `Explicit methods` are vanilla `@Override` methods whose mapped Steel trait method is overridden by the struct.
- `Defaulted` means the Steel trait supplies a default body. That default may be correct, but the struct does not handle the vanilla override explicitly.
- `Not explicit` (entity domain) means no same-named method exists in the Steel struct's impl blocks. Steel implements an entity trait hierarchy, so a generic default may still handle it; this is not counted as done or as missing.
- `Unknown` methods have no name mapping; they are excluded from the denominator and are neither counted as done nor as missing.
- `Client-only` methods are excluded because Steel's server has no server-side work for them.

## Per-class method detail

Only classes whose mapped Steel trait method is required (no default body) and not overridden are listed. `defaulted` and entity `not_explicit` lists are in the JSON report.
