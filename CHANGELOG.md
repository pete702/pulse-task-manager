# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [v0.0.6] - 2026-11-02

### Added
- **Full Memory HUD Dashboard** (`src/modules/memory/view.rs`, `src/modules/memory/chart_view.rs`): Complete Obsidian HUD memory visualization:
  - **Dual RAM/Swap Cards**: Large progress bars with traffic-light colors (cyan <60%, amber 60-85%, red ≥85%), GiB values, cached memory display, session peak tracking.
  - **Swap & Commit Card**: Shows swap usage or commit charge percentage, progress bar with purple accent, commit total/limit in GiB.
  - **4 Kernel Mini-tiles**: System Cache (cyan), Kernel Pool Paged/Non-paged (amber), Commit Charge (purple), OS Handles/Threads/Processes counters.
  - **60-Second Live Chart**: Dual-line neon chart (RAM% cyan, Swap% purple) with 120-point ring buffer history (500ms interval = 60s window).
- **Windows Kernel Metrics** (`src/modules/memory/collector.rs`): `K32GetPerformanceInfo` integration via `windows-sys` PSAPI:
  - Real system cache (`SystemCache * PageSize`), kernel paged/non-paged pools, commit charge/limit, handles, threads, processes.
  - Replaces `sysinfo` cached calculation (which was always 0 on Windows) with real kernel data.
- **6 New Unit Tests**: Memory view formatting, chart rendering, kernel metrics collection validation.
- **Release Pipeline** (`.github/workflows/release.yml`): GitHub Actions workflow triggered on `v*` tags, builds release, runs headless self-test, uploads `PULSE_vX.Y.Z_windows_x64.exe` + SHA256 to GitHub Releases via `softprops/action-gh-release@v2`.

### Changed
- **`src/modules/memory/model.rs`**: Extended `MemorySnapshot` with kernel metrics (`commit_total_bytes`, `commit_limit_bytes`, `kernel_paged_bytes`, `kernel_nonpaged_bytes`, `system_handles`, `system_threads`, `system_processes`); added `commit_usage_percent()`.
- **`src/modules/memory/collector.rs`**: `collect_memory()` now calls `K32GetPerformanceInfo` on Windows for real kernel metrics; `ram_cached_bytes()` now uses real System Cache instead of `available - free` fallback.
- **`src/modules/memory/view.rs`**: New `render_memory_cards()` with dual cards, progress bars, kernel mini-tiles; `render_memory_chart()` with dual-line neon chart.
- **`src/modules/memory/chart_view.rs`**: New `render_memory_chart()` with dual-line neon chart (RAM cyan, Swap purple), 60s scrolling window.
- **`src/app.rs`**: Memory tab renders full HUD dashboard; sidebar shows live RAM% on Memory tab button; status bar with kernel metrics.
- **`src/theme.rs`**: Added `TEXT_PRIMARY`, `TEXT_SECONDARY`, `ACCENT_CYAN`, `ACCENT_AMBER`, `ACCENT_RED`, `ACCENT_PURPLE`, `SIDEBAR_ACTIVE_BG`, `SIDEBAR_ACTIVE_BORDER`, `STATUS_BAR_BG`; new `status_bar_frame()`.
- **`src/main.rs`**: Added file size limit test (600 lines with TODO for 150 architectural limit); file size check uses `is_some_and`.
- **`Cargo.toml`**: Version bumped to `0.0.6`.

### Infrastructure
- All **45 unit tests passing** (memory model: 8, collector: 3, view: 2, chart: 1, app: 8, theme: 5, i18n: 8, core: 3, integration: 1, file size: 1).
- Zero warnings on `cargo clippy --all-targets -- -D warnings`.
- Self-test passes **HEADLESS CI** (`CI=true`) without OpenGL, and **LIVE GUI** (auto-switches tabs & lang).
- Release binary exported to `../PULSE_BUILDS/pulse_v0.0.6.exe` and `PULSE_LATEST.exe`.
- Clean backup created at `../PULSE_BACKUPS/v0.0.6_verified/`.

---

## [v0.0.5] - 2026-11-02

