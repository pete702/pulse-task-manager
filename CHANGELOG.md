# Changelog
All notable PULSE changes are in this file. Earlier 0.1.x entries are in CHANGELOG_ARCHIVE.md.
## [0.4.129] - 2026-04-12
- **CPU table columns**: residual width correction and truncation guard (B22).
- **CPU table actions**: consistent identity verification across group targets (B23).
- **Docs/README**: version string updated to v0.4.129.

## [0.4.128] - 2026-04-12
- **Storage model**: explicit per-volume read/write summation (B14-B-05).
- **Storage inspector**: HashSet drive filter in group() (B15-B-04).

## [0.4.127] - 2026-04-12
- **UI primitives**: tactical ticks always render (B18-B-02).
- **Sidebar**: Enter activates, request_focus on nav; scope-keyed alert latch (B18-B-03, B18-B-04).
- **Three Walls**: test for both cpu and memory None (B18-B-08).
- **Util/theme fonts**: UTC doc note, MAX_FONT_BYTES constant (B20-B-03, B20-B-04).

## [0.4.126] - 2026-04-12
- **Network chart tests**: restored traffic-window axis test (B16 WHEEL 5).
- **Latency probe tests**: stabilized jitter assertion (B16 WHEEL 6).
- **Network cards**: pinned bandwidth tier and cached KPI strings (B17-B-03).

## [0.4.125] - 2026-04-12
- **Memory blackbox**: badge now reads "ARMED (85% trigger)" (B12-B-01).
- **Storage physical disk**: test-only helper moved into test module (B14-B-09).
- **Storage KPI**: cached formatted strings to avoid per-frame re-allocation (B15-B-12).

## [0.4.124] - 2026-04-12
- **Storage collector**: per-disk counter rollback guard emits zero and re-seeds (B14-B-03).
- **Storage I/O**: documented top-I/O tie-break rule (B14-B-06).
- **Storage inspector**: drive-filter badge shows "N of M procs" (B15-B-05).
- **Storage view**: HashSet for seen volume tracking (B15-B-12).

## [0.4.123] - 2026-04-12
- **Memory history**: removed unused cache key; unified startup single-point guard (B12-B-03, B12-B-09).
- **Memory chart**: removed unused chart_height parameter (B12-B-10).
- **Memory chart HUD**: proportional X-axis tick positions (B12-B-04).
- **Memory top cards**: width breakpoint from MIN_WINDOW_WIDTH (B13-B-05).

## [0.4.122] - 2026-04-12
- **Worker policy**: fast cadence for Cpu and Processes tabs (B05 N-4).
- **Memory delta**: probe-failure cooldown and clamped delta rates (B10-B-03, B10-B-08).
- **Memory collector**: explicit refresh precondition and display-ceiling clamp (B10-B-04, B10-B-05).
- **Downsampling**: bounded threshold<=2 path (B10-B-07).

## [0.4.121] - 2026-04-12
- **Worker policy**: fast process cadence only for Cpu tab (B05 N-4).
- **Self-trim**: documented two-tier working-set trim policy (B06 N-3).
- **Storage layout**: named width thresholds (B03 B-15).
- **Storage view**: removed test-only selected_mount helper (B15-B-08).
- **Docs guard**: case-insensitive dir skip, shared is_cyrillic, zero dead-code attributes check (B20-B-02).

