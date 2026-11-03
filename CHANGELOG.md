# Changelog

All notable changes to **PULSE** (`pulse_manager`) are documented in this file.
Binary assets and checksums are published on [GitHub Releases](https://github.com/pete702/pulse-task-manager/releases).

## [0.1.1] - 2026-11-03

- **Raw Process Actions**: `Trim Working Sets`, `Trim Selected`, `Open Location`, and `End Task` now target raw process rows with explicit PID; `exe_path` feeds `Open Location`.
- **`dt`-Normalized `Δ RAM/s`**: Per-process deltas divide by the real sample interval, removing 1 Hz drift from the leak hunter.
- **Minute-Based `1m/5m/15m` Chart**: X-axis is labeled in minutes; the 1m view shows only `-1m`, `-0.5m`, and `0m` with right-edge padding.
- **Worker `LatestSlot` + Minimized Throttling**: Full bounded channels fall back to the newest-slot cache without dummy snapshots; process scans run every 5th tick while minimized.
- **Rust 1.78.0 CI Lockfile Fix**: MSRV pinned to `rust-version = "1.78"`; locked dependencies, fmt/clippy/test/build all run on `1.78.0`.

## [0.1.0] - 2026-11-03

- **Phase 1 Memory Subsystem Complete**: Non-blocking Memory Action Bar (`action_bar.rs`) with working-set trim (`K32EmptyWorkingSet`), trim selected, end task, open file location, and Before/After `ReclaimReport` audit.
- **Three Walls Bottleneck Badge** (`three_walls.rs`): Live `SYSTEM OPTIMAL` / `MEMORY WALL` / `COMPUTE WALL` / `I/O WALL` indicator in top header.
- **Memory Chart `1m/5m/15m` Selector** (`chart_view.rs`): Visible header buttons with cached `PlotPoints` keyed by `(history.len(), window_secs)` for zero-realloc switching.
- **5-Column Aligned Process Table** (`table_view.rs`, `table_body.rs`): Column definitions match header/row cells 1-to-1; row selection via click on Process Name cell.
- **Worker `LatestSlot` Channel** (`worker.rs`): `Arc<Mutex<Option<Arc<SystemSnapshot>>>>` for newest-snapshot access; on channel Full, drains and retries once.
- **Panic Log Hook** (`main.rs`): Custom panic hook writes to `%TEMP%\pulse_panic.log` for headless CI diagnostics.
- **Audit Fixes**: Removed all 24 `rustfmt` skip attributes; unified line limit to 180 for all `.rs` files in `src/` and `tests/`; CI workflow uses stable-only main job with separate MSRV `1.87.0` job; `--locked` on self-test.
- **SafeHandle 1024 Buffer** (`safe_handle.rs`): `verify_handle_identity` uses `[0u16; 1024]` buffer for `QueryFullProcessImageNameW`.

## [0.0.16] - 2026-11-03

- **TOCTOU Handle Identity Verification**: Added `verify_handle_identity` using `QueryFullProcessImageNameW` and enforced `IsProcessCritical` checks in both `trim_single_pid` and `terminate_process_group` (engine only in `v0.0.16`, UI wiring in `v0.1.0`).
- **Cross-Platform Fixes**: Wrapped `CommandExt` in `#[cfg(target_os = "windows")]`; `open_process_file_location` resolves `explorer.exe` via `%SystemRoot%` environment variable.
- **Resource P1 Optimizations**: `search_hash: u64` + `uptime_secs` table cache key (zero per-frame `String` allocations); 1 Hz `cached_format_uptime` with `OnceLock<Mutex<(u64, String)>>`; reusable `HashSet<u32>` in `ProcessDeltaTracker`; full-tick jitter measurement; compacted `table_body.rs` to 158 lines.
- **CI & Repo Hygiene**: `--locked` on all cargo commands + MSRV `1.78.0` toolchain check in `.github/workflows/ci.yml`; full-repo recursive walker in `tests/docs_guard.rs` enforcing `0` NUL bytes, `0` Cyrillic (`U+0400..=U+052F`), and `<= 165` lines for `src/**/*.rs`.

## [0.0.15] - 2026-11-03

- **Win32 Safety & Path Normalization Fixes**: Made `SafeHandle` raw field private, wrapped `OpenProcessToken` in RAII, added `IsProcessCritical` checks, and fixed full-path normalization in `verify_pid_safety_gate`.
- **Accurate Trim Accounting & Explorer Launch**: Fixed post-trim `K32GetProcessMemoryInfo` failure handling in `ReclaimReport` and switched `open_process_file_location` to `CommandExt::raw_arg` for paths with spaces.
- **Documentation & Placeholder Sync**: Synced `ARCHITECTURE.md` theme/FFI tokens with `src/theme.rs` and `src/main.rs`, and matched placeholder tab versions to the roadmap.

## [0.0.14] - 2026-11-03

- **Zero-Clone Frame Pipeline**: Replaced per-frame `SystemSnapshot` and `Vec<ProcessMemoryItem>` clones with `Arc<SystemSnapshot>` and `Arc<[ProcessMemoryItem]>`, moved `apply_obsidian_theme` to one-time startup init, and cached Win32 `GetUserDefaultUILanguage` in `OnceLock`.
- **Live 1 Hz Table Cache & Bounded Worker Channel**: Replaced the static table cache key with a 64-bit dynamic telemetry signature, switched the worker to `mpsc::sync_channel(2)`, and optimized `ProcessDeltaTracker` retention to `O(n)` via `HashSet`.
- **clippy.toml** (`msrv = "1.78.0"`).

## [0.0.13] - 2026-11-03

- **Win32 RAII SafeHandle & PID Safety Gate**: Added `SafeHandle` (`CloseHandle` on `Drop`), non-blocking elevation check (`is_app_elevated`), and `verify_pid_safety_gate` protecting critical Windows OS and anti-cheat processes (`src/modules/memory/safe_handle.rs`).
- **Deterministic Memory Actions Engine**: Added per-process working-set trim (`K32GetProcessMemoryInfo` before/after `K32EmptyWorkingSet`), batch `ReclaimReport` auditing, safe group termination, and file location launcher (`src/modules/memory/actions.rs`).

## [0.0.12] - 2026-11-03

- **RAM Composition Bar**: Added a 3-segment horizontal RAMMap-style bar (`In Use`, `Standby Cache`, `Free`) with hover breakdown tooltip (`src/modules/memory/composition_bar.rs`).
- **Real-Time Leak Hunter (`Δ RAM/s`)**: Added per-PID working-set delta tracking (`delta_bytes_per_sec`) and live `▲ +X.X M/s` / `▼ -X.X M/s` indicators in the process table.
- **1 Hz Table Cache & Minimize Edge Trim**: Cached filtered/sorted process rows at 1 Hz to eliminate 60 FPS re-sorting overhead, and restricted `K32EmptyWorkingSet` self-trim to startup and window minimize transitions.
- **Dependency & Documentation Cleanup**: Removed 7 unused crates from `Cargo.toml` (leaving 6 core crates), added the 85% chart threshold line, fixed sidebar card rendering, and introduced `tests/docs_guard.rs`.

