# 🏗️ PULSE Architecture & Engineering Rules

PULSE (`pulse_manager`) is a native Windows 10/11 hardware telemetry HUD and process manager written in Rust (`Edition 2021`, `rust-version = "1.78"`) using `eframe` / `egui` (`0.27.2`). Current release: **v0.1.14**.

## 1. Core Architectural Principles

1. **Decoupled Two-Thread Telemetry Model**:
   - **UI Thread** (`src/app.rs`, `src/ui/`): Renders `egui` widgets, handles input, never calls blocking Win32/`sysinfo`. Holds `Arc<SystemSnapshot>` for zero-copy per-frame access. Repaints throttled to 1 Hz when idle/minimized.
   - **Worker Thread** (`src/core/worker.rs`): **RAM at 1 Hz** (`snapshot_id`), **process scans at 0.5 Hz** (`process_scan_id` every 2 s). Minimized: scans pause, `Arc<Vec<RawProcessRow>>` shared zero-copy; wake/user action triggers immediate full scan. `LatestSlot` with `AtomicU64` overflow tracking; `sync_channel(2)` full → UI reads newer slot.

2. **O(1) Process Table & Selection Cache**:
   - Filtered/grouped/sorted rows cached in `MemoryTableState` keyed by `process_scan_id` (monotonic u64). Scan ID + filter/sort/group unchanged → rebuild in `O(1)`. Rows stored as `Arc<[ProcessMemoryItem]>` → 60 FPS frames zero clones. `SelectedGroupSummary` with `Vec<ProcessTarget>` also cached per `scan_id`.

3. **Viewport Hibernation & Single Frame-#3 Self-Trim**:
   - `K32EmptyWorkingSet` called **once on frame #3 after GPU init**, never again unless explicit. Minimize transition no longer triggers trim.

4. **Non-Blocking Actions & Cached Elevation**:
   - `Open Location`: sync path validation → `explorer.exe` on background thread.
   - `is_app_elevated()`: `OnceLock<bool>` → single Win32 query per process lifetime.

## 2. Hard Modularity & Hygiene Guards

- **Source File Limit** (`MAX_RS_FILE_LINES = 180`): Every `.rs` in `src/`/`tests/` ≤ 180 lines post `cargo fmt`, enforced by `selftest_tests.rs` and `docs_guard.rs`. Zero rustfmt skip attributes.
- **Submodule Test Architecture**: Unit tests in sibling `*_tests.rs` inside `src/`.
- **Language & Documentation Guards**:
  - `selftest_tests.rs`: scans `src/**/*.rs` for zero Cyrillic (`U+0400..=U+04FF`), zero `Lang::Uk`.
  - `docs_guard.rs`: `README.md`, `ARCHITECTURE.md`, `CHANGELOG.md` ≤ 120 lines / 8192 bytes, zero NUL, zero Cyrillic, no internal paths/unverified claims.

## 3. Module Boundaries (`src/`)

- `src/core/`: Worker (`worker.rs`), `SystemSnapshot` channel (`mod.rs`), CLI `--self-test` (`selftest.rs`), guards (`selftest_tests.rs`).
- `src/modules/memory/`: RAM/Swap collector, `K32GetPerformanceInfo` FFI, 3-segment bar, spike blackbox, `Δ RAM/s`, formatters, virtualized `TableBuilder`, action bar (`action_bar.rs`), PID Safety Gate (`safe_handle.rs`). Cache key: `search_hash: u64` + `uptime_secs`; 1 Hz `cached_format_uptime`; `1m/5m/15m` chart with cached `PlotPoints`.
- `src/modules/{cpu,storage,network,system}/`: Planned Phases 2–5.
- `src/ui/`: Root panel layout (`Top`→`Bottom`→`Left`→`Central`), theme, top chrome, sidebar, status bar, Three Walls badge (`three_walls.rs`).
- `src/i18n/`: English UI strings + Win32 OS locale badge (`GetUserDefaultUILanguage`).

## 4. Win32 FFI & Safety Invariants

- `#![deny(unsafe_code)]` at crate root. 15 localized `#[allow(unsafe_code)]` blocks for: `K32GetPerformanceInfo`, `K32GetProcessMemoryInfo`, `K32EmptyWorkingSet`, `OpenProcess`, `IsProcessCritical`, `QueryFullProcessImageNameW`, `OpenProcessToken`, `GetTokenInformation`, `CloseHandle`, `TerminateProcess`, `GetUserDefaultUILanguage` (verified v0.1.1).
- Every FFI block isolated in helper functions with return-code checks, `std::mem::zeroed()` init.
- **TOCTOU Handle Identity**: `trim_single_pid` / `terminate_process_group` enforce `IsProcessCritical` + `QueryFullProcessImageNameW` via `verify_handle_identity` before mutation. Fail closed.
- **Win11 x64 CFG Normalization**: `virtual_memory() >= 1 TiB` → normalized to working set for realistic commit on grouped system services.

## 5. Three-Point Identity & Immutable Kill Confirmation

- **3-Point Verification** (`safe_handle.rs`, `process_target.rs`):
  1. **Basename** — `QueryFullProcessImageNameW` stripped to filename.
  2. **Creation Time** — `GetProcessTimes` cached per `(pid, sysinfo_start_time)` in `ProcessDeltaTracker`.
  3. **Full Path** — `QueryFullProcessImageNameW` full path vs `ProcessTarget.exe_path`.
  All three must match frozen `ProcessTarget`; any failure fails closed.
- **Immutable `PendingKillConfirm`** (`action_bar.rs`): First `[ End Task ]` freezes `Vec<ProcessTarget>` → `PendingKillConfirm { group_key, targets, armed_at }`. Second click (3 s) sends frozen list. Same-name spawn after arming excluded.

### v0.1.2 Audit & Performance Hardening
- `snapshot_id` (1 Hz) vs `process_scan_id` (0.5 Hz, minimized via `Arc`) — decoupled zero-copy fast path.
- `kernel_metrics_valid` + `WallState::Unconfirmed` — honest Three Walls until kernel metrics arrive.
- `os_scheduler_jitter_ms` pure sleep overshoot, separate from `scan_duration_ms`.
- Background `Open Location` + `OnceLock` elevation.
- 3-point `ProcessTarget` with auto-canceling `PendingKillConfirm` on drift.

### v0.1.14 Phase 1 Complete
- Adaptive telemetry cadence: 3 s visible / 5 s background / 0 fps minimized with event-driven wakeup.
- Working-set trim on frame #3 and minimize transition; physical RAM 27–29 MB, 0% idle CPU.
- Memory UX polish: `[ Pause / Resume ]` toggle, context menu (Open Location / Terminate / Copy), double-click action, `Ctrl+F` / `Esc` / `[X]` search ergonomics, empty-state notice.
- Modular internals: worker split into `worker_io` / `worker_policy` / `worker_loop`; table body split into context-menu, sort, name-cell modules.
- 131 tests passing (127 unit + 4 docs-guard); Phase 1 complete, Phase 2 (CPU) next.

## 6. Obsidian Theme Tokens (`src/theme.rs`)

| Token | Hex | Purpose |
| :--- | :--- | :--- |
| `BG_WINDOW` | `#0B0C10` | Main window background |
| `BG_CARD` | `#12141C` | Card/panel background |
| `ACCENT_CYAN` | `#00F0FF` | Primary accent, active tab, optimal load (`< 60%`) |
| `ACCENT_AMBER` | `#FFB800` | Elevated load (`60–85%`), moderate growth (`▲ +M/s`) |
| `ACCENT_RED` | `#FF2A54` | Critical pressure (`≥ 85%`), `▲ HOG`, spike alerts |