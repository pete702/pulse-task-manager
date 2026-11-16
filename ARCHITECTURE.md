# PULSE Architecture & Engineering Rules

PULSE (`pulse_manager`) is a native Windows 10/11 hardware telemetry HUD and process manager written in Rust (`Edition 2021`, `rust-version = "1.88"`) using `eframe` / `egui` (`0.27.2`). Current release: **v0.4.137**.

## 1. Core Architectural Principles

1. **Decoupled Two-Thread Telemetry Model**:
   - **UI Thread** (`src/app.rs`, `src/ui/`): renders egui widgets, handles input, never calls blocking Win32/sysinfo. Holds `Arc<SystemSnapshot>` for zero-copy per-frame access.
   - **Worker Thread** (`src/core/worker.rs`, `worker_loop.rs`): RAM at 1 Hz (`snapshot_id`), process scans every 3 s fast / 5 s background (`process_scan_id`).
2. **Live UI Structure**: Processes, Memory, Processor, Storage, Network, System & GPU.
3. **Startup Trim & Minimize Hibernation** (`app_trim.rs`): one safe self-trim ~5 s after startup; 0 FPS when minimized.
4. **Non-Blocking Actions & Cached Elevation**: Open Location on background thread; `is_app_elevated()` cached via `OnceLock`.

## 2. Hard Modularity & Hygiene Guards

- **Source File Limit** (`MAX_RS_FILE_LINES = 280`): enforced by `selftest_tests.rs` and `docs_guard.rs`.
- **Submodule Test Architecture**: unit tests in sibling `*_tests.rs`.
- **Language & Documentation Guards**: zero Cyrillic, zero `Lang::Uk`; docs at most 120 lines / 8192 bytes, strict ASCII.
- **Persisted State** (`persist.rs`): atomic write to `state.cfg.tmp` then rename.

## 3. Module Boundaries (`src/`)

- `src/core/`: worker, `SystemSnapshot` channel, state persistence, CLI self-test, guards.
- `src/modules/cpu/`: Processor cockpit, per-core grid, process table, EcoQoS, safety gate.
- `src/modules/memory/`: RAM/Swap collector, kernel metrics, spike blackbox, leak hunter.
- `src/modules/storage/`: physical disk I/O, volume handles, freeze spike blackbox.
- `src/modules/network/`: adapter throughput, TCP RTT probe, connection grade.
- `src/ui/`: root panel layout, theme, top bar, sidebar, status bar, `chart_kit`, `table_kit`.
- `src/i18n/`: English UI strings + Win32 OS locale name.
- `src/app_cmd.rs`: `AppCommand` enum for clean UI-to-state dispatch.

## 4. Shared Helpers

- `ring_history`: series caching and startup single-point guard in memory history.
- `chart_kit`: `find_nearest_point`, `find_nearest_index`, `baseline_y`, `chart_frame`.
- `table_kit`: usage bar, sort header, copy row, row highlight.

## 5. Win32 FFI & Safety Invariants

- `#![deny(unsafe_code)]` at crate root. Localized `#[allow(unsafe_code)]` for Win32 only.
- **PID Safety Gate**: blocks PID 0, 4, self, 31 critical OS processes, 11 anti-cheat.
- **3-Point Identity Verification**: basename + creation time + full path.
- **Win11 x64 CFG Normalization**: `virtual_memory() >= 1 TiB` normalized to working set.

## 6. Five Direct Dependencies (pinned exactly)

`eframe`, `egui`, `egui_plot`, `sysinfo`, `windows-sys`. No new crates. `Cargo.lock` committed. `rust-version = "1.88"`. MIT licensed.
