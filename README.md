# PULSE -- Windows System, Storage, Network & Process Telemetry HUD

[![CI](https://github.com/pete702/pulse-task-manager/actions/workflows/ci.yml/badge.svg)](https://github.com/pete702/pulse-task-manager/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Platform: Windows 10/11 x64](https://img.shields.io/badge/Platform-Windows%2010%2F11%20x64-0078D4.svg)](https://github.com/pete702/pulse-task-manager/releases)

A native Windows 10/11 task manager in Rust + egui that stays responsive even when the built-in Task Manager lags. Single portable executable, about < 5 MB.

[Download Latest Portable .exe (Windows 10/11 x64)](https://github.com/pete702/pulse-task-manager/releases/latest)

<!-- ![PULSE Preview](docs/preview.png) -->

## Why PULSE?

- **User-mode only.** Standard Win32 APIs, no kernel drivers, no injection, no hooks. Safe to run alongside anti-cheat software.
- **Group Clone Fleet Management.** Collapse duplicate processes into one row with `[Nx]` badges, then trim, eco-mode, or end the whole group at once.
- **RAM Composition.** See In Use, Standby Cache, and Free RAM plus Commit Charge and Kernel Pools (paged / non-paged).
- **Physical Disk & Top-8 I/O.** Per-disk read/write rates and the top consumers, with spike detection.
- **Network Ping/Jitter Grade.** Live throughput plus an OPTIMAL / DEGRADED / CRITICAL connection grade.
- **3-Point Identity Safety Gate.** Basename, creation time, and full path verified before any trim or kill. Frozen target lists on confirmation.
- **Minimize Hibernation & Startup Trim.** Zero FPS when minimized; trims its own working set once at startup and on minimize.

## Comparison

| Feature | PULSE | Windows 11 Task Manager | System Informer |
| :--- | :--- | :--- | :--- |
| Portable single exe (< 5 MB) | Yes | No | No |
| Clone grouping with fleet actions | Yes | No | Partial |
| RAM composition + kernel pools | Yes | Partial | Yes |
| Network ping/jitter grade | Yes | No | No |

## Tabs

- `[00] Processes` -- unified process table, search, clone grouping, EcoQoS, trim, end task
- `[01] Memory (RAM)` -- composition bar, commit charge, kernel pools, history chart
- `[02] Processor (CPU)` -- per-core grid, bottleneck alert, process CPU table
- `[03] Storage` -- physical disks, I/O rates, freeze spike blackbox
- `[04] Network` -- adapter throughput, ping/jitter radar, connection grade
- `[05] System & GPU` -- planned for v0.5.0

## Hotkeys

- `Ctrl+1..6` -- switch tabs
- `Ctrl+F` -- focus process search
- `F5` -- force process scan
- `F10` -- pause/resume telemetry
- `Esc` -- clear search or selection
- `Space` -- pause collection (on table)

## Build from Source

Requires Rust 1.88+ on Windows 10/11 (x64 MSVC toolchain):

```powershell
git clone https://github.com/pete702/pulse-task-manager.git
cd pulse-task-manager
cargo build --release --locked
```

## CLI Flags

```powershell
pulse_manager.exe --version   # Prints: PULSE v0.5.25 (pulse_manager)
pulse_manager.exe --help      # Prints usage
pulse_manager.exe --self-test # Headless CI self-test
pulse_manager.exe --lang=en   # Force English UI
```

## Documentation & License

For developers: [Architecture & Engineering Rules](ARCHITECTURE.md).

Licensed under the [MIT License](LICENSE). Copyright (c) 2026 Petro (pete702).