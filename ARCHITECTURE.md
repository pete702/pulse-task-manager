# 🏗️ PULSE Architecture & Engineering Rules

PULSE (`pulse_manager`) is a native Windows 10/11 hardware telemetry HUD and process manager written in Rust (`Edition 2021`, MSRV `1.78`) using `eframe` / `egui` (`0.27.2`).

## 1. Core Architectural Principles

1. **Decoupled Two-Thread Telemetry Model**:
   - **UI Thread (`src/app.rs`, `src/ui/`)**: Renders immediate-mode `egui` widgets and handles user input. Never calls blocking Win32 or `sysinfo` APIs directly.
   - **Background Worker Thread (`src/core/worker.rs`)**: Runs a fixed 1 Hz (`1000 ms`) polling loop, collects Win32 and process metrics, computes per-PID deltas, and sends `SystemSnapshot` payloads over `std::sync::mpsc::channel()` (while minimized, the UI thread skips rendering via `request_repaint_after(1000 ms)`).
2. **1 Hz Process Table Cache**:
   - Filtered, grouped (`[Nx]`), and sorted process rows are cached in `MemoryTableState` keyed by `(snapshot_id, search_query, sort_col, sort_asc, group_clones)`. Mouse movement at 60 FPS performs zero sorting or string allocations.
3. **Viewport Hibernation & Edge-Triggered Self-Trim**:
   - `K32EmptyWorkingSet` (`trim_own_working_set`) is called once at worker startup and once on each open-to-minimized window transition (`!was_minimized && is_minimized`) to release inactive pages without causing page-fault storms while the window is open.

## 2. Hard Modularity & Hygiene Guards

- **Source File Limit (`MAX_RS_FILE_LINES = 180`)**: Every `.rs` file inside `src/` must be `180` lines or fewer after `cargo fmt`, enforced by `src/core/selftest_tests.rs`.
- **Submodule Test Architecture**: Unit tests reside in sibling `*_tests.rs` submodules inside `src/`.
- **Language & Documentation Guards**:
  - `src/core/selftest_tests.rs` scans `src/**/*.rs` to enforce zero Cyrillic characters (`U+0400..=U+04FF`) and zero `Lang::Uk` tokens.
  - `tests/docs_guard.rs` verifies that `README.md`, `ARCHITECTURE.md`, and `CHANGELOG.md` stay under `120` lines / `8192` bytes, contain zero NUL bytes and zero Cyrillic characters, and exclude internal workspace paths or unverified marketing claims.

## 3. Module Boundaries (`src/`)

- `src/core/` — Background worker (`worker.rs`), `SystemSnapshot` channel (`mod.rs`), CLI `--self-test` harness (`selftest.rs`), and modularity/language guards (`selftest_tests.rs`).
- `src/modules/memory/` — RAM/Swap collector, Win32 kernel pool FFI (`K32GetPerformanceInfo`), 3-segment composition bar, spike blackbox, delta tracker (`Δ RAM/s`), formatters, and virtualized `TableBuilder` process views.
- `src/modules/{cpu,storage,network,system}/` — Subsystem modules scheduled across Phases 2–5.
- `src/ui/` — Root panel layout (`Top` -> `Bottom` -> `Left` -> `Central`), Obsidian theme tokens, top chrome header, sidebar navigation cards, and status bar.
- `src/i18n/` — English UI string dictionary and native Win32 OS locale badge (`GetUserDefaultUILanguage`).

## 4. Win32 FFI & Safety Invariants

- `#![deny(unsafe_op_in_unsafe_fn)]` and `#![warn(unsafe_code)]` are active at the crate root (`src/main.rs`).
- Every Win32 FFI block (`K32GetPerformanceInfo`, `K32EmptyWorkingSet`, `GetCurrentProcess`, `GetUserDefaultUILanguage`) is isolated in dedicated helper functions with explicit return-code checks and `std::mem::zeroed()` initialization.
- **Windows x64 CFG Virtual Memory Normalization**: On 64-bit Windows 11, Control Flow Guard reserves `2 TiB` of virtual address space per system process. PULSE normalizes `virtual_memory() >= (1_u64 << 40)` (`1 TiB`) to the process working set so grouped system services report realistic commit values.

## 5. Obsidian Theme Tokens (`src/ui/theme.rs`)

| Token | Hex | Purpose |
| :--- | :--- | :--- |
| `BG_PRIMARY` | `#0B0E14` | Deep Obsidian root background |
| `BG_PANEL` | `#121721` | Card and panel surface |
| `ACCENT_CYAN` | `#00F0FF` | Active tab indicator and primary telemetry accent |
| `MINT` | `#00E5A3` | Optimal load (`< 60%`) and negative memory delta (`▼ -M/s`) |
| `AMBER` | `#FFB800` | Elevated load (`60%–85%`) and moderate growth (`▲ +M/s`) |
| `CRIMSON` | `#FF2A54` | Critical pressure (`>= 85%`), `▲ HOG` badge, and spike alerts |