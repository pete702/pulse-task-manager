# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
- **`src/core/worker.rs`**: Worker now collects current process RAM via `sysinfo::Process::memory()` every 500ms. Unit tests updated.
- **`src/app.rs`**: Language switcher button, self-diagnosis badge, `request_repaint_after(500ms)` for economical updates. `SelfTestApp` for automated testing. `ActiveTab::label()` now uses i18n.
- **`src/theme.rs`**: Added `setup_fonts(ctx)` for Windows system fonts.
- **`Cargo.toml`**: Version bumped to `0.0.3`.

### Infrastructure
- All 23 unit tests passing (theme: 6, app: 7, core: 2, i18n: 8).
- Zero warnings on `cargo clippy --all-targets -- -D warnings`.
- Self-test passes both headless (CI) and live GUI modes.
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

### Planned for v0.0.4
- **Memory Module — Model** (`src/modules/memory/model.rs`): `PhysicalMemory`, `SwapMemory`, `CacheCleaner` structs with English doc-comments, unit tests.

### Planned for v0.0.5
- **Memory Module — Collector** (`src/modules/memory/collector.rs`): `sysinfo` integration for RAM/Swap, `EmptyWorkingSet` wrapper via `windows-sys`.

### Planned for v0.0.6
- **Memory Module — View** (`src/modules/memory/view.rs`): RAM/Swap cards with `card_frame()`, progress bars, `%` with `load_color()`, `egui_plot` history graphs, "Clear Cache" button.

### Planned for v0.0.7
- **Processes Module — Model** (`src/modules/processes/model.rs`): `ProcessEntry`, `GroupKey`, `ProcessGroup`.

### Planned for v0.0.8
- **Processes Module — Collector** (`src/modules/processes/collector.rs`): `sysinfo::Processes` snapshot + Windows enrichment (exe path, session, cmdline).

### Planned for v0.0.9
- **Processes Module — View** (`src/modules/processes/view.rs`): `egui_extras::TableBuilder` with monospace PID, grouping, search/filter.

### Planned for v0.1.0 — Memory Release
- Integration of all Memory module components.
- End-to-end test: RAM/Swap monitoring + cache cleanup working.
- `CHANGELOG.md` + `PULSE_AI_CONTEXT.md` updated.
- Git tag `v0.1.0`.