# Changelog

All notable PULSE changes are in this file. Earlier 0.1.x entries are in CHANGELOG_ARCHIVE.md.

## [0.4.65] - 2026-03-31

- **CPU Hotkeys/UI**: Escape now surrenders CPU search focus unconditionally on the CPU tab.
- **CPU Hardening/Tests**: extracted grid/bottleneck helpers and added regression tests for grid sizing, labels, badges, and quick filters.
- **Version bump to 0.4.65**.

## [0.4.64] - 2026-03-31

- **CPU Chart**: cached Y-grid marks, preallocated time conversion/anchoring, and tactical scanner hover/cursor cleanup.
- **CPU UI/State**: single Escape path for CPU search, CPU sub-view toggle, and shared min window width constant.
- **Version bump to 0.4.64**.

## [0.4.63] - 2026-03-31

- **CPU Chart**: fixed legend rect lag/double frame/comment, separated legend state/button IDs, deduped/cached X grid marks, and added chart axis tests.
- **Tests**: asserted exact priority class restoration in live EcoQoS roundtrip.
- **Version bump to 0.4.63**.

## [0.4.62] - 2026-03-31

- **CPU UI**: fixed bottleneck sentinel/idle states, tactical ticks at 0%, narrow layout thresholds, and core grid overflow/hover allocs.
- **CPU State/Hotkeys**: added `show_core_grid` accessors, wired `CpuSubView` dispatch, and made process hotkeys focus-aware.
- **Version bump to 0.4.62**.

## [0.4.61] - 2026-03-31

- **Collector EcoQoS**: bounded probe attempts per scan (max 24) to cap Win32 probes when results are unavailable.
- **CPU Freq**: thread-local frequency buffer in `win32_cpu` to avoid holding a Mutex across the Win32 call.
- **Version bump to 0.4.61**.

## [0.4.60] - 2026-03-31

- **Build/Deps**: cleaned local build artifacts; trimmed unused default features for `egui_extras` and `windows-sys` in `Cargo.toml`.
- **Tests**: added live Windows kernel EcoQoS enable/restore roundtrip test.
- **Version bump to 0.4.60**.

## [0.4.59] - 2026-03-31

- **qos_action**: probe returns `Some(false)` for PID 0/4 without budget; tests moved to `collector_tests.rs` to decompress module.
- **CPU Table**: `ProcessQuickFilter` moved to `quick_filter.rs` with explicit numeric mapping; simplified delta scale; capped `eco_overrides` on rollback insert.
- **Version bump to 0.4.59**.

## [0.4.58] - 2026-03-31

- **EcoQoS Safety**: oldest-first eviction for pending/orig maps, no silent disable-miss priority, atomic priority/throttling order with rollback, stricter Eco detection and probe budget.
- **Collector/Tests**: deterministic round-robin Eco probe test, `disk_rates_from_prev` helper, removed test-only disk helpers, no `seen_pids` clone per scan.
- **CPU Freq**: MAX_CORES 256 with reusable frequency buffer in `win32_cpu`.
- **Version bump to 0.4.58**.

## [0.4.57] - 2026-03-31

- **Eco Rollback**: optimistic overrides restored on failure, `apply_eco_rollbacks` via action channel.
- **Delta and Hover**: 10s scale clamped to [1/30, 10] with 1.0s floor, hover uses `resolve_exe_cached`.
- **Version bump to 0.4.57**.

## [0.4.56] - 2026-03-31

- **Group Identity**: `GroupTarget` with start/creation/exe, imposter path guard, group trim/eco/kill/open with reuse checks.
- **Menu Split**: actions moved to `table_menu_actions.rs` to keep files <= 265 lines, tests compacted.
- **Version bump to 0.4.56**.

## [0.4.55] - 2026-03-31

- **Sort and Search**: navigation uses `ensure_sort`, Unicode `contains_ci` with ASCII fast path, header still toggles.
- **Hover Cache**: TTL fallback, no sticky insert when saturated, Drop-guarded in-flight, baseline eviction and 0.05s dt floor.
- **Version bump to 0.4.55**.

## [0.4.54] - 2026-03-31

- **Grouped Hog Fix**: single `from_raw` path, clamped group CPU, per-member hog flag, zero-alloc `pid_contains`.
- **Creation Cache Bound**: max 64 Win32 probes per scan, no permanent `None`, `collect_process_cpu` call simplified.
- **Version bump to 0.4.54**.

## [0.4.53] - 2026-03-31

- **Round-Robin Eco**: new `collector_eco` cache (budget 12, TTL 300s, cursor, prune), `last_scan` field, `disk_rate` helper, exe paths live.
- **Model Harden**: NaN-safe `new`, `from_raw` without double format, removed duplicate builder.
- **Version bump to 0.4.53**.

## [0.4.52] - 2026-03-31

- **EcoQoS Harden**: single-key eviction, `probe->Option<bool>`, no tautological path, ORIG priority remove, prio==0 guard.
- **Open Location Gates**: system/gate/critical/identity checks, `decode_utf16_cstr` NUL strip, dead telemetry removed.
- **Version bump to 0.4.52**.

## [0.4.51] - 2026-03-31

- **Non-Blocking Hover**: exe cache never queries on UI thread (bg thread, 4 in-flight, fallback insert).
- **Persisted Widths**: `PERSISTED_WIDTHS` slot + `cpu_col_widths` in state.cfg, saved on drag/autofit.
- **Version bump to 0.4.51**.


