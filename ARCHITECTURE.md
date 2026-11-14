# PULSE Architecture & Engineering Rules

PULSE (`pulse_manager`) is a native Windows 10/11 hardware telemetry HUD and process manager written in Rust (`Edition 2021`, `rust-version = "1.88"`) using `eframe` / `egui` (`0.27.2`). Current release: **v0.4.120**.

## 1. Core Architectural Principles

1. **Decoupled Two-Thread Telemetry Model**:
   - **UI Thread** (`src/app.rs`, `src/ui/`): renders egui widgets, handles input, never calls blocking Win32/sysinfo. Holds `Arc<SystemSnapshot>` for zero-copy per-frame access. Minimized: 0 FPS UI (5s background worker poll + 30s safety watchdog).
   - **Worker Thread** (`src/core/worker.rs`, `worker_loop.rs`): RAM at 1 Hz (`snapshot_id`), process scans every 3 s fast / 5 s background (`process_scan_id`). `LatestSlot` with `AtomicU64` overflow tracking; `sync_channel(2)` full means UI reads newer slot. `Space` pauses collection via worker pause state. `F10` pauses global telemetry.

2. **Live UI Structure**:
   - `[00] Processes`: unified process table (`cpu/table.rs`), search, clone grouping, EcoQoS, trim, end task.
   - `[01] Memory (RAM)`: composition bar, commit charge, kernel pools, history chart.
   - `[02] Processor (CPU)`: per-core grid, bottleneck alert, process CPU table.
   - `[03] Storage`: physical disks, I/O rates, freeze spike blackbox.
   - `[04] Network`: adapter throughput, ping/jitter radar, connection grade.
   - `[05] System & GPU`: planned for v0.5.0.

3. **Startup Trim & Minimize Hibernation** (`app_trim.rs`):
   - One safe `K32EmptyWorkingSet` self-trim ~5 seconds after startup (once per session).
   - Trim on minimize transition; 0 FPS UI when minimized.

4. **Non-Blocking Actions & Cached Elevation**:
   - Open Location: sync path validation, then `explorer.exe` on background thread.
   - `is_app_elevated()`: `OnceLock<bool>` for single Win32 query per process lifetime.

## 2. Hard Modularity & Hygiene Guards

- **Source File Limit** (`MAX_RS_FILE_LINES = 280`): enforced by `selftest_tests.rs` and `docs_guard.rs`. No skip attributes allowed.
- **Submodule Test Architecture**: unit tests in sibling `*_tests.rs` inside `src/`.
- **Language & Documentation Guards**:
  - `selftest_tests.rs`: zero Cyrillic (`U+0400..=U+04FF`), zero `Lang::Uk`.
  - `docs_guard.rs`: `README.md`, `ARCHITECTURE.md`, `CHANGELOG.md` at most 120 lines / 8192 bytes, strict ASCII, no internal paths.
- **Persisted State** (`persist.rs`): atomic write to `state.cfg.tmp` then rename; symlink guards; error signaling via `config_write_failed()` flag.

## 3. Module Boundaries (`src/`)

- `src/core/`: worker, `SystemSnapshot` channel, state persistence, CLI self-test, guards.
- `src/modules/cpu/`: Processor cockpit, per-core grid, process table, EcoQoS, safety gate.
- `src/modules/memory/`: RAM/Swap collector, kernel metrics, spike blackbox, leak hunter, PID safety gate (`safe_handle.rs`), location open.
- `src/modules/storage/`: physical disk I/O, volume handles, freeze spike blackbox.
- `src/modules/network/`: adapter throughput, TCP RTT probe, connection grade.
- `src/ui/`: root panel layout, theme, top bar, sidebar, status bar.
- `src/i18n/`: English UI strings + Win32 OS locale name.

## 4. Win32 FFI & Safety Invariants

- `#![deny(unsafe_code)]` at crate root. Localized `#[allow(unsafe_code)]` for Win32 only.
- **PID Safety Gate**: blocks PID 0, 4, self, 31 critical OS processes, 11 anti-cheat before any mutating action.
- **3-Point Identity Verification** (`safe_handle.rs`, `process_target.rs`): basename + creation time + full path. Frozen `GroupTarget` lists on kill confirmation. TOCTOU-safe `Open Location` (live handle/path verification).
- **Win11 x64 CFG Normalization**: `virtual_memory() >= 1 TiB` normalized to working set.

## 5. Six Direct Dependencies (pinned exactly)

`eframe`, `egui`, `egui_plot`, `egui_extras`, `sysinfo`, `windows-sys`. No new crates. `Cargo.lock` committed. `rust-version = "1.88"`. MIT licensed. All source under `src/` and `tests/`.