## [0.4.120] - 2026-04-12
- **Network probe**: non-blocking DNS flush via spawned thread; atomic `probe_snapshot()`.
- **Network collector**: rollback guard for counter resets.
- **Network view**: conditional selection write-back.
## [0.4.119] - 2026-04-12
- **Memory chart**: explicit auto-trim badge text and pre-allocated plot series.
- **Storage chart**: clamped tooltip width, binary-search nearest point, no-data fallback, dimmed last toggle.
## [0.4.118] - 2026-04-12
- **Fast cadence**: removed `Memory` from fast process cadence (B05 N-3).
- **Elevation cache**: `is_app_elevated()` cached in `OnceLock<bool>` (B03 N-6).
- **Sidebar badges**: deduplicated via shared helper and bounded `(u8,u16)` cache (B18-B-01).
## [0.4.117] - 2026-04-12
- **Chart kit production**: wired `chart_kit` helpers (nearest point, baseline, frame) in all 4 modules.
- **Latency probe**: production `LatencyProbe` with `Drop` cleanup; removed test-only gating.
- **Storage**: summed group throughput, inspector cache keyed by scan/filter.
## [0.4.116] - 2026-04-12
- **Chart kit**: added `src/ui/chart_kit.rs` with binary-search nearest point and shared baseline.
- **rustfmt skip**: removed all format-skip attributes; docs_guard now checks src.
- **CPU table**: fixed progress bars and added Ctrl+C row copy; trimmed table_state.
## [0.4.115] - 2026-04-12
- **P4 UI state**: removed global `OnceLock`/`Mutex` UI state from storage and network views; state now owned in `PulseApp`.
- **Network probe**: clean worker shutdown command + join; removed dead_code in network/cpu.
## [0.4.114] - 2026-04-12
- **Storage cleanup**: removed `allow(dead_code)` in model/io_agg/view; RAII `VolumeHandle` cache with safe eviction and worker-exit clear (B14-B-01).
- **i18n cleanup**: removed `allow(dead_code)` across frame/keys; gated test-only keys.
## [0.4.113] - 2026-04-12
- **Ring history**: wired `core::ring_history` into storage/history and network/history (monotonic guard, window slicing, capacity).
- **Dead code**: cleaned `#[allow(dead_code)]` in storage, network, memory conversions, collector; named struct for process memory result.
## [0.4.112] - 2026-04-12
- **Ring history**: wired `core::ring_history` into cpu and memory history (monotonic guard, window slicing).
## [0.4.111] - 2026-04-12
- **Ring history**: integrated `core::ring_history` into cpu/history and memory/history (monotonic guard).
- **Platform split**: trim extracted from actions.rs.
## [0.4.110] - 2026-04-12
- **Core**: added `src/core/ring_history.rs` shared ring-buffer helpers and cache key.
## [0.4.109] - 2026-04-12
- **Platform split**: `actions.rs` split into `actions.rs` (terminate) + `trim.rs` (working-set trim).
## [0.4.108] - 2026-04-12
- **P5 cleanup**: removed dead code in storage, network, i18n, theme, util; wired UptimeLabel in chrome.
## [0.4.107] - 2026-04-12
- **Dead code sweep**: removed module-wide `#![allow(dead_code)]` from memory/mod.rs; deleted dead constants/helpers in memory/constants, formatters, history, model.
- **Physical disk**: require >= 48 bytes from DISK_PERFORMANCE.
## [0.4.106] - 2026-04-12
- **Platform move**: real code moved into `src/platform/` (actions, location, qos, win32_api, win32_cpu) with zero re-export shims.
## [0.4.105] - 2026-04-12
- **Platform move**: real code moved into `src/platform/` (handle, protected, target) with zero re-export shims.
## [0.4.104] - 2026-04-12
- **Platform**: added `src/platform/` layer with DWM dark title bar on startup.
- **Spike tie-break**: process_spike now picks largest absolute growth.
## [0.4.103] - 2026-04-12
- **Platform layer**: added `src/platform/` (handle, protected, location, actions, qos, sys_metrics) with DWM dark title bar on startup.
- **License/Docs**: MIT license, rewritten README/ARCHITECTURE, CI linux-clippy job, issue templates.
## [0.4.102] - 2026-04-12
- **License**: switched repository from proprietary to MIT; added MIT LICENSE file.
- **Docs**: rewrote README.md and ARCHITECTURE.md for human users, 100% docs_guard compliant.
- **CI/Templates**: added linux-clippy job and bug/feature issue templates.
## [0.4.100] - 2026-04-12
- **B06 N-1**: restored triangle symbol in window title via pure-ASCII Rust escape.
- **Startup Self-Trim**: one safe K32EmptyWorkingSet self-trim ~5s after startup.
- **B05 N-1/N-2**: removed dead `paused_at_uptime_secs`; resume-apply now handles snapshots with `memory == None`.
- **B06 N-2**: surfaced `write_config_file` failures via explicit error flag; split persist helpers into `persist_codec.rs`.
