# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

### Planned for v0.1.0
- **Core App**: `src/main.rs` + `src/app.rs` with eframe event loop
- **Theme System**: `src/theme.rs` implementing Obsidian HUD tokens
- **Memory Module**: `src/modules/memory/` — RAM/Swap monitoring, cache cleanup (EmptyWorkingSet)
- **Processes Module**: `src/modules/processes/` — process table, grouping, search
- **System Module**: `src/modules/system/` — CPU, GPU, Disk, Network overview
- **Utilities**: `src/utils/formatting.rs` (bytes_to_human, monospace helpers), `windows_api.rs` wrappers