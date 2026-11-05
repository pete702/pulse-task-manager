# Changelog

All notable changes to **PULSE** (`pulse_manager`) are documented in this file.
Binary assets and checksums are published on [GitHub Releases](https://github.com/pete702/pulse-task-manager/releases).

## [0.1.17] - 2026-11-05

- **Flush-Aligned Chart**: Memory chart now uses `.show_axes([false, false])` + `.show_grid([true, true])` for pixel-perfect left/right alignment with the `MEMORY COMPOSITION` bar above; no more gutter gaps.
- **Translucent In-Canvas HUD Legend**: Removed opaque `egui_plot` legend box; custom HUD legend drawn at top-left of plot canvas with filled circles (cyan RAM, purple Commit/Swap) and monospace labels — zero background, translucent text.
- **HUD Axis Labels Refined**: X-axis time labels now use `plot_rect.left() + 8.0` / `plot_rect.right() - 8.0` padding for first/last ticks so `-60s`/`-5m`/`-15m` and `0s`/`0m` are never clipped.
- **Font RAM Savings**: Removed redundant `seguisym.ttf` loading (~6 MB glyph cache / GPU texture atlas saved); all geometric symbols (`•`, `▲`, `▼`, `✓`, `[01]..[05]`, `||`, `>`) natively supported by `segoeui.ttf` + `consola.ttf`.
- **Zero-Allocation Process Search**: Added `contains_ci()` helper using `as_bytes().windows().eq_ignore_ascii_case()` for ASCII queries — no `to_lowercase()` allocations per process row per scan; falls back to lowercase only for non-ASCII.
- **README Latest Release Link**: Stale `v0.1.14` download link replaced with version-independent `releases/latest`; guarded by `docs_guard` test.

## [0.1.16] - 2026-11-05

- **Chart Growth Fix**: `anchored()` now only anchors right edge to `0.0`; left edge grows naturally leftward as history accumulates — no more full-width jump on sample #2.
- **Flush Full-Width Plot**: Chart now uses full card width/height; external time-scale row removed.
- **In-Canvas HUD Axis Labels**: Y-axis `%` labels (100/75/50/25) drawn inside right edge; X-axis time labels drawn inside bottom edge — both translucent monospace.
- **Font Fallback**: Added `seguisym.ttf` (Segoe UI Symbol) as fallback for geometric symbols in both Proportional and Monospace families.
- **Emoji → Geometric Symbols**: All UI emojis replaced with crisp geometric symbols (`▲`, `▣`, `◧`, `×`, `◻`, `||`, `>`, `•`, `[01]..[05]` prefixes) — no more `?` missing-glyph boxes.

## [0.1.15] - 2026-11-05

- **Zero Steady-State Syscalls**: `DeltaTracker` single-pass with cached `creation_time` (FILETIME) — eliminates ~70 `OpenProcess`/`GetProcessTimes` syscalls/s in steady state.
- **Conditional `Arc<str>` Formatting**: `ProcessMemoryItem::update_formatted` now only re-allocates display strings when underlying numeric values change (RAM, virtual, delta, process count).
- **Startup Chart Fix**: Single-point series renders a 1-second lead-in `[-1.0, y]..[0.0, y]` instead of stretching across the full window.
- **Repaint Watchdog**: Fallback `request_repaint_after` reduced from 30 s to 2 s; minimized mode stays at 0 repaints.
- **Cache Precision**: `TableCacheKey` now stores full `search_query: String` (no hash collisions); dead helper functions removed.
- **Thread-Spam Guard**: `MemoryActionBarState::spawn` refuses new jobs while one is pending (0 deps, 2 lines).
- **Automated Releases**: GitHub Actions `release.yml` builds on `stable` toolchain and publishes `PULSE_vX.Y.Z_windows_x64.exe` on tag push.

## [0.1.14] - 2026-11-04

- **Adaptive Telemetry Cadence**: Background worker scans every `3 s` while the window is visible, `5 s` in the background, and stops rendering (`0 fps`) when minimized; event-driven UI wakeup plus zero-allocation `Arc<str>` process rows.
- **Working-Set Trim Restored**: Single post-startup frame-#3 trim and a minimize-transition trim bring physical RAM back to `27-29 MB` with `0%` idle CPU.
- **Phase 1 Memory UX Polish**: `[ Pause / Resume ]` live-stream toggle, right-click process context menu (`Open Location`, `Terminate`, `Copy Name & PID`), double-click row action, `Ctrl+F` / `Esc` / `[X]` search ergonomics with an empty-state notice, and CLI copyright attribution.
- **Modular Internals**: `worker.rs` split into `worker_io` / `worker_policy` / `worker_loop`; table body split into context-menu, sort, and name-cell modules; 280-line module limit enforced; 131 tests passing (127 unit + 4 `docs_guard`).

## [0.1.11] - 2026-11-04

- **Phase 1 Memory Subsystem Frozen**: 3-segment RAM composition bar, live Commit % chart (`1m/5m/15m`), frame-aligned time scale.
- **Unified O(1) Table Cache & Identity Gates**: `MemoryTableState` keyed by `process_scan_id`; `verify_handle_identity` fails closed.
- **dGPU Sleep Exports & Kernel Tiles**: Zeroed 3D buffers cut ~29 MB working set.
- **130 Tests Passing**: 126 unit + 4 `docs_guard`; release build verified.

## [0.1.2] - 2026-11-04

- **Zero-Copy `Arc` Polling**: System RAM 1 Hz, process scans 0.5 Hz; minimized pauses scans, shares `Arc<Vec<RawProcessRow>>` zero-copy.
- **O(1) UI Table & Selection Cache**: `MemoryTableState` keyed by `process_scan_id` short-circuits on unchanged scan ID.
- **3-Point PID Identity & Frozen Kills**: `basename + creation_time + full_path` checked before `EmptyWorkingSet` / `TerminateProcess`.
- **Single Frame-#3 Self-Trim & CLI Flags**: One `K32EmptyWorkingSet` after GPU init; `--version` / `--help` exit cleanly.

## [0.1.1] - 2026-11-03

- **Raw Process Actions**: `Trim Working Sets`, `Trim Selected`, `Open Location`, `End Task` target raw rows with explicit PID.
- **`dt`-Normalized `Δ RAM/s`**: Per-process deltas divide by real sample interval.
- **Minute-Based `1m/5m/15m` Chart**: X-axis in minutes with right-edge padding; worker `LatestSlot` keeps newest snapshot.
- **Rust 1.85 CI Lockfile Fix**: MSRV pinned to `rust-version = "1.85"`; locked deps, fmt/clippy/test/build on `stable`.

## [0.1.0] - 2026-11-03

- **Phase 1 Memory Subsystem Complete**: Non-blocking Action Bar with trim, end task, open location, `ReclaimReport` audit.
- **Three Walls Badge & Chart Selector**: Live `SYSTEM OPTIMAL` / `MEMORY WALL` badge; cached `PlotPoints` for zero-realloc switching.
- **5-Column Aligned Process Table**: Column defs match header/row 1-to-1; selection via Process Name cell.
- **Audit Fixes**: Removed all fmt skip attrs; 180-line limit; panic log hook; `[0u16; 1024]` identity buffer.

## [0.0.16] - 2026-11-03

- **TOCTOU Handle Identity Verification**: `verify_handle_identity` via `QueryFullProcessImageNameW` with `IsProcessCritical` checks.
- **Resource P1 Optimizations**: `search_hash: u64` + `uptime_secs` cache key, cached uptime formatting, reusable delta `HashSet<u32>`.
- **CI & Repo Hygiene**: `--locked` on all cargo commands, MSRV `1.85` CI job, recursive guard for `0` NUL/Cyrillic/line limits.

## [0.0.15] - 2026-11-03

- **Win32 Safety & Path Normalization**: `SafeHandle` raw field private, `OpenProcessToken` RAII, `IsProcessCritical` checks.
- **Accurate Trim Accounting & Explorer Launch**: Fixed post-trim `K32GetProcessMemoryInfo` handling; `CommandExt::raw_arg` for paths with spaces.
- **Documentation & Placeholder Sync**: Synced `ARCHITECTURE.md` tokens with `src/theme.rs` / `src/main.rs`.