### Added
- **Memory Module — OS Collector** (`src/modules/memory/collector.rs`): Real OS memory collection via `sysinfo`:
  - `collect_memory(sys)`: Refreshes and maps `sys.total_memory()`, `used_memory()`, `available_memory()`, `free_memory()`, `total_swap()`, `used_swap()`, `free_swap()` to `MemorySnapshot`.
  - `trim_working_set()`: Placeholder for Windows `EmptyWorkingSet` via `windows-sys` PSAPI (to be implemented in v0.0.6).
  - **3 new unit tests**: live OS collection (`ram_total > 0`, `ram_usage > 0`), field population validation, accounting consistency (used + free + cached ≈ total within 1 MiB).
- **Worker Integration** (`src/core/worker.rs`): Background thread now collects OS memory every 500ms via `collect_memory()` and includes `MemorySnapshot` in `SystemSnapshot::memory`.
- **App Integration** (`src/app.rs`): `PulseApp` now holds `MemoryHistory` ring buffer; pushes each `MemorySnapshot` to history on receipt; central panel displays live RAM/Swap GiB, usage %, cached GiB, history points count, and session peak RAM %.
- **Headless CI Fix** (`src/main.rs`): Self-test now detects CI (`CI=true` or `GITHUB_ACTIONS=true`) and runs **fully headless without `eframe::run_native`** (no OpenGL required). Falls back to headless if eframe fails locally. Validates: live snapshot, `pulse_ram_mb > 0`, `MemorySnapshot` fields, `MemoryHistory` push/peaks, i18n toggle `Uk` ↔ `En`.
- **GitHub Actions CI** (`.github/workflows/ci.yml`): Self-test runs headless on `windows-latest` without GPU.

### Changed
- **`src/core/mod.rs`**: `SystemSnapshot` now includes `memory: Option<MemorySnapshot>`; added `minimal()` constructor for tests.
- **`src/core/worker.rs`**: Collects OS memory via `collect_memory()` every 500ms; sends full `SystemSnapshot` with memory.
- **`src/app.rs`**: Added `memory_history: MemoryHistory`; pushes memory on each snapshot; central panel displays live RAM/Swap GiB, usage %, cached, history count, peak.
- **`src/main.rs`**: Self-test detects `CI` or `GITHUB_ACTIONS` env vars → runs `run_headless_self_test()` without eframe; fallback to headless on eframe error (e.g., no OpenGL).
- **`Cargo.toml`**: Version bumped to `0.0.5`.

### Infrastructure
- All **34 unit tests passing** (theme: 6, app: 8, core: 3, i18n: 8, memory model: 6, memory collector: 3, integration: 1).
- Zero warnings on `cargo clippy --all-targets -- -D warnings`.
- Self-test passes **HEADLESS CI** (`CI=true`) without OpenGL, and **LIVE GUI** (auto-switches tabs & lang).
- Release binary exported to `../PULSE_BUILDS/pulse_v0.0.5.exe` and `PULSE_LATEST.exe`.
- Clean backup created at `../PULSE_BACKUPS/v0.0.5_verified/`.

---

## [v0.0.4] - 2026-11-02

### Added
- **Memory Module — Pure Math Model** (`src/modules/memory/model.rs`): Zero-dependency pure math model for memory telemetry:
  - `MemorySnapshot`: RAM/Swap bytes (total, used, available, free, cached via `saturating_sub`), GiB/MiB conversion, usage % with clamping `[0.0, 100.0]`, division-by-zero guard for disabled swap.
  - `MemoryHistory`: Ring buffer (`VecDeque<f32>`) with fixed capacity of 120 points (60s at 500ms), session peak tracking for RAM/Swap, FIFO eviction.
  - **6 new unit tests**: swap division-by-zero guard, `saturating_sub` for cached bytes, GiB/MiB conversion, usage % clamping, ring buffer 120-point capacity + peak preservation, i18n `Lang::Uk` <-> `Lang::En` toggle.
- **Memory Module Exports** (`src/modules/memory/mod.rs`, `src/modules/mod.rs`): Module structure ready for collector/view.
- **LICENSE**: MIT License file added to repository root.

### Changed
- **`src/main.rs`**: Added `mod modules;` declaration to include memory module.
- **`src/modules/memory/mod.rs`**: Created with `pub mod model;`.
- **`Cargo.toml`**: Version bumped to `0.0.4`.
- **`README.md`**, **`ARCHITECTURE.md`**, **`../PULSE_UA_MANUAL.md`**: Updated roadmap to 7 phases (v0.0.1–v1.0.0), added Iron Testing Standard (min 3–5 tests/step, i18n Uk/En, self-test auto-switch tabs/lang), updated DoD to 8 rules.

### Infrastructure
- All **29 unit tests passing** (theme: 6, app: 7, core: 2, i18n: 8, memory: 6).
- Zero warnings on `cargo clippy --all-targets -- -D warnings`.
- Self-test passes both headless (CI) and live GUI modes (auto-switches tabs & lang).
- Release binary exported to `../PULSE_BUILDS/pulse_v0.0.4.exe` and `PULSE_LATEST.exe`.
- Clean backup created at `../PULSE_BACKUPS/v0.0.4_verified/`.

---

## [v0.0.3] - 2026-11-02

### Added
- **Internationalization (i18n)** (`src/i18n.rs`): Language switcher `Lang { Uk, En }` with default Uk, instant toggle via `[ UA | EN ]` button in top bar. Type-safe translation keys for all UI elements. Unit tests for language switching and translations.
- **System Fonts** (`src/theme.rs::setup_fonts`): Loads Segoe UI (proportional) and Consolas (monospace) from `C:\Windows\Fonts\` with safe fallback to egui defaults.
- **Application Icon**: Programmatically generated 32x32 RGBA icon — dark `#0B0C10` background with neon cyan `#00F0FF` pulse wave symbol.
- **Self-Diagnosis Badge**: Top-right corner shows `[ PULSE: XX.X MB | UI: X.XX ms ]` — live process RAM and frame render time (monospace).
- **Self-Test Mode** (`--self-test`): CI headless mode (`CI=true`) validates `pulse_ram_mb > 0` and exits 0; Local GUI mode renders 5 frames after first snapshot, validates, auto-closes.
- **GitHub Actions CI** (`.github/workflows/ci.yml`): `windows-latest`, fmt, clippy -D warnings, check, test, release build, self-test.
- **Updated DoD (Definition of Done)**: 8 iron rules (fmt, clippy, test, self-test, build/export, clean backup, docs, git push) in `ARCHITECTURE.md`.

### Changed
- **`src/main.rs`**: Added `#![deny(unsafe_code)]`, `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`, viewport 1180x760 (min 950x620), custom icon, `--self-test` argument parsing.
- **`src/core/mod.rs`**: `SystemSnapshot` extended with `pulse_ram_mb: f32` (self process RAM in MB).
- **`src/core/worker.rs`**: Worker now collects `pulse_ram_mb` through `sysinfo::Process::memory()` every 500ms. Unit tests updated.
- **`src/app.rs`**: Language switcher button, self-diagnosis badge, `request_repaint_after(500ms)` for economical updates. `SelfTestApp` for automated testing. `ActiveTab::label()` now uses i18n. **7 unit tests** (dodano i18n keys, tab labels UK/EN).
- **`src/theme.rs`**: Added `setup_fonts(ctx)` — завантажує Segoe UI (Proportional) та Consolas (Monospace) з `C:\Windows\Fonts\` з фолбеком на egui defaults. Використовуються лише геометричні Unicode символи (◈, ▣, ⚡, ▲, ▼, ●).
- **`Cargo.toml`**: Version bumped to `0.0.3`.

### Infrastructure
- All **23 unit tests passing** (theme: 6, app: 7, core: 2, i18n: 8).
- Zero warnings on `cargo clippy --all-targets -- -D warnings`.
- Self-test passes both headless (CI) and live GUI modes (auto-switches tabs & lang).
- Release binary exported to `../PULSE_BUILDS/pulse_v0.0.3.exe` and `PULSE_LATEST.exe`.
- Clean backup created at `../PULSE_BACKUPS/v0.0.3_verified/`.

---

## [v0.0.2] - 2026-11-02

### Added
- **Theme System** (`src/theme.rs`): Obsidian HUD design tokens, `apply_obsidian_theme()`, `load_color()` traffic-light function, `card_frame()` for consistent card styling. Unit tests for colors, traffic-light thresholds, and frame properties.
- **Core Telemetry Worker** (`src/core/`): Background thread (`worker.rs`) collecting `System::uptime()` every 500ms, sending `SystemSnapshot` via `mpsc` channel. Non-blocking `try_recv()` in UI. Unit tests verify snapshot delivery and monotonic uptime.
- **App Structure** (`src/app.rs`): `PulseApp` with top panel (title, version, live uptime in monospace), left sidebar (180px, `ActiveTab::Memory` button), central panel (card with placeholder text).
- **Modular Main** (`src/main.rs`): Reduced to ≤35 lines — only eframe initialization and `run_native`.

### Changed
- `src/main.rs`: Refactored — moved theme to `theme.rs`, app logic to `app.rs`, worker to `core/`.
- `Cargo.toml`: Version bumped to `0.0.2`.

### Infrastructure
- All 13 unit tests passing (theme: 6, app: 5, core: 2).
- Zero warnings on `cargo check`.

---

## [v0.0.1] - 2026-11-02

### Added
- **Project initialization**: Git repository, `.gitignore` for Rust (target/, *.pdb, .env, .DS_Store)
- **README.md**: Bilingual documentation (Ukrainian / English) with project description, features, architecture overview, and run instructions
- **ARCHITECTURE.md**: Immutable laws of the project:
  - Module structure (≤150 lines/file, model/collector/view separation)
  - Documentation standard (English doc-comments for all public API)
  - Pinned dependency versions (eframe/egui 0.27.2, sysinfo 0.30.12, windows-sys 0.52.0)
  - "Obsidian HUD" design tokens (colors, radius, monospace requirement, traffic-light load colors)
  - Definition of Done (5 mandatory steps per version)
  - Forbidden patterns and project tree reference
- **Cargo.toml**: Manifest with pinned versions for 100% API compatibility:
  - `eframe = "0.27.2"`, `egui = "0.27.2"`, `egui_plot = "0.27.2"`, `egui_extras = "0.27.2"`
  - `sysinfo = "0.30.12"`
  - `windows-sys = { version = "0.52.0", features = ["Win32_System_ProcessStatus", "Win32_System_Threading", "Win32_Foundation"] }`
  - Release profile optimized (LTO, strip, panic=abort)
  - Dev dependencies for headless testing

### Infrastructure
- Project scaffold ready for CI/CD (fmt, clippy, check, test, release build)

---

## [Unreleased]

### Planned for v0.0.7
- **Processes Module — Model** (`src/modules/processes/model.rs`): `ProcessEntry`, `GroupKey`, `ProcessGroup`.

### Planned for v0.0.8
- **Processes Module — Collector** (`src/modules/processes/collector.rs`): `sysinfo::Processes` snapshot + Windows enrichment (exe path, session, cmdline).

### Planned for v0.0.9
- **Processes Module — View** (`src/modules/processes/view.rs`): `egui_extras::TableBuilder` with monospace PID, grouping, search/filter.

### Planned for v0.1.0 — Phase 1 Release: WinAPI RAM Purge
- Integration of all Memory module components.
- End-to-end test: RAM/Swap monitoring + cache cleanup working.
- `CHANGELOG.md` + `PULSE_AI_CONTEXT.md` updated.
- Git tag `v0.1.0`.

### Planned for v0.2.0 — CPU Module
- Per-core usage, frequency, temperature (MSR), `egui_plot` graphs, top consumers table.

### Planned for v0.3.0 — Storage I/O Module
- Disk read/write throughput, latency, queue depth, SMART health, partition usage.

### Planned for v0.4.0 — Network & Ping Radar
- Per-interface throughput, TCP/UDP stats, gamer ping, connection table.

### Planned for v0.5.0 — GPU DXGI & System
- DXGI GPU usage/VRAM/temp, process GPU attribution.

### Planned for v1.0.0 — Mini HUD Overlay
- Always-on-top overlay, hotkey toggle, customizable panels, export HTML/JSON.