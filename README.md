# PULSE -- Windows System, Storage, Network & Process Telemetry HUD

A native Windows 10/11 task manager in pure Rust + `egui` for real-time CPU, RAM composition, Physical Disk I/O, Network throughput + TCP Ping/Jitter radar, and process management.

[Download Latest Portable .exe (Windows 10/11 x64)](https://github.com/pete702/pulse-task-manager/releases/latest)

[![CI](https://github.com/pete702/pulse-task-manager/actions/workflows/ci.yml/badge.svg)](https://github.com/pete702/pulse-task-manager/actions)
[![License: Proprietary -- All Rights Reserved](https://img.shields.io/badge/License-Proprietary--All_Rights_Reserved-red.svg)](LICENSE)
[![Platform: Windows 10/11 x64](https://img.shields.io/badge/Platform-Windows%2010%2F11%20x64-0078D4.svg)](https://github.com/pete702/pulse-task-manager/releases)

**Single-file portable widget-free HUD - no service, no installer.**

## Why PULSE?

Standard Windows Task Manager shows total RAM usage, but rarely tells you **which process** caused a sudden memory jump or how much of your used memory is actually Windows file and system cache.

PULSE provides a dark-mode telemetry HUD with a **Memory Spike Blackbox**, a **3-Segment RAMMap Composition Bar**, and a **Real-Time Leak Hunter (Delta RAM/s, dt-normalized)**.

## Proprietary & Privacy

Proprietary core forever ([01]-[05] telemetry, PID Safety Gate, zero network).
Future Pro extensions (24h history, automation rules) will be optional offline modules with zero telemetry.

## Layout

- **`[01] CPU` -- Processor & Processes Cockpit**: per-core load grid, 1m/5m/15m dual-series chart, single-core bottleneck alert, virtualized process table (single-source 7-column layout `col_x/col_w`, draggable dividers at `col_x[i+1]`, compact CLONES, proportional scaling, clipped RAM bar, dynamic PROCESS NAME expansion, CPU %, RAM, Disk I/O, clone grouping `[Nx]`, Win32 EcoQoS toggle, Standby/Working Set purge, End Task, `Ctrl+F` search) with responsive 2-tier toolbar (single row >= 1180px, two rows below; right controls reserved).
- **`[02] Memory` -- RAM Composition & Leak Hunter**: Physical RAM 3-segment bar (In Use / Standby Cache / Free), Commit Charge & Kernel Pools (Paged / Non-Paged), 1m/5m/15m utilization chart, Memory Spike Blackbox, and per-process Working Set / Peak / Delta (`MB/s`) table with safe RAM Trim.
- **`[03] Storage` -- Physical Disk I/O & Freeze Hunter**: `\\.\PhysicalDriveN` `IOCTL_DISK_PERFORMANCE` counters plus `IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS` fallback, deduplicated Read/Write MB/s chart (`1m/5m/15m`) with non-overlapping header and decimal Y labels, I/O Freeze Spike blackbox, master-detail cockpit, aligned single-baseline KPIs, ultra-compact disk cards with bar-matched percentages, fixed-width Top-8 activity table, and clamped Read/Write hover tooltips.
- **`[04] Network` -- Adapter Throughput & Ping/Jitter Radar**: Physical NIC filtering, live RX/TX Mbps and Latency/Jitter (`ms`) charts (`1m/5m/15m`) with 9-step decade auto-scaling, adaptive 1 Hz / 5 s non-blocking TCP RTT probe, connection grade (`OPTIMAL` / `DEGRADED` / `CRITICAL`), and per-adapter traffic scope.
- **`[05] System & GPU` -- Hardware Topology, DXGI GPU VRAM & Diagnostic Report (`Planned v0.5.0`)**.

## Feature Status

| Subsystem / Feature | Status | Details |
| :--- | :--- | :--- |
| **Physical RAM & Swap/Pagefile** | **Implemented [OK]** | Live usage bars, commit charge, and pressure verdicts |
| **Win32 Kernel Pool Telemetry** | **Implemented [OK]** | Paged Pool, Non-Paged Pool, System Cache via `K32GetPerformanceInfo` |
| **3-Segment RAM Composition Bar** | **Implemented [OK]** | Visual breakdown of `In Use`, `Standby Cache`, and `Free` RAM |
| **Virtualized Process Table** | **Implemented [OK]** | `Ctrl+F` search, `Esc` clear, `[Nx]` clone grouping, `HOG` badges |
| **Real-Time Leak Hunter (Delta RAM/s)** | **Implemented [OK]** | Per-process working-set delta normalized by real sample `dt` |
| **Memory Spike Blackbox & Chart** | **Implemented [OK]** | Records culprit PID/process on RAM jumps; `1m/5m/15m` Commit % chart |
| **1-Click Safe RAM Purge & Actions** | **Implemented [OK]** | `Trim Working Sets`, `Trim Selected`, `Open Location`, `End Task`, PID Safety Gate |
| **Processor (CPU) & EcoQoS** | **Implemented [OK] (`v0.2.0`)** | Per-core grid, single-core bottleneck detector, Win32 EcoQoS toggle |
| **Storage & Disk I/O Freeze Hunter** | **Implemented [OK] (`v0.3.0`)** | Volume capacity bars and per-process read/write MB/s table |
| **Network & Ping/Jitter Radar** | **Implemented [OK] (`v0.4.0`)** | Adapter throughput and non-blocking TCP latency/jitter probe |
| **DXGI Multi-GPU & System Report** | *Planned (`v0.5.0`)* | Dedicated/Shared VRAM via `IDXGIAdapter3` and sanitized Markdown report |

## Technical Characteristics

- **Single Portable Executable**: Standalone `.exe` (`~3.7 MiB` binary size on disk), no installer, no background service.
- **Live System Top Bar**: Top header shows `[ CPU | RAM | UPTIME | HEALTH ]` plus true Win32 `GetUserDefaultLocaleName` OS locale; PULSE self-RAM/UI diagnostics live in the status bar.
- **State Persistence & Tactical HUD**: Tab, sort, grid view, and window size survive restarts via `%LOCALAPPDATA%\PULSE\state.cfg`; next planned is Total Sci-Fi Cockpit UI 2.0 Redesign after `[05]`.
- **1 Hz Pausable Telemetry Worker**: Main `egui` UI thread + background telemetry worker thread (`bounded std::sync::mpsc::sync_channel(2)`) with 1 Hz system telemetry polling and adaptive process scanning (3 s when Processes tab active, 5 s on Overview). `Space` pause/resume freezes worker collection without dropping history; `Ctrl+F` search; `Esc` clear.
- **Interned `Arc<str>` Process Names**: `util::intern_name` shares one `Arc<str>` per process name across CPU and Memory collectors, capped at 512 entries.
- **Cached Physical Disk Handles**: Physical disk handles cached in `OnceLock` map (max 32), evicted and closed on `DeviceIoControl` failure -- no more per-tick `CreateFileW`/`CloseHandle`.
- **Minimize Hibernation**: 0 FPS UI when minimized (5s background worker poll + 30s safety watchdog); UI rendering pauses while minimized and triggers a single controlled working-set self-trim (`K32EmptyWorkingSet`) on frame #3 after GPU init and on minimize transition.
- **User-Mode Only**: Uses standard Win32 APIs (`K32GetPerformanceInfo`, `sysinfo`); no kernel drivers, no DLL injection, and no hooks into other processes.
- **PID Safety Gate**: Blocks PID 0, PID 4, self-PID, 31 critical OS processes, and 11 known anti-cheat processes before any mutating action.
- **3-Point Process Identity**: `basename + creation_time + full_path` verification before any mutating action; frozen target list on `[ End Task ]` confirmation.
- **Three Walls Badge**: Live `SYSTEM OPTIMAL` / `RAM ELEVATED` / `MEMORY WALL` indicator driven by RAM and Commit pressure.
- **Cross-Target CI**: Linux Clippy clean with `HANDLE` as `isize` on all targets, `cfg`-gated imports in `qos_action.rs`, `physical_disk.rs`, `safe_handle.rs`, `actions.rs`, fail-closed stubs for non-Windows.

## Keyboard Shortcuts

- `Space` -- Pause / resume telemetry worker
- `Ctrl + F` -- Focus process search filter
- `Esc` -- Clear search filter and reset focus

## Build from Source

Requires Rust `1.88+` on Windows 10/11 (x64 MSVC toolchain):

```powershell
git clone https://github.com/pete702/pulse-task-manager.git
cd pulse-task-manager
cargo run --release
```

## CLI Flags

```powershell
pulse_manager.exe --version   # Prints: PULSE v0.4.47 (pulse_manager)
pulse_manager.exe --help      # Prints usage
pulse_manager.exe --self-test # Headless CI self-test
```

## Documentation & License

For developers: [Architecture & Engineering Rules](ARCHITECTURE.md) - [Changelog](CHANGELOG.md)

Proprietary -- Copyright (c) 2026 Petro (pete702). All Rights Reserved.
All source code and architecture belong exclusively to Petro (pete702). Copying, modifying, redistributing, or reusing the source code or binaries in any form without explicit written permission from the author is strictly prohibited. Free personal non-commercial use of official unmodified binaries only. See LICENSE for full terms.
