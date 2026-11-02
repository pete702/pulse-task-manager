# ⚡ PULSE — Ultra-Fast Modular Task Manager & System Monitor for Windows

> **Next-Gen System Telemetry for Windows** — Built in Rust + egui with an **Obsidian HUD** aesthetic.

---

## 📖 Description

**PULSE** is a modern, high-performance system monitor and task manager for Windows, written in **Rust** using **eframe/egui**. The project is built on three pillars:

- **Strict Modularity** — No file exceeds 150 lines. Every module follows `model.rs` (data + pure logic + tests), `collector.rs` (OS data collection), `view.rs` (UI).
- **Zero-Cost Abstractions** — Pinned dependency versions for 100% API compatibility. Release profile uses LTO, strip, panic=abort.
- **Reference-Grade "Obsidian HUD" UI** — Dark theme (`#0B0C10` window, `#12141C` cards, `#1F2330` borders, 6px radius). All numbers (MB, GB, %, PID) rendered in monospace. Traffic-light load colors: `<60%` cyan (`#00F0FF`), `60–85%` amber (`#FFB800`), `≥85%` neon red (`#FF2A54`).

---

## ✨ Key Features (Roadmap)

| Phase | Version | Feature | Status |
|-------|---------|---------|--------|
| **Foundation** | v0.0.1–v0.0.2 | Project scaffold, Git, Obsidian HUD theme, background telemetry worker (500ms uptime), modular app structure, 13 unit tests | ✅ **Done** |
| **Memory — Model** | v0.0.3 | `PhysicalMemory`, `SwapMemory`, `CacheCleaner` structs with unit tests | 🔄 **Next** |
| **Memory — Collector** | v0.0.4 | `sysinfo` RAM/Swap collection, `EmptyWorkingSet` via `windows-sys` | ⏳ Planned |
| **Memory — View** | v0.0.5 | RAM/Swap cards, progress bars with traffic-light colors, `egui_plot` history graphs, "Clear Cache" button | ⏳ Planned |
| **Processes — Model** | v0.0.6 | `ProcessEntry`, `GroupKey`, `ProcessGroup` aggregation | ⏳ Planned |
| **Processes — Collector** | v0.0.7 | `sysinfo::Processes` + Windows enrichment (exe path, session, cmdline) | ⏳ Planned |
| **Processes — View** | v0.0.8 | `egui_extras::TableBuilder` with monospace PID, grouping, search/filter | ⏳ Planned |
| **Memory Release** | **v0.1.0** | **Full RAM/Swap monitoring + one-click cache cleanup working end-to-end** | 🎯 **Target** |
| **System Module** | v0.2.0–v0.3.0 | CPU/GPU/Disk/Network overview panels | ⏳ Planned |
| **Polish & v1.0.0** | v0.9.0–v1.0.0 | Settings, autostart, CI/CD, production release | ⏳ Planned |

---

## 🏗️ Architecture

```
pulse_manager/
├── src/
│   ├── main.rs              # Entry point: eframe::run_native (≤35 lines)
│   ├── app.rs               # PulseApp: state, UI panels, snapshot handling
│   ├── theme.rs             # Obsidian HUD tokens, apply_obsidian_theme, load_color, card_frame
│   ├── core/
│   │   ├── mod.rs           # SystemSnapshot, mpsc channel types, spawn_telemetry_worker
│   │   └── worker.rs        # Background thread: 500ms interval, System::uptime() → channel
│   └── modules/             # Future: memory/, processes/, system/ (model/collector/view each)
└── tests/                   # Integration tests (planned)
```

### Core Principles

- **Module Boundaries** — Max 150 lines/file. Strict `model/collector/view` separation.
- **Documentation** — All public API documented in English (`///`).
- **Pinned Dependencies** — Exact versions in `Cargo.toml` (eframe 0.27.2, sysinfo 0.30.12, windows-sys 0.52.0).
- **Design Tokens** — Single source of truth in `theme.rs::colors`.
- **Monospace Numbers** — All numeric values use `.monospace()`.
- **Traffic-Light Colors** — `load_color(percent)` for all load indicators.
- **Non-Blocking UI** — Background worker sends via `mpsc`; UI reads via `try_recv()`.

---

## 🚀 Current Status: v0.0.2

**Implemented:**
- ✅ Obsidian HUD theme system (`theme.rs`) with design tokens, traffic-light function, card frame
- ✅ Background telemetry worker (`core/worker.rs`) — 500ms interval, `System::uptime()` via `mpsc`
- ✅ Modular app structure (`app.rs`) — top panel (title, version, live uptime), left sidebar (180px, Memory tab), central panel (card placeholder)
- ✅ Minimal entry point (`main.rs` — 27 lines)
- ✅ **13 unit tests passing** (theme: 6, app: 5, core: 2)
- ✅ `cargo check` — zero warnings
- ✅ Release profile optimized (LTO=fat, strip, panic=abort)

**Running:**
```bash
cargo run --release
```
Opens a 1200×800 window with live uptime updating every 100ms.

---

## 🛠️ Requirements

- **Rust 1.78+** (MSVC toolchain for Windows)
- **Windows 10/11 (x64)**
- Dependencies pinned in `Cargo.toml` (see [Pinned Dependencies](#-pinned-dependencies))

---

## 📦 Pinned Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `eframe` | 0.27.2 | Native egui framework (window, event loop, wgpu) |
| `egui` | 0.27.2 | Immediate-mode GUI |
| `egui_plot` | 0.27.2 | Real-time graphs (CPU/RAM/GPU history) |
| `egui_extras` | 0.27.2 | Table, Strip, Image widgets |
| `sysinfo` | 0.30.12 | Cross-platform system info (CPU, RAM, Disks, Processes) |
| `windows-sys` | 0.52.0 | Direct Windows API (PSAPI, Threading, Foundation) for `EmptyWorkingSet` |
| `chrono` | 0.4.38 | Date/time formatting |
| `serde` / `serde_json` | 1.0.208 / 1.0.123 | Configuration serialization |
| `tracing` / `tracing-subscriber` | 0.1.40 / 0.3.18 | Structured logging |
| `tokio` | 1.40.1 | Async runtime for background tasks |
| `rayon` | 1.10.0 | Parallel processing for heavy workloads |

> **Never use `^`, `~`, `*`, or `latest`.** Versions are locked for reproducible builds.

---

## 📋 Definition of Done (Per Version)

Every version must complete all 5 steps:

1. **Code Quality** — `cargo check` (0 errors, 0 warnings), `cargo test` (all passing)
2. **Version Bump** — Update `version` in `Cargo.toml` (SemVer)
3. **Changelog** — Add entry to `CHANGELOG.md` (Keep a Changelog format)
4. **External History** — Append detailed block to `../PULSE_AI_CONTEXT.md` (never delete old entries)
5. **Git Commit** — `git add . && git commit -m "vX.Y.Z: description"`

---

## 📜 License

MIT License — see [LICENSE](LICENSE) for details.

---

## 🔗 Links

- **Changelog:** [CHANGELOG.md](CHANGELOG.md)
- **Architecture:** [ARCHITECTURE.md](ARCHITECTURE.md)
- **External Engineering Log (Ukrainian):** `../PULSE_AI_CONTEXT.md` (not in repo)
- **Owner Manual (Ukrainian):** `../PULSE_UA_MANUAL.md` (not in repo)