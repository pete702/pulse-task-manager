# ⚡ PULSE — Ultra-Fast Modular Task Manager & System Monitor for Windows

![Version](https://img.shields.io/badge/version-v0.0.9-00F0FF)
![Tests](https://img.shields.io/badge/tests-72%20passing-00F0FF)
![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-1F2330)
![License](https://img.shields.io/badge/license-MIT-FFB800)

> **Next-Gen System Telemetry for Windows** — Built in Rust + egui with an **Obsidian HUD** aesthetic.

---

## 📖 Description

**PULSE** is a modern, high-performance system monitor and task manager for Windows, written in **Rust** using **eframe/egui**. The project is built on three pillars:

- **Strict Modularity** — Clear `model.rs` (data + pure logic + tests), `collector.rs` (OS data collection), `view.rs` (UI) separation. Files target ≤180 lines; legacy modules are being split incrementally.
- **Zero-Cost Abstractions** — Pinned dependency versions for 100% API compatibility. Release profile uses thin LTO, strip, panic=abort.
- **Reference-Grade "Obsidian HUD" UI** — Dark theme (`#0B0C10` window, `#12141C` cards, `#1F2330` borders, 6px radius). All numbers (MB, GB, %, PID) rendered in monospace. Traffic-light load colors: `<60%` cyan (`#00F0FF`), `60–85%` amber (`#FFB800`), `≥85%` neon red (`#FF2A54`).

![PULSE — Memory tab (English UI)](screenshot.png)

*Memory tab: RAM/Swap cards, kernel metrics, and 1m/5m/15m utilization history.*

---

## ✨ Key Features (Roadmap — 7 Phases to v1.0.0)

| Phase | Versions | Focus | Status |
|-------|----------|-------|--------|
| **1. Foundation & Infrastructure** | v0.0.1–v0.0.2 | Git, Cargo, Obsidian HUD theme, background worker (500ms uptime), basic app structure | ✅ **Done** |
| **2. i18n, Fonts, Icon, Self-Test, CI** | v0.0.3 | i18n with translation keys, automatic UI language detection from Windows (since v0.0.7), system fonts (Segoe UI/Consolas), generated neon icon, `--self-test` (headless CI + live GUI), GitHub Actions CI, backup automation, self-diagnosis badge | ✅ **Done** |
| **3. Memory Math Model & Ring Buffer** | v0.0.4 | Pure math `MemorySnapshot` (RAM/Swap bytes, GiB/MiB, saturating cached, clamped usage %, division-by-zero guard), `MemoryHistory` ring buffer (120 points = 60s), session peaks, 6 new unit tests | ✅ **Done** |
| **4. OS Memory Collector & Worker Integration** | v0.0.5 | `sysinfo` RAM/Swap collection, `EmptyWorkingSet` via `windows-sys` PSAPI, worker integration with `MemorySnapshot` + `MemoryHistory`, headless CI fix for OpenGL-less runners | ✅ **Done** |
| **5. Memory HUD Cards & 60s Chart** | v0.0.6 | RAM/Swap cards with `card_frame()`, progress bars with traffic-light colors, `egui_plot` dual-line chart (ring buffer), one-click cache cleanup | ✅ **Done** |
| **6. OS Audit & Code Review** | v0.0.7 | Auto-detect Windows UI Language (GetUserDefaultUILanguage), fix SystemCache bytes (K32GetPerformanceInfo), stretch 60s plot to full width, optimize worker (refresh_pid), remove bilingual text, extract UI chrome module | ✅ **Done** |
| **7. Multi-Horizon Chart & Process Memory Model** | v0.0.8 | 1m/5m/15m chart horizons (900-point ring buffer), `ProcessMemoryItem` clone grouping, `is_hog` detector, `MemorySpikeEvent` black box, `K32EmptyWorkingSet` self-trim, real UI render timing, full localization (language chosen from the Windows locale) | ✅ **Done** |
| **8. Interactive Process Table** | v0.0.9 | Mission Center-style sub-view switcher, virtualized `egui_extras::TableBuilder` (O(visible_rows)), instant search, clone grouping `[Nx]`, sortable columns, `⚠️ HOG` badges, spike black-box banner | ✅ **Done** |
| **9. Phase 1 Release: WinAPI RAM Purge** | **v0.1.0** | Full RAM/Swap monitoring + one-click cache cleanup end-to-end, `CHANGELOG.md` + `PULSE_AI_CONTEXT.md` updated, Git tag | 🎯 **Target** |
| **10. CPU Module** | v0.2.0 | Per-core usage, frequency, temperature (MSR), `egui_plot` graphs, top consumers table | ⏳ Planned |
| **11. Storage I/O Module** | v0.3.0 | Disk read/write throughput, latency, queue depth, SMART health, partition usage | ⏳ Planned |
| **12. Network & Ping Radar** | v0.4.0 | Per-interface throughput, TCP/UDP stats, gamer ping, connection table | ⏳ Planned |
| **13. GPU DXGI & System** | v0.5.0 | DXGI GPU usage/VRAM/temp, process GPU attribution | ⏳ Planned |
| **14. Mini HUD Overlay** | v1.0.0 | Always-on-top overlay, hotkey toggle, customizable panels, export HTML/JSON | 🎯 **Final** |

---

## 🏗️ Architecture

```
pulse_manager/
├── src/
│   ├── main.rs                      # Entry point: eframe::run_native, --self-test, icon, windows_subsystem
│   ├── app.rs                       # PulseApp: tabs, Memory sub-view, table state, snapshots, render timing
│   ├── theme.rs                     # Obsidian HUD tokens, apply_obsidian_theme, load_color, card_frame, setup_fonts
│   ├── i18n.rs                      # UI language auto-detected from Windows (GetUserDefaultUILanguage)
│   ├── core/
│   │   ├── mod.rs                   # SystemSnapshot (uptime + pulse_ram_mb + memory), mpsc channel types, spawn_telemetry_worker
│   │   └── worker.rs                # Background thread: 1000 ms interval, process scan + self-trim + self RSS → channel
│   ├── ui/
│   │   ├── mod.rs                   # UI chrome exports
│   │   └── chrome.rs                # Top panel, sidebar, status bar rendering
│   └── modules/
│       ├── mod.rs                   # Pub re-exports all modules
│       └── memory/
│           ├── mod.rs               # Module exports
│           ├── model.rs             # MemorySnapshot, MemoryHistory (900 pts / 15 min), ChartWindow
│           ├── process_model.rs     # ProcessMemoryItem, clone grouping, sorting, MemorySpikeEvent
│           ├── collector.rs         # sysinfo RAM/Swap, K32GetPerformanceInfo, K32GetProcessMemoryInfo, K32EmptyWorkingSet
│           ├── view.rs              # RAM/Swap cards, kernel tiles, localized labels
│           ├── chart_view.rs        # Multi-horizon (1m/5m/15m) dual-line neon chart
│           ├── page.rs              # Mission Center sub-view switcher: Overview / Processes
│           ├── table_view.rs        # MemoryTableState + search/group/sort orchestration
│           ├── table_body.rs        # Virtualized egui_extras rows and sortable header (O(visible_rows))
│           └── table_widgets.rs     # Spike black-box banner, toolbar, [Nx]/⚠️ HOG badges, byte formatting
```

### Core Principles

- **Module Boundaries** — Target ≤180 lines/file. Strict `model/collector/view` separation.
- **Documentation** — All public API documented in English (`///`).
- **Pinned Dependencies** — Exact versions in `Cargo.toml` (eframe 0.27.2, sysinfo 0.30.12, windows-sys 0.52.0).
- **Design Tokens** — Single source of truth in `theme.rs::colors`.
- **Monospace Numbers** — All numeric values use `.monospace()`.
- **Traffic-Light Colors** — `load_color(percent)` for all load indicators.
- **Non-Blocking UI** — Background worker sends via `mpsc`; UI reads via `try_recv()`.
- **Virtualized Rendering** — `egui_extras::TableBuilder` paints only visible rows, so the table cost is O(visible_rows), not O(all processes).
- **Safety-Gated Actions** — Per-process Win32 operations are designed to pass a PID safety gate before touching another process.

---

## 🧠 NVIDIA Agent Skills Engineering Patterns

PULSE distills six reusable engineering patterns from NVIDIA Agent Skills into concrete Rust/egui mechanisms. The table records both the pattern and its current implementation status, so the architecture stays honest as the project grows.

| NVIDIA Agent Skill | PULSE Engineering Pattern | Implementation | Status |
|--------------------|---------------------------|----------------|--------|
| `ethos-orchestrate` / `mcore-testing` | **Self-Enforcing Architecture Guard** — an automated test scans every `src/**/*.rs` file and fails `cargo test` when a file exceeds the line ceiling. | `src/main.rs::test_all_rust_files_under_reasonable_limit` (600-line pragmatic gate; 180-line target in v0.1.0) | ✅ Active |
| `nemo-mbridge-resiliency` | **Resource Hog Detector (`⚠️ HOG`) + Spike Blackbox** — flags processes at ≥15% of total RAM and records a `MemorySpikeEvent` when system RAM reaches ≥85%, including the culprit PID and byte count. | `process_model.rs`, `table_body.rs`, `table_widgets.rs` | ✅ Active (relative threshold; absolute ≥1.5 GiB check queued for v0.1.0) |
| `tao-launch-workflow` | **Pre-launch Safety Gate for PID** — before any Win32 action, reject PID 0 (System Idle), PID 4 (System), PULSE's own PID, and critical OS processes. | Integration point reserved in `collector.rs`; per-process reclamation lands in v0.1.0 | 🛡️ Design contract |
| `jetson-memory-audit` | **Before/After Memory Reclamation Audit** — measure working-set bytes immediately before and after `K32EmptyWorkingSet`, then report the exact freed delta. | `K32GetProcessMemoryInfo` / `own_working_set_mb` measurement primitives in `collector.rs`; audit record in v0.1.0 | 🔬 Primitive ready |
| `nemo-mbridge-perf-moe-optimization-workflow` | **Three Walls Bottleneck Radar** — classify the system as `SYSTEM OPTIMAL`, `MEMORY WALL`, `COMPUTE WALL`, or `I/O WALL` from live telemetry. | `MEMORY WALL` is available from RAM/commit usage today; `COMPUTE WALL` arrives with the CPU module (v0.2.0), `I/O WALL` with Storage/Network (v0.3.0–v0.4.0) | 🧭 Partial |
| `medtech-model-evidence-export` | **Sanitized Diagnostic Report** — export a shareable diagnostic report with `[REDACTED]` masking for machine names, user paths, IP/MAC addresses, and other PII. | Export layer planned for the v1.0.0 reporting milestone | 📋 Planned |

---

## 🚀 Current Status: v0.0.9

**Implemented:**
- ✅ **Foundation & Infrastructure** (v0.0.1–v0.0.2): Git, Cargo, Obsidian HUD theme, background worker (500ms uptime), basic app structure
- ✅ **i18n, Fonts, Icon, Self-Test, CI** (v0.0.3): i18n with translation keys, automatic UI language detection from Windows (since v0.0.7), system fonts (Segoe UI/Consolas), generated neon icon, `--self-test` (headless CI + live GUI), GitHub Actions CI, backup automation, self-diagnosis badge
- ✅ **Memory Math Model & Ring Buffer** (v0.0.4): Pure math `MemorySnapshot` (RAM/Swap bytes, GiB/MiB, saturating cached, clamped usage %, division-by-zero guard), `MemoryHistory` ring buffer (120 points = 60s), session peaks, 6 new unit tests
- ✅ **OS Memory Collector & Worker Integration** (v0.0.5): `sysinfo` RAM/Swap collection via `collect_memory()`, `EmptyWorkingSet` wrapper via `windows-sys` PSAPI, worker integration with `MemorySnapshot` + `MemoryHistory`, headless CI fix for OpenGL-less runners (headless self-test without `eframe::run_native`)
- ✅ **Memory HUD Cards & 60s Chart** (v0.0.6): Dual RAM/Swap cards with traffic-light progress bars, 4 kernel mini-tiles (System Cache, Kernel Pool, Commit Charge, OS Handles/Threads), `egui_plot` dual-line neon chart (60s history), GitHub Release pipeline
- ✅ **OS Audit & Code Review** (v0.0.7): Auto-detect Windows UI Language via `GetUserDefaultUILanguage()` (no manual switcher), fixed real SystemCache via `K32GetPerformanceInfo` (cb init + override field), 60s chart stretches to full card width (`.width(ui.available_width())`), worker optimized to `refresh_pids(&[pid])` (~35-50MB RAM vs 150MB), all UI text 100% localized via i18n (no bilingual "vinagret"), UI chrome extracted to `src/ui/chrome.rs` (< 180 lines/file)
- ✅ **Multi-Horizon Chart & Process Memory Model** (v0.0.8): 1m/5m/15m chart switcher with 900-point (15-min) ring buffer; `process_model.rs` with `ProcessMemoryItem`, `MemorySortCol`, case-insensitive search, clone grouping, `is_hog` (RAM ≥ 15%), and `MemorySpikeEvent` black box (RAM ≥ 85%); `K32EmptyWorkingSet` self-trim before RSS measurement; real `update()` render timing; fully localized cards, kernel tiles, and status bar (language auto-selected from the Windows locale, no mixed-language leftovers)
- ✅ **Interactive Process Table** (v0.0.9): Mission Center-style `◈ Telemetry Overview` / `📋 Memory Processes (N)` sub-view switcher; virtualized `egui_extras::TableBuilder` with O(visible_rows) rendering; instant name/PID search; clone grouping with `[Nx]` badges; sortable columns; `⚠️ HOG` badges; spike black-box banner with culprit PID and bytes; compact RAM bar; row selection state
- ✅ **72 unit tests passing** (theme: 7, app: 10, core: 3, i18n: 10, memory model: 12, collector: 8, process model: 9, chart: 2, view: 2, page: 2, table view: 2, table body: 1, table widgets: 2, ui: 1, file size: 1)
- ✅ `cargo clippy -- -D warnings` — zero warnings
- ✅ Self-test: **HEADLESS CI** (`CI=true` / `GITHUB_ACTIONS=true`) — runs without `eframe::run_native`, validates live snapshot + memory history + i18n toggle; **LIVE GUI** — auto-switches tabs & lang
- ✅ Release profile optimized (thin LTO, strip, panic=abort)
- ✅ GitHub Actions CI (`.github/workflows/ci.yml`) — fmt, clippy, check, test, build, self-test (headless)
- ✅ GitHub Release pipeline (`.github/workflows/release.yml`) — triggered on `v*` tags, builds release, runs headless self-test, uploads `.exe` + `.sha256`

**Running:**
```bash
cargo run --release
```
Opens a 1180×760 window with live uptime, self-diagnosis badge, auto-detected locale indicator, and memory telemetry updating every 1000 ms (1s). The Memory tab opens on `◈ Telemetry Overview` with RAM/Swap cards, kernel tiles, and a 1m/5m/15m history chart. Switch to `📋 Memory Processes (N)` for the virtualized process table: search by name or PID, group clone processes with `[Nx]`, sort by any column, and spot `⚠️ HOG` processes alongside the latest RAM spike banner.

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
| **3** | **Unit Tests** | `cargo test` | **All tests pass** (min 3–5 new per step, including i18n coverage) |
| **4** | **Self-Test** | `cargo run -- --self-test` | Prints `SELF-TEST PASSED`, auto-switches tabs & lang, exits 0 |
| **5** | **Build & Export** | `cargo build --release` → copy to `../PULSE_BUILDS/pulse_vX.Y.Z.exe` + `PULSE_LATEST.exe` | Both `.exe` exist and run |
| **6** | **Clean Backup** | Copy `src/`, `Cargo.toml`, `Cargo.lock`, `*.md` to `../PULSE_BACKUPS/vX.Y.Z_verified/` (NO `target/`, NO `.git/`) | Backup folder complete |
| **7** | **Documentation** | Update `CHANGELOG.md` (EN), append to `../PULSE_AI_CONTEXT.md` (UA), update `../PULSE_UA_MANUAL.md` (UA) — **never delete history** | All three files current |
| **8** | **Git Push** | `git add . && git commit -m "vX.Y.Z: description" && git push` | Remote `main` updated |

### Iron Testing Standard
- **Every step** (including visual `view.rs`, `chart_view.rs`, `table_view.rs`) adds **minimum 3–5 new unit tests** covering: math logic, i18n translations, UI state transitions.
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