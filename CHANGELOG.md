# Changelog

All notable changes to **PULSE** (`pulse_manager`) are documented in this file.
Binary assets and checksums are published on [GitHub Releases](https://github.com/pete702/pulse-task-manager/releases).

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

- **Phase 1 Memory Subsystem Frozen**: 3-segment RAM composition bar (In Use / Cached / Free), live Commit % history chart (`1m/5m/15m`), frame-aligned time scale.
- **Unified O(1) Table Cache & Identity Gates**: `MemoryTableState` keyed by `process_scan_id`; `verify_handle_identity` fails closed without a valid `creation_time`; frozen `PendingKillConfirm` targets.
- **dGPU Sleep Exports & Kernel Tile Fixes**: Zeroed 3D buffers cut ~29 MB of working set; tiles sized from inner width with correct 100% bar allocation.
- **130 Tests Passing**: 126 unit + 4 `docs_guard`; release build verified.

## [0.1.2] - 2026-11-04

- **Zero-Copy `Arc` Polling**: System RAM at 1 Hz, process scans at 0.5 Hz (2 s); minimized mode pauses scans and shares `Arc<Vec<RawProcessRow>>` zero-copy (~0.12 ms fast ticks).
- **O(1) UI Table & Selection Cache**: `MemoryTableState` keyed by `process_scan_id` short-circuits on unchanged scan ID, filter, sort, or grouping; `SelectedGroupSummary` never re-iterates raw rows.
- **3-Point PID Identity & Frozen Kills**: `basename + creation_time + full_path` checked before `EmptyWorkingSet` / `TerminateProcess`; first `[ End Task ]` click freezes the exact target list; background `Open Location` with `OnceLock` elevation.
- **Single Frame-#3 Self-Trim & CLI Flags**: One `K32EmptyWorkingSet` after GPU init; `--version` / `--help` exit cleanly without launching the GUI; 124 tests passing.

## [0.1.1] - 2026-11-03

- **Raw Process Actions**: `Trim Working Sets`, `Trim Selected`, `Open Location`, and `End Task` target raw process rows with explicit PID; `exe_path` feeds `Open Location`.
- **`dt`-Normalized `Δ RAM/s`**: Per-process deltas divide by the real sample interval, removing 1 Hz drift from the leak hunter.
- **Minute-Based `1m/5m/15m` Chart**: X-axis labeled in minutes with right-edge padding; worker `LatestSlot` keeps the newest snapshot when bounded channels are full.
- **Rust 1.85 CI Lockfile Fix**: MSRV pinned to `rust-version = "1.85"`; locked dependencies, fmt/clippy/test/build all run on `stable`.

## [0.1.0] - 2026-11-03

- **Phase 1 Memory Subsystem Complete**: Non-blocking Memory Action Bar (`action_bar.rs`) with working-set trim, trim selected, end task, open file location, and Before/After `ReclaimReport` audit.
- **Three Walls Badge & Chart Selector**: Live `SYSTEM OPTIMAL` / `MEMORY WALL` / `COMPUTE WALL` / `I/O WALL` header badge; cached `PlotPoints` for zero-realloc `1m/5m/15m` switching.
- **5-Column Aligned Process Table**: Column definitions match header/row cells 1-to-1; selection via the Process Name cell; worker `LatestSlot` channel for newest-snapshot access.
- **Audit Fixes**: Removed all 24 `rustfmt` skip attributes; 180-line limit for all `.rs` files in `src/` and `tests/`; panic log hook in `main.rs`; `[0u16; 1024]` identity buffer in `safe_handle.rs`.

## [0.0.16] - 2026-11-03

- **TOCTOU Handle Identity Verification**: `verify_handle_identity` via `QueryFullProcessImageNameW` with enforced `IsProcessCritical` checks in trim and terminate paths; cross-platform `CommandExt` gating.
- **Resource P1 Optimizations**: `search_hash: u64` + `uptime_secs` cache key, cached uptime formatting, reusable delta `HashSet<u32>`, and a compacted process table body.
- **CI & Repo Hygiene**: `--locked` on all cargo commands, MSRV `1.85` CI job, and a recursive guard enforcing `0` NUL bytes, `0` Cyrillic characters, and per-file line limits.

## [0.0.15] - 2026-11-03

- **Win32 Safety & Path Normalization Fixes**: Made `SafeHandle` raw field private, wrapped `OpenProcessToken` in RAII, added `IsProcessCritical` checks, and fixed full-path normalization in `verify_pid_safety_gate`.
- **Accurate Trim Accounting & Explorer Launch**: Fixed post-trim `K32GetProcessMemoryInfo` failure handling in `ReclaimReport` and switched `open_process_file_location` to `CommandExt::raw_arg` for paths with spaces.
- **Documentation & Placeholder Sync**: Synced `ARCHITECTURE.md` theme/FFI tokens with `src/theme.rs` and `src/main.rs`, and matched placeholder tab versions to the roadmap.

