# 🏗️ PULSE Architecture & Engineering Rules

PULSE (`pulse_manager`) is a native Windows 10/11 hardware telemetry HUD and process manager written in Rust (`Edition 2021`, MSRV `1.78`) using `eframe` / `egui` (`0.27.2`).

## 1. Core Architectural Principles

1. **Decoupled Two-Thread Telemetry Model**:
   - **UI Thread (`src/app.rs`, `src/ui/`)**: Renders immediate-mode `egui` widgets and handles user input. Never calls blocking Win32 or `sysinfo` APIs directly. Holds `Arc<SystemSnapshot>` for zero-copy per-frame access. Early-returns on minimized viewport (`request_repaint_after(1000 ms)`).
   - **Background Worker Thread (`src/core/worker.rs`)**: Runs a fixed 1 Hz (`1000 ms`) polling loop, collects Win32 and process metrics, computes per-PID deltas, and sends `SystemSnapshot` payloads over **bounded** `std::sync::mpsc::sync_channel(2)` (capacity 2). Uses `try_send` to skip snapshots on slow UI and cleanly breaks on receiver disconnect.
2. **1 Hz Process Table Cache**:
   - Filtered, grouped (`[Nx]`), and sorted process rows are cached in `MemoryTableState` keyed by a **64-bit dynamic telemetry signature** (`ram_used_bytes`, `ram_available_bytes`, `swap_used_bytes`, `ram_cached_bytes`, rolling hash of per-process `delta_bytes_per_sec / used_bytes`). Cached rows are stored as `Arc<[ProcessMemoryItem]>` so 60 FPS frames perform zero row or `String` clones (only an atomic `Arc` refcount increment).
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
- `src/modules/memory/` — RAM/Swap collector, Win32 kernel pool FFI (`K32GetPerformanceInfo`), 3-segment composition bar, spike blackbox, delta tracker (`Δ RAM/s`), formatters, and virtualized `TableBuilder` process views. Table cache key uses `search_hash: u64` + `uptime_secs`; 1 Hz `cached_format_uptime` allocates only on uptime change.
- `src/modules/{cpu,storage,network,system}/` — Planned subsystem modules scheduled across Phases 2–5.
- `src/ui/` — Root panel layout (`Top` -> `Bottom` -> `Left` -> `Central`), Obsidian theme tokens, top chrome header, sidebar navigation cards, and status bar.
- `src/i18n/` — English UI string dictionary and native Win32 OS locale badge (`GetUserDefaultUILanguage`).

## 4. Win32 FFI & Safety Invariants

- `#![deny(unsafe_code)]` is active at the crate root (`src/main.rs`). All Win32 FFI calls are wrapped in localized `#[allow(unsafe_code)]` blocks for: `K32GetPerformanceInfo`, `K32GetProcessMemoryInfo`, `K32EmptyWorkingSet`, `OpenProcess`, `IsProcessCritical`, `QueryFullProcessImageNameW`, `OpenProcessToken`, `GetTokenInformation`, `CloseHandle`, `TerminateProcess`, and `GetUserDefaultUILanguage`. Total `#[allow(unsafe_code)]` attributes in `src/`: **15**.
- Every Win32 FFI block is isolated in dedicated helper functions with explicit return-code checks and `std::mem::zeroed()` initialization.
- **TOCTOU Handle Identity Protection (`v0.0.16`)**: Both `trim_single_pid` and `terminate_process_group` enforce `IsProcessCritical` and `QueryFullProcessImageNameW` via `verify_handle_identity` before any mutating operation. The `actions.rs` module is engine-only in `v0.0.16` (UI wiring scheduled for `v0.1.0`).
- **Windows x64 CFG Virtual Memory Normalization**: On 64-bit Windows 11, Control Flow Guard reserves `2 TiB` of virtual address space per system process. PULSE normalizes `virtual_memory() >= (1_u64 << 40)` (`1 TiB`) to the process working set so grouped system services report realistic commit values.

## 5. Obsidian Theme Tokens (`src/theme.rs`)

| Token | Hex | Purpose |
| :--- | :--- | :--- |
| `BG_WINDOW` | `#0B0C10` | Main window background |
| `BG_CARD` | `#12141C` | Card/panel background |
| `ACCENT_CYAN` | `#00F0FF` | Primary telemetry accent, active tab indicator, optimal load (`< 60%`) |
| `ACCENT_AMBER` | `#FFB800` | Elevated load (`60%–85%`) and moderate growth (`▲ +M/s`) |
| `ACCENT_RED` | `#FF2A54` | Critical pressure (`>= 85%`), `▲ HOG` badge, spike alerts |