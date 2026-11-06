# ▲ PULSE — Windows Memory & Process Telemetry HUD

A native Windows 10/11 task manager in Rust + `egui` that shows RAM composition, real-time per-process memory growth (`Δ RAM/s`), and which process caused a sudden memory spike.

**[📥 Download Latest Portable .exe (Windows 10/11 x64)](https://github.com/pete702/pulse-task-manager/releases/latest)**

[![CI](https://github.com/pete702/pulse-task-manager/actions/workflows/ci.yml/badge.svg)](https://github.com/pete702/pulse-task-manager/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform: Windows 10/11 x64](https://img.shields.io/badge/Platform-Windows%2010%2F11%20x64-0078D4.svg)](https://github.com/pete702/pulse-task-manager/releases)

**Single-file portable widget-free HUD - no service, no installer.**

## Why PULSE?

Standard Windows Task Manager shows total RAM usage, but rarely tells you **which process** caused a sudden memory jump or how much of your used memory is actually Windows file and system cache.

PULSE provides a dark-mode telemetry HUD with a **Memory Spike Blackbox**, a **3-Segment RAMMap Composition Bar**, and a **Real-Time Leak Hunter (`Δ RAM/s`, dt-normalized)**.

## Open-Core & Privacy

Free MIT core forever ([00]-[05] telemetry, PID Safety Gate, zero network).
Future Pro extensions (24h history, automation rules) will be optional offline modules with zero telemetry.

## Layout

- **`[00] Processes` — Unified Task Manager Hub**: full-width virtualized process table with `CPU %`, `MEMORY (RAM)`, `10s DELTA`, clone grouping `[Nx]`, `[HOG]` badges, draggable column dividers, `Trim RAM`, `EcoQoS`, `Terminate`, `Ctrl+F` search, and instant 0ms in-place sorting.
- **`[01] Memory (RAM)` — Pure Hardware Cockpit**: two symmetric top cards (Physical RAM with 3-segment composition bar; Commit Charge & Kernel Pools with precision bar) plus a full-height Memory Utilization History chart, zero scrollbars.
- **`[02] Processor (CPU)` — Pure Hardware Cockpit**: Total Processor Load, Peak Thread Bottleneck, per-core grid, dual-series 1m/5m/15m telemetry chart, single-thread bottleneck alert.

## Feature Status

| Subsystem / Feature | Status | Details |
| :--- | :--- | :--- |
| **Physical RAM & Swap/Pagefile** | **Implemented ✅** | Live usage bars, commit charge, and pressure verdicts |
| **Win32 Kernel Pool Telemetry** | **Implemented ✅** | Paged Pool, Non-Paged Pool, System Cache via `K32GetPerformanceInfo` |
| **3-Segment RAM Composition Bar** | **Implemented ✅** | Visual breakdown of `In Use`, `System Cache`, and `Free` RAM |
| **Virtualized Process Table** | **Implemented ✅** | `Ctrl+F` search, `Esc` clear, `[Nx]` clone grouping, `▲ HOG` badges |
| **Real-Time Leak Hunter (`Δ RAM/s`)** | **Implemented ✅** | Per-process working-set delta normalized by real sample `dt` (`▲ +M/s` / `▼ -M/s`) |
| **Memory Spike Blackbox & Chart** | **Implemented ✅** | Records culprit PID/process on RAM jumps; `1m/5m/15m` Commit % chart with 85% line |
| **1-Click Safe RAM Purge & Actions** | **Implemented ✅** | Raw-process action targeting: `Trim Working Sets`, `Trim Selected`, `Open Location`, `End Task`, with PID Safety Gate and Before/After audit |
| **Processor (CPU) & EcoQoS** | **Implemented ✅ (`v0.2.0`)** | Per-core grid, single-core bottleneck detector, Win32 EcoQoS toggle |
| **Storage & Disk I/O Freeze Hunter** | *Planned (`v0.3.0`) 🔄* | Volume capacity bars and per-process read/write MB/s table |
| **Network & Ping/Jitter Radar** | *Planned (`v0.4.0`) 🔄* | Adapter throughput and non-blocking TCP latency/jitter probe |
| **DXGI Multi-GPU & System Report** | *Planned (`v0.5.0`) 🔄* | Dedicated/Shared VRAM via `IDXGIAdapter3` and sanitized Markdown report |

## Technical Characteristics

- **Single Portable Executable**: Standalone `.exe` (`~3.7 MiB` binary size on disk), no installer, no background service.
- **Live System Top Bar**: Top header shows `[ CPU | RAM | UPTIME | HEALTH ]` plus true Win32 `GetUserDefaultLocaleName` OS locale; PULSE self-RAM/UI diagnostics live in the status bar.
- **State Persistence & Tactical HUD**: Tab, sort, grid view, and window size survive restarts via `%LOCALAPPDATA%\PULSE\state.cfg`; corner brackets, ticked bars, and scanline mark the Sci-Fi cockpit; next planned is Total Sci-Fi Cockpit UI 2.0 Redesign after `[03]`-`[05]`.
- **Decoupled 1 Hz / Adaptive 3 s / 5 s Collector**: Main `egui` UI thread + background telemetry worker thread (`bounded std::sync::mpsc::sync_channel(2)`) with 1 Hz system telemetry polling and adaptive process scanning (3 s when Processes tab active, 5 s on Overview); paused process scanning with zero-copy `Arc` sharing in minimized mode (instant full scan on wake, tab switch, or user action); `O(1)` UI table & selection caching via `process_scan_id`.
- **Minimize Hibernation**: 0 FPS UI when minimized (5s background worker poll + 30s safety watchdog); UI rendering pauses while minimized and triggers a single controlled working-set self-trim (`K32EmptyWorkingSet`) on frame #3 after GPU init and on minimize transition.
- **User-Mode Only**: Uses standard Win32 APIs (`K32GetPerformanceInfo`, `sysinfo`); no kernel drivers, no DLL injection, and no hooks into other processes.
- **PID Safety Gate**: Blocks PID 0, PID 4, self-PID, 31 critical OS processes, and 11 known anti-cheat processes before any mutating action.
- **3-Point Process Identity**: `basename + creation_time + full_path` verification before any mutating action; frozen target list on `[ End Task ]` confirmation.
- **Three Walls Badge**: Live `SYSTEM OPTIMAL` / `RAM ELEVATED` / `MEMORY WALL` indicator driven by RAM and Commit pressure.
- **Global Compute Alert**: `[ ▲ COMPUTE BOTTLENECK ]` appears in amber in the top chrome when a core or total load reaches 85%, unless a memory wall alert is active.

## Keyboard Shortcuts

- `Ctrl + F` — Focus process search filter
- `Esc` — Clear search filter and reset focus

## Build from Source

Requires Rust `1.88+` on Windows 10/11 (x64 MSVC toolchain):

```powershell
git clone https://github.com/pete702/pulse-task-manager.git
cd pulse-task-manager
cargo run --release
```

## CLI Flags

```powershell
pulse_manager.exe --version   # Prints: PULSE v0.4.5 (pulse_manager)
pulse_manager.exe --help      # Prints usage
pulse_manager.exe --self-test # Headless CI self-test
```

## Documentation & License

For developers: [Architecture & Engineering Rules](ARCHITECTURE.md) · [Changelog](CHANGELOG.md)

Open-Source under the MIT License. Copyright (c) 2026 Petro (pete702) - PULSE Project.
