# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
  - Documentation standard (Ukrainian doc-comments for all public API)
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

### Planned for v0.0.3
- **Memory Module — Model** (`src/modules/memory/model.rs`): `PhysicalMemory`, `SwapMemory`, `CacheCleaner` structs with Ukrainian doc-comments, unit tests.

### Planned for v0.0.4
- **Memory Module — Collector** (`src/modules/memory/collector.rs`): `sysinfo` integration for RAM/Swap, `EmptyWorkingSet` wrapper via `windows-sys`.

### Planned for v0.0.5
- **Memory Module — View** (`src/modules/memory/view.rs`): RAM/Swap cards with `card_frame()`, progress bars, `%` with `load_color()`, `egui_plot` history graphs, "Clear Cache" button.

### Planned for v0.0.6
- **Processes Module — Model** (`src/modules/processes/model.rs`): `ProcessEntry`, `GroupKey`, `ProcessGroup`.

### Planned for v0.0.7
- **Processes Module — Collector** (`src/modules/processes/collector.rs`): `sysinfo::Processes` snapshot + Windows enrichment (exe path, session, cmdline).

### Planned for v0.0.8
- **Processes Module — View** (`src/modules/processes/view.rs`): `egui_extras::TableBuilder` with monospace PID, grouping, search/filter.

### Planned for v0.1.0 — Memory Release
- Integration of all Memory module components.
- End-to-end test: RAM/Swap monitoring + cache cleanup working.
- `CHANGELOG.md` + `PULSE_AI_CONTEXT.md` updated.
- Git tag `v0.1.0`.