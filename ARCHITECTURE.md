# PULSE Architecture & Engineering Rules

PULSE (`pulse_manager`) is a native Windows 10/11 hardware telemetry HUD and process manager written in Rust (`Edition 2021`, `rust-version = "1.88"`) using `eframe` / `egui` (`0.27.2`). Current release: **v0.4.57**.

## 1. Core Architectural Principles

1. **Decoupled Two-Thread Telemetry Model**:
   - **UI Thread** (`src/app.rs`, `src/ui/`): Renders `egui` widgets, handles input, never calls blocking Win32/`sysinfo`. Holds `Arc<SystemSnapshot>` for zero-copy per-frame access. Minimized: 0 FPS UI when minimized (5s background worker poll + 30s safety watchdog).
   - **Worker Thread** (`src/core/worker.rs`): **RAM at 1 Hz** (`snapshot_id`), **process scans every 3 s visible / 5 s minimized** (`process_scan_id`). Minimized: scans pause, 5s watchdog poll, 30s UI safety repaint, `Arc<Vec<RawProcessRow>>` shared zero-copy; restore triggers wakeup; wake/user action triggers immediate full scan. `LatestSlot` with `AtomicU64` overflow tracking; `sync_channel(2)` full means UI reads newer slot. `Space` pauses collection via `WorkerConfig.paused`.

2. **O(1) Process Table & Selection Cache**:
   - Filtered/grouped/sorted rows cached in `MemoryTableState` keyed by `process_scan_id` (monotonic u64). Scan ID + filter/sort/group unchanged means rebuild in `O(1)`. Rows stored as `Arc<[ProcessMemoryItem]>` for 60 FPS frames zero clones. `SelectedGroupSummary` with `Vec<ProcessTarget>` also cached per `scan_id`.

3. **Viewport Hibernation & Single Frame-#3 Self-Trim**:
   - `K32EmptyWorkingSet` called **once on frame #3 after GPU init**, never again unless explicit. Minimize transition triggers trim via periodic guard only; `last_viewport_size` updates only when `vp` is `Some`.

4. **Non-Blocking Actions & Cached Elevation**:
   - `Open Location`: sync path validation, then `explorer.exe` on background thread.
   - `is_app_elevated()`: `OnceLock<bool>` for single Win32 query per process lifetime.

## 2. Hard Modularity & Hygiene Guards

- **Source File Limit** (`MAX_RS_FILE_LINES = 280`): Every `.rs` in `src/`/`tests/` at most 280 lines post `cargo fmt`, enforced by `selftest_tests.rs` and `docs_guard.rs`. Zero rustfmt skip attributes.
- **Submodule Test Architecture**: Unit tests in sibling `*_tests.rs` inside `src/`.
- **Language & Documentation Guards**:
  - `selftest_tests.rs`: scans `src/**/*.rs` for zero Cyrillic (`U+0400..=U+04FF`), zero `Lang::Uk`.
  - `docs_guard.rs`: `README.md`, `ARCHITECTURE.md`, `CHANGELOG.md` at most 120 lines / 8192 bytes, zero NUL, zero Cyrillic, strict ASCII for markdown docs, no internal paths/unverified claims.

## 3. Module Boundaries (`src/`)

- `src/core/`: Worker (`worker.rs`, `worker_loop.rs`, `worker_io.rs`, `worker_policy.rs`), `SystemSnapshot` channel (`mod.rs`), state persistence (`persist.rs`, `%LOCALAPPDATA%\PULSE\state.cfg` zero-crate), CLI `--self-test` (`selftest.rs`, `selftest_app.rs`), guards (`selftest_tests.rs`).
- `src/modules/cpu/`: Implemented `[01] CPU` - Processor cockpit, per-core grid, dual-series `1m/5m/15m` chart, single-core bottleneck alert, virtualized process table with single-source 7-column engine (`table_cols.rs` `col_x/col_w`, dividers at `col_x[i+1]`, proportional resize, clipped RAM bar, dynamic PROCESS NAME expansion, compact CLONES), responsive 2-tier toolbar (single row >= 1180px, 2 rows below), `CPU %`, `RAM`, `Disk I/O`, `[Nx]` clone grouping, EcoQoS toggle, `Ctrl+F` search. Interned `Arc<str>` names via `util::intern_name`.
- `src/modules/memory/`: Implemented `[02] Memory` - RAM/Swap collector, `K32GetPerformanceInfo` FFI, 3-segment bar (In Use / Standby Cache / Free), Commit Charge & Kernel Pools, spike blackbox, `Delta RAM/s` leak hunter, virtualized table, PID Safety Gate (`safe_handle.rs`). Cache key: `(process_scan_id, ram_total_bytes, search_query, sort_col, sort_asc, group_clones)`; `1m/5m/15m` chart with cached `PlotPoints`.
- `src/modules/storage/`: Implemented `[03] Storage` - `\\.\PhysicalDriveN` `IOCTL_DISK_PERFORMANCE` plus `IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS` fallback, cached volume handles in `OnceLock` map (max 32), per-mount hardware rates for unmapped volumes, deduplicated Read/Write `MB/s` chart (`1m/5m/15m`) with non-overlapping header, decimal Y labels, and clamped tooltips, I/O Freeze Spike blackbox, master-detail cockpit (280px full-height master, aligned KPI baselines, ultra-compact cards, detail KPI/chart/activity).
- `src/modules/network/`: Implemented `[04] Network` - Physical NIC filtering, live RX/TX `Mbps` and Latency/Jitter (`ms`) charts with 9-step decade auto-scaling (`nice_ceiling`), adaptive 1 Hz / 5 s non-blocking TCP RTT probe, connection grade (`OPTIMAL` / `DEGRADED` / `CRITICAL`), per-adapter scope.
- `src/ui/`: Root panel layout (`Top` to `Bottom` to `Left` to `Central`), theme, live System Telemetry TopBar (`[ CPU | RAM | UPTIME | HEALTH ]`), sidebar, status bar, Three Walls badge (`three_walls.rs`), HUD accents (`draw_cyber_corner_brackets`, `draw_tactical_ticks`, scanline).
- `src/i18n/`: English UI strings + Win32 OS locale name (`GetUserDefaultLocaleName`).

## 4. Win32 FFI & Safety Invariants

- `#![deny(unsafe_code)]` at crate root. Localized `#[allow(unsafe_code)]` blocks for: `K32GetPerformanceInfo`, `K32GetProcessMemoryInfo`, `K32EmptyWorkingSet`, `OpenProcess`, `IsProcessCritical`, `QueryFullProcessImageNameW`, `OpenProcessToken`, `GetTokenInformation`, `CloseHandle`, `TerminateProcess`, `GetUserDefaultLocaleName`, `AttachConsole` (verified v0.1.1).
- Every FFI block isolated in helper functions with return-code checks, `std::mem::zeroed()` init. Cross-target: `HANDLE` is `isize` on Windows and non-Windows, Windows-only imports `cfg`-gated, fail-closed stubs keep Linux Clippy clean.
- **TOCTOU Handle Identity**: `trim_single_pid` / `terminate_process_group` enforce `IsProcessCritical` + `QueryFullProcessImageNameW` via `verify_handle_identity` before mutation. Fail closed.
- **Win11 x64 CFG Normalization**: `virtual_memory() >= 1 TiB` normalized to working set for realistic commit on grouped system services.

## 5. Three-Point Identity & Immutable Kill Confirmation

- **3-Point Verification** (`safe_handle.rs`, `process_target.rs`):
  1. **Basename** -- `QueryFullProcessImageNameW` stripped to filename.
  2. **Creation Time** -- `GetProcessTimes` cached per `(pid, sysinfo_start_time)` in `ProcessDeltaTracker`.
  3. **Full Path** -- `QueryFullProcessImageNameW` full path vs `ProcessTarget.exe_path`.
  All three must match frozen `ProcessTarget`; any failure fails closed.
- **Immutable `PendingKillConfirm`** (`action_bar.rs`): First `[ End Task ]` freezes `Vec<ProcessTarget>` to `PendingKillConfirm { group_key, targets, armed_at }`. Second click (3 s) sends frozen list. Same-name spawn after arming excluded.

## 6. Obsidian Theme Tokens (`src/theme.rs`)

| Token | Hex | Purpose |
| :--- | :--- | :--- |
| `BG_WINDOW` | `#0B0C10` | Main window background |
| `BG_CARD` | `#12141C` | Card/panel background |
| `ACCENT_CYAN` | `#00F0FF` | Primary accent, active tab, optimal load (`< 60%`) |
| `ACCENT_AMBER` | `#FFB800` | Elevated load (`60-85%`), moderate growth |
| `ACCENT_RED` | `#FF2A54` | Critical pressure (`>= 85%`), `HOG`, spike alerts |
