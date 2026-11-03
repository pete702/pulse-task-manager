# ⚡ PULSE — Featherweight Task Manager & Hardware HUD for Windows

![Version](https://img.shields.io/badge/version-v0.0.11-00F0FF)
![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11%20(x64)-1F2330)
![Engine](https://img.shields.io/badge/engine-Rust%20%2B%20egui-00F0FF)
![Safety](https://img.shields.io/badge/safety-100%25%20Anti--Cheat%20Safe-00F0FF)

> **Built for Gamers, Video Editors, and Power Users who want instant hardware clarity without background bloat.**

![PULSE Obsidian HUD](screenshot.png)

---

## ✨ Why PULSE?

| Feature | What It Means for You |
|---------|----------------------|
| **Featherweight by Design** | Single portable `.exe`. Runs on strictly **2 threads** (no heavy thread pools) with **0% GPU/CPU usage when minimized**, leaving 100% of your PC's power for your games and renders. |
| **100% Anti-Cheat Safe** | Zero DLL injection, zero DirectX hooks, and zero kernel drivers. Completely safe to run alongside **Vanguard, EasyAntiCheat, BattlEye, and Ricochet**. |
| **Catches What Task Manager Misses** | Built-in **Memory Spike Blackbox** records the exact process that caused a sudden RAM surge, while the **System Jitter Probe** detects Windows kernel micro-stutters in real time. |
| **Automatic Workload Detection** | Automatically detects whether your PC is in **Desktop Idle**, **Workstation Load**, or **Gaming / Heavy App** mode (e.g., DaVinci Resolve or AAA games). |
| **Clean "Obsidian HUD" Interface** | High-contrast dark theme with traffic-light load indicators (Cyan / Amber / Crimson) and instant process clone grouping (`chrome.exe [33x]`). |

---

## 📊 PULSE vs. Traditional Monitors

| Capability | Windows Task Manager | Heavy Gaming Overlays | **PULSE** |
|------------|---------------------|----------------------|-----------|
| Background Footprint | ~30–50 MB, 4+ threads | ~100–200 MB, hooks into games | **~4.4 MB, 2 threads** |
| Anti-Cheat Safety (Zero DLL Hooks) | ✅ Native | ❌ Often flagged | ✅ **Zero hooks, 100% safe** |
| Process Clone Grouping `[Nx]` | ❌ | ❌ | ✅ **Instant grouping** |
| Memory Spike Blackbox & OS Jitter Probe | ❌ | ❌ | ✅ **Built-in** |
| Portable Single `.exe` (No Installer) | ❌ System app | ❌ Installer required | ✅ **Portable, no install** |

---

## 🚀 Quick Start

### Download & Run (Recommended)
1. Go to **[Releases](https://github.com/pulse-manager/pulse_manager/releases)**
2. Download `PULSE_v0.0.11_modularity_memory.exe` (or `PULSE_latest.exe`)
3. Double-click — no installation needed

### From Source
```bash
git clone https://github.com/pulse-manager/pulse_manager
cd pulse_manager
cargo run --release
```

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl + F` | Instant search by Process Name or PID (switches to Memory → Processes) |
| `Esc` | Clear search filter |

---

## 🗺️ Product Roadmap

| Phase | Target | Focus |
|-------|--------|-------|
| **1** | **v0.1.0** | Memory Telemetry, Spike Blackbox & **1-Click Safe RAM Purge** *(Current — v0.0.11)* |
| **2** | v0.2.0 | CPU Per-Core Heatmap, Single-Thread Bottleneck Detector & EcoQoS Mode |
| **3** | v0.3.0 | Storage & Disk I/O Freeze Hunter |
| **4** | v0.4.0 | Network Throughput & Esports Ping/Jitter Radar |
| **5** | v0.5.0 | Multi-GPU VRAM Telemetry & 1-Click Sanitized System Report |
| **6** | **v1.0.0** | Always-On-Top Tactical Mini-HUD (F10) & External Language Packs |

---

## 🛠️ For Developers

- **Architecture:** [ARCHITECTURE.md](ARCHITECTURE.md) — module tree, 2-thread design, safety gates, NVIDIA Agent Skills patterns
- **Changelog:** [CHANGELOG.md](CHANGELOG.md) — detailed version history
- **Core Crates:** `eframe` 0.27.2, `egui` 0.27.2, `egui_plot` 0.27.2, `egui_extras` 0.27.2, `sysinfo` 0.30.12 (`default-features = false`), `windows-sys` 0.52.0
- **Build:** `cargo build --release` (thin LTO, strip=symbols, panic=abort)
- **Tests:** `cargo test` (72+ unit tests, architectural guards for file size & zero Cyrillic)

---

## 📜 License

MIT License — see [LICENSE](LICENSE) for details.