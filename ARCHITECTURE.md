# 🏗️ PULSE Architecture & Engineering Rules

PULSE (`pulse_manager`) is a native Windows 10/11 hardware telemetry HUD and process manager written in Rust (`Edition 2021`, `rust-version = "1.78"`) using `eframe` / `egui` (`0.27.2`). Current release: **v0.1.2**.

## 1. Core Architectural Principles

1. **Decoupled Two-Thread Telemetry Model**:
   - **UI Thread (`src/app.rs`, `src/ui/`)**: Renders immediate-mode `egui` widgets and handles user input. Never calls blocking Win32 or `sysinfo` APIs directly. Holds `Arc<SystemSnapshot>` for zero-copy per-frame access. When the window is idle or minimized, repaints are throttled to 1 Hz via `request_repaint_after(1000 ms)`.
   - **Background Worker Thread (`src/core/worker.rs`)**: Runs decoupled polling: **system RAM at 1 Hz** (`snapshot_id`) and **full process scans at 0.5 Hz** (`process_scan_id` every 2 s) in normal mode. While minimized, process scans pause and `Arc<Vec<RawProcessRow>>` is shared zero-copy; on wake or any user action (Trim/End Task/Open Location) an immediate full process scan runs. Every tick stores the latest snapshot in `LatestSlot` with **overflow tracking** (`AtomicU64` counter); if the bounded `std::sync::mpsc::sync_channel(2)` is full, the UI reads the newer slot instead. Worker exits cleanly on receiver disconnect.
2. **O(1) Process Table & Selection Cache**:
   - Filtered, grouped (`[Nx]`), and sorted process rows are cached in `MemoryTableState` keyed by the authoritative **`process_scan_id`** (monotonic u64). When `scan_id` matches and filter/sort/group settings are unchanged, rebuild returns in `O(1)` without re-computing the telemetry signature. Cached rows are stored as `Arc<[ProcessMemoryItem]>` so 60 FPS frames perform zero row or `String` clones. Selection resolution (`SelectedGroupSummary` with `Vec<ProcessTarget>`) is also cached per `scan_id` so the action bar never iterates raw rows or allocates on every frame.
3. **Viewport Hibernation & Single Frame-#3 Self-Trim**:
   - `K32EmptyWorkingSet` (`trim_own_working_set`) is called exactly **once on frame #3 after GPU initialization** (post `eframe` first render) and never again unless explicitly triggered. The minimize transition no longer triggers a trim.
4. **Non-Blocking Actions & Cached Elevation**:
   - `Open Location` validates path synchronously (empty/NUL check) then spawns `explorer.exe` on a background thread so slow/network drives never block the UI thread.
   - `is_app_elevated()` uses `OnceLock<bool>` so `OpenProcessToken` / `GetTokenInformation` is queried at most once per process lifetime.

## 2. Hard Modularity & Hygiene Guards

- **Source File Limit (`MAX_RS_FILE_LINES = 180`)**: Every `.rs` file inside `src/` and `tests/` must be `180` lines or fewer after `cargo fmt`, enforced by `src/core/selftest_tests.rs` and `tests/docs_guard.rs`. Zero `rustfmt` skip attributes in the entire repository (forbidden token).
- **Submodule Test Architecture**: Unit tests reside in sibling `*_tests.rs` submodules inside `src/`.
- **Language & Documentation Guards**:
  - `src/core/selftest_tests.rs` scans `src/**/*.rs` to enforce zero Cyrillic characters (`U+0400..=U+04FF`) and zero `Lang::Uk` tokens.
  - `tests/docs_guard.rs` verifies that `README.md`, `ARCHITECTURE.md`, and `CHANGELOG.md` stay under `120` lines / `8192` bytes, contain zero NUL bytes and zero Cyrillic characters, and exclude internal workspace paths or unverified marketing claims.

## 3. Module Boundaries (`src/`)

- `src/core/` — Background worker (`worker.rs`), `SystemSnapshot` channel (`mod.rs`), CLI `--self-test` harness (`selftest.rs`), and modularity/language guards (`selftest_tests.rs`).
- `src/modules/memory/` — RAM/Swap collector, Win32 kernel pool FFI (`K32GetPerformanceInfo`), 3-segment composition bar, spike blackbox, delta tracker (`Δ RAM/s`), formatters, virtualized `TableBuilder` process views, non-blocking action bar (`action_bar.rs`), and PID Safety Gate (`safe_handle.rs`). Table cache key uses `search_hash: u64` + `uptime_secs`; 1 Hz `cached_format_uptime` allocates only on uptime change. `1m/5m/15m` chart selector with cached `PlotPoints` (`chart_view.rs`).
- `src/modules/{cpu,storage,network,system}/` — Planned subsystem modules scheduled across Phases 2–5.
- `src/ui/` — Root panel layout (`Top` -> `Bottom` -> `Left` -> `Central`), Obsidian theme tokens, top chrome header, sidebar navigation cards, status bar, and Three Walls bottleneck badge (`three_walls.rs`).
- `src/i18n/` — English UI string dictionary and native Win32 OS locale badge (`GetUserDefaultUILanguage`).

## 4. Win32 FFI & Safety Invariants

- `#![deny(unsafe_code)]` is active at the crate root (`src/main.rs`). All Win32 FFI calls are wrapped in localized `#[allow(unsafe_code)]` blocks for: `K32GetPerformanceInfo`, `K32GetProcessMemoryInfo`, `K32EmptyWorkingSet`, `OpenProcess`, `IsProcessCritical`, `QueryFullProcessImageNameW`, `OpenProcessToken`, `GetTokenInformation`, `CloseHandle`, `TerminateProcess`, and `GetUserDefaultUILanguage`. Total `#[allow(unsafe_code)]` attributes in `src/`: **15** (verified at v0.1.1).
- Every Win32 FFI block is isolated in dedicated helper functions with explicit return-code checks and `std::mem::zeroed()` initialization.
- **TOCTOU Handle Identity Protection**: `trim_single_pid` and `terminate_process_group` enforce `IsProcessCritical` and `QueryFullProcessImageNameW` via `verify_handle_identity` before any mutating operation. Both checks fail closed: a Win32 query failure or unresolvable image path blocks the mutation.
- **Windows x64 CFG Virtual Memory Normalization**: On 64-bit Windows 11, Control Flow Guard reserves `2 TiB` of virtual address space per system process. PULSE normalizes `virtual_memory() >= (1_u64 << 40)` (`1 TiB`) to the process working set so grouped system services report realistic commit values.

## 5. Three-Point Process Identity & Immutable Kill Confirmation

- **3-Point Verification** (`src/modules/memory/safe_handle.rs`, `process_target.rs`): Before any mutating action (`EmptyWorkingSet`, `TerminateProcess`), PULSE verifies process identity via:
  1. **Basename** — `QueryFullProcessImageNameW` stripped to filename.
  2. **Creation Time** — `GetProcessTimes` cached per `(pid, sysinfo_start_time)` in `ProcessDeltaTracker` (queried once per `(pid, start_time)` pair).
  3. **Full Path** — `QueryFullProcessImageNameW` full path, validated against `ProcessTarget.exe_path`.
  All three must match the frozen `ProcessTarget` captured at scan time; any failure fails closed (blocks the mutation).
- **Immutable `PendingKillConfirm`** (`src/modules/memory/action_bar.rs`): On first `[ End Task ]` click, the exact `Vec<ProcessTarget>` for the selected group is frozen into `PendingKillConfirm { group_key, targets, armed_at }`. The second click (within 3 s) sends this frozen list to `WorkerAction::TerminateSelected`. A newly spawned process with the same name after arming is **excluded** because the target list is immutable.

### v0.1.2 Audit & Performance Hardening
- `snapshot_id` (1 Hz) vs `process_scan_id` (0.5 Hz, minimized via `Arc`) — decoupled polling with zero-copy fast path.
- `kernel_metrics_valid` flag + `WallState::Unconfirmed` (`INITIALIZING TELEMETRY`) — honest Three Walls badge until real kernel metrics arrive.
- `os_scheduler_jitter_ms` measured as pure sleep overshoot, kept separate from `scan_duration_ms`.
- Non-blocking background-thread `Open Location` with `OnceLock` cached elevation check.
- 3-point `ProcessTarget` identity (`basename + creation_time + full_path`) with auto-canceling `PendingKillConfirm` on target drift.

## 6. Obsidian Theme Tokens (`src/theme.rs`)

| Token | Hex | Purpose |
| :--- | :--- | :--- |
| `BG_WINDOW` | `#0B0C10` | Main window background |
| `BG_CARD` | `#12141C` | Card/panel background |
| `ACCENT_CYAN` | `#00F0FF` | Primary telemetry accent, active tab indicator, optimal load (`< 60%`) |
| `ACCENT_AMBER` | `#FFB800` | Elevated load (`60%–85%`) and moderate growth (`▲ +M/s`) |
| `ACCENT_RED` | `#FF2A54` | Critical pressure (`>= 85%`), `▲ HOG` badge, spike alerts |