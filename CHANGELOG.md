# Changelog

All notable changes to **PULSE** (`pulse_manager`) are documented in this file.
Binary assets and checksums are published on [GitHub Releases](https://github.com/pete702/pulse-task-manager/releases).

## [0.0.13] - 2026-11-03

- **Win32 RAII SafeHandle & PID Safety Gate**: Added `SafeHandle` (`CloseHandle` on `Drop`), non-blocking elevation check (`is_app_elevated`), and `verify_pid_safety_gate` protecting critical Windows OS and anti-cheat processes (`src/modules/memory/safe_handle.rs`).
- **Deterministic Memory Actions Engine**: Added per-process working-set trim (`K32GetProcessMemoryInfo` before/after `K32EmptyWorkingSet`), batch `ReclaimReport` auditing, safe group termination, and file location launcher (`src/modules/memory/actions.rs`).

## [0.0.12] - 2026-11-03

- **RAM Composition Bar**: Added a 3-segment horizontal RAMMap-style bar (`In Use`, `Standby Cache`, `Free`) with hover breakdown tooltip (`src/modules/memory/composition_bar.rs`).
- **Real-Time Leak Hunter (`Δ RAM/s`)**: Added per-PID working-set delta tracking (`delta_bytes_per_sec`) and live `▲ +X.X M/s` / `▼ -X.X M/s` indicators in the process table.
- **1 Hz Table Cache & Minimize Edge Trim**: Cached filtered/sorted process rows at 1 Hz to eliminate 60 FPS re-sorting overhead, and restricted `K32EmptyWorkingSet` self-trim to startup and window minimize transitions.
- **Dependency & Documentation Cleanup**: Removed 7 unused crates from `Cargo.toml` (leaving 6 core crates), added the 85% chart threshold line, fixed sidebar card rendering, and introduced `tests/docs_guard.rs`.

## [0.0.11] - 2026-11-02

- **OS Jitter Probe & Workload Mode**: Added 1 Hz timer sleep-overshoot jitter tracking (`Δ ms`), workload classification (`Idle` / `Everyday` / `Heavy`), and viewport hibernation when minimized.
- **Keyboard Shortcuts & Clone Grouping**: Added `Ctrl+F` search focus, `Esc` filter clear, `[Nx]` process clone aggregation, and geometric `▲ HOG` badges.
- **Windows x64 CFG Normalization**: Normalized `>= 1 TiB` Control Flow Guard virtual memory reservations on 64-bit Windows 11 system processes.

## [0.0.10] - 2026-11-02

- **Strict 180-Line Modularity Refactoring**: Decomposed the codebase into 50 focused `.rs` modules under a hard `MAX_RS_FILE_LINES = 180` ceiling enforced by `src/core/selftest_tests.rs`.
- **Submodule Test Architecture**: Extracted unit tests into sibling `*_tests.rs` files and enforced the International English (`0` Cyrillic) repository policy with a native Win32 OS locale badge (`GetUserDefaultUILanguage`).

## [0.0.9] - 2026-11-02

- **Live GUI `--self-test` & Top Process Analytics**: Upgraded `--self-test` to verify live frame rendering and added Top-5 RAM consumer cards alongside `1m / 5m / 15m` history chart scaling.

## [0.0.8] - 2026-11-02

- **Win32 Kernel Telemetry & Virtualized Table**: Integrated `K32GetPerformanceInfo` for Paged/Non-Paged Pool and Standby Cache metrics, virtualized `TableBuilder` process rows, and the Memory Spike Blackbox detector.