# ⚡ PULSE — Ultra-Fast Modular Task Manager & System Monitor for Windows

> **Next-Gen System Telemetry for Windows** — Built in Rust + egui with an **Obsidian HUD** aesthetic.

---

## 📖 Description

**PULSE** is a modern, high-performance system monitor and task manager for Windows, written in **Rust** using **eframe/egui**. The project is built on three pillars:

- **Strict Modularity** — No file exceeds 150 lines. Every module follows `model.rs` (data + pure logic + tests), `collector.rs` (OS data collection), `view.rs` (UI).
- **Zero-Cost Abstractions** — Pinned dependency versions for 100% API compatibility. Release profile uses LTO, strip, panic=abort.
- **Reference-Grade "Obsidian HUD" UI** — Dark theme (`#0B0C10` window, `#12141C` cards, `#1F2330` borders, 6px radius). All numbers (MB, GB, %, PID) rendered in monospace. Traffic-light load colors: `<60%` cyan (`#00F0FF`), `60–85%` amber (`#FFB800`), `≥85%` neon red (`#FF2A54`).

---

## ✨ Key Features (Roadmap — 6 Phases to v1.0.0)

| Phase | Versions | Focus | Status |
|-------|----------|-------|--------|
| **1. Foundation & Infrastructure** | v0.0.1–v0.0.3 | Git, Cargo, Obsidian HUD theme, background worker (500ms), i18n (UA/EN), system fonts, generated icon, self-test, CI, backup, self-diagnosis badge | ✅ **Done** |
| **2. Memory Module** | v0.0.4–v0.0.6 | Pure math model (`MemorySnapshot`, `MemoryHistory` ring buffer), `sysinfo` collector, RAM/Swap cards, progress bars, `egui_plot` history, one-click cache cleanup | 🔄 **In Progress** (v0.0.4) |
| **3. Processes Module** | v0.0.7–v0.0.9 | `ProcessEntry`, `GroupKey`, `ProcessGroup`, `sysinfo::Processes` + Windows enrichment, `egui_extras::TableBuilder` with monospace PID, grouping, search | ⏳ Planned |
| **4. CPU Module** | v0.1.0–v0.2.0 | Per-core usage, frequency, temperature (MSR), `egui_plot` graphs, top consumers table | ⏳ Planned |
| **5. Storage I/O Module** | v0.2.0–v0.3.0 | Disk read/write throughput, latency, queue depth, SMART health, partition usage | ⏳ Planned |
| **6. Network & GPU Module** | v0.3.0–v0.5.0 | Per-interface throughput, TCP/UDP stats, gamer ping, DXGI GPU usage/VRAM/temp | ⏳ Planned |
| **7. Mini HUD Overlay** | v0.5.0–v1.0.0 | Always-on-top overlay, hotkey toggle, customizable panels, export to HTML/JSON | 🎯 **Final Target** |

---

## 🏗️ Architecture

```
pulse_manager/
├── src/
│   ├── main.rs              # Entry point: eframe::run_native (≤35 lines)
│   ├── app.rs               # PulseApp: state, UI panels, snapshot handling
│   ├── theme.rs             # Obsidian HUD tokens, apply_obsidian_theme, load_color, card_frame, setup_fonts
│   ├── i18n.rs              # Language switcher (UA/EN), translation keys
│   ├── core/
│   │   ├── mod.rs           # SystemSnapshot, mpsc channel types, spawn_telemetry_worker
│   │   └── worker.rs        # Background thread: 500ms interval, uptime + self RAM → channel
│   ├── modules/
│   │   ├── mod.rs           # Pub re-exports all modules
│   │   ├── memory/
│   │   │   ├── mod.rs       # Module exports
│   │   │   ├── model.rs     # MemorySnapshot, MemoryHistory (pure math, no deps)
│   │   │   ├── collector.rs # sysinfo RAM/Swap, EmptyWorkingSet via windows-sys
│   │   │   └── view.rs      # RAM/Swap cards, progress bars, egui_plot graphs, Clear Cache button
│   │   ├── processes/
│   │   │   ├── mod.rs
│   │   │   ├── model.rs     # ProcessEntry, GroupKey, ProcessGroup
│   │   │   ├── collector.rs # sysinfo::Processes + Windows enrichment
│   │   │   └── view.rs      # Table, grouping, search, monospace PID
│   │   └── system/
│   │       ├── mod.rs
│   │       ├── model.rs     # CpuInfo, GpuInfo, DiskInfo, NetInfo
│   │       ├── collector.rs # sysinfo collectors
│   │       └── view.rs      # System overview panels
│   └── utils/
│       ├── mod.rs
│       ├── formatting.rs    # bytes_to_human, monospace helpers
│       └── windows_api.rs   # Wrappers for EmptyWorkingSet, PSAPI
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

## 🚀 Current Status: v0.0.3

**Implemented:**
- ✅ Obsidian HUD theme system (`theme.rs`) with design tokens, traffic-light function, card frame, system fonts
- ✅ Background telemetry worker (`core/worker.rs`) — 500ms interval, `System::uptime()` + self RAM via `mpsc`
- ✅ Modular app structure (`app.rs`) — top panel (title, version, live uptime, lang switch, self-diag badge), left sidebar (180px, Memory tab), central panel (card placeholder)
- ✅ Internationalization (`i18n.rs`) — `Lang { Uk, En }`, `[ UA | EN ]` button, type-safe keys
- ✅ Minimal entry point (`main.rs` — 27 lines) with `#![deny(unsafe_code)]`, `windows_subsystem`, custom icon, `--self-test`
- ✅ **23 unit tests passing** (theme: 6, app: 7, core: 2, i18n: 8)
- ✅ `cargo clippy --all-targets -- -D warnings` — zero warnings
- ✅ Self-test: **HEADLESS CI** (`CI=true`) and **LIVE GUI** (5 frames, auto-switch tabs/lang)
- ✅ Release profile optimized (LTO=fat, strip, panic=abort)
- ✅ GitHub Actions CI (`.github/workflows/ci.yml`)

**Running:**
```bash
cargo run --release
```
Opens a 1180×760 window with live uptime updating every 500ms.

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

## 📋 Definition of Done (Per Version — 8 Iron Rules)

Every version must complete **all 8 steps**:

| # | Action | Command | Success Criteria |
|---|--------|---------|------------------|
| **1** | **Format** | `cargo fmt` | Zero formatting diffs |
| **2** | **Clippy** | `cargo clippy --all-targets -- -D warnings` | **0 warnings, 0 errors** |
| **3** | **Unit Tests** | `cargo test` | **All tests pass** (min 3–5 new per step, including i18n for Uk/En) |
| **4** | **Self-Test** | `cargo run -- --self-test` | Prints `SELF-TEST PASSED`, auto-switches tabs & lang, exits 0 |
| **5** | **Build & Export** | `cargo build --release` → copy to `../PULSE_BUILDS/pulse_vX.Y.Z.exe` + `PULSE_LATEST.exe` | Both `.exe` exist and run |
| **6** | **Clean Backup** | Copy `src/`, `Cargo.toml`, `Cargo.lock`, `*.md` to `../PULSE_BACKUPS/vX.Y.Z_verified/` (NO `target/`, NO `.git/`) | Backup folder complete |
| **7** | **Documentation** | Update `CHANGELOG.md` (EN), append to `../PULSE_AI_CONTEXT.md` (UA), update `../PULSE_UA_MANUAL.md` (UA) — **never delete history** | All three files current |
| **8** | **Git Push** | `git add . && git commit -m "vX.Y.Z: description" && git push` | Remote `main` updated |

### Iron Testing Standard
- **Every step** (including visual `view.rs`, `chart_view.rs`, `table_view.rs`) adds **minimum 3–5 new unit tests** covering: math logic, i18n translations (Uk + En), UI state transitions.
- **Live self-test** (`--self-test`) must **auto-switch all tabs and languages** (`Lang::Uk` → `Lang::En`) during test frames before auto-close to guarantee zero panics in rendering.

---

## 📜 License

MIT License — see [LICENSE](LICENSE) for details.

---

## 🔗 Links

- **Changelog:** [CHANGELOG.md](CHANGELOG.md)
- **Architecture:** [ARCHITECTURE.md](ARCHITECTURE.md)
- **External Engineering Log (Ukrainian):** `../PULSE_AI_CONTEXT.md` (not in repo)
- **Owner Manual (Ukrainian):** `../PULSE_UA_MANUAL.md` (not in repo)