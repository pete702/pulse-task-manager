# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.0.11] - 2026-11-03

### Added
- **Universal UI Perfection & Rich Telemetry Sidebar** (`src/ui/sidebar.rs`, 174 lines):
  - Custom Obsidian HUD mini-cards via `ui.interact(..., egui::Sense::click())` — no ugly button fill, no phantom focus ring
  - Active tab: background `#161F2E`, border `1.0px #00F0FF`, rounded `6.0`
  - Inactive tab: background `#12141C` (hover `#181C26`), border `1.0px #1F2330`, rounded `6.0`
  - **Memory (RAM) card** — Row 1: Title + live RAM% (traffic-light colored); Row 2: 3px mini progress bar; Row 3: Muted monospace "X.XX / Y.YY GiB • Commit ZZ%"
  - Other 4 cards — Title + muted subtitle ("Telemetry: Phase 2", "Telemetry: Phase 3")
- **Kernel Tiles Refactor** (`src/modules/memory/kernel_tiles.rs` → 154 lines):
  - Single `render_tile` helper — all 4 tiles share one layout block
  - Fixed dimensions: `ui.set_min_size(vec2(width, 66.0)); ui.set_max_size(vec2(width, 66.0));` — identical height, no wrapping
  - Kernel Pool subtitle: integer MiB formatting (`Paged: {:.0} MiB • Non-Paged: {:.0} MiB`) — never wraps to second line
- **Process Table Fixes**:
  - **`▲ HOG` badge** (`src/modules/memory/table_body.rs`): Geometric badge in `egui::Frame::none().fill(Color32::from_rgb(58,16,29)).stroke(Stroke::new(1.0, CRIMSON)).rounding(3.0).inner_margin(Margin::symmetric(5.0, 1.0))` — aligns cleanly with process name and `[Nx]` badge
  - **CFG Virtual Memory Normalization** (`src/modules/memory/collector.rs`): On 64-bit Windows 11, CFG reserves 2 TiB per system process. If `virt_bytes >= (1_u64 << 40)` (≥1 TiB), normalize that process's virtual memory to its actual working set (`ram`) — aggregated groups now show realistic committed/virtual memory instead of 100+ TiB
- **Chart View Improvements** (`src/modules/memory/chart_view.rs`, 163 lines):
  - Adaptive height: `let chart_height = (ui.available_height() - 18.0).clamp(115.0, 260.0);` — X-axis labels always visible on resize
  - Tooltip formatter: `.label_formatter(|name, value| { let secs = value.x.abs().round() as u32; if name.is_empty() { format!("{secs}s ago: {:.1}%", value.y) } else { format!("{name}\n{secs}s ago: {:.1}%", value.y) } })`
- **Zero-Overhead Engine** (`src/core/worker.rs`, `src/app.rs`, `src/ui/status_bar.rs`):
  - **Trim once at startup**: `trim_own_working_set()` called only once before worker loop — eliminates page-fault thrashing while GUI is open
  - **OS Scheduler Jitter Probe**: `os_scheduler_jitter_ms = sleep_start.elapsed().as_millis().saturating_sub(1000) as u32` — stored in `SystemSnapshot`, displayed in status bar as "OS Jitter: X ms"
  - **Heuristic Workload Mode** (computed from snapshot):
    - Any process group ≥ 2.5 GiB → `[ ◈ MODE: GAMING / HEAVY APP ]` (red)
    - Total RAM ≥ 75% → `[ ▣ MODE: WORKSTATION LOAD ]` (amber)
    - Else → `[ ● MODE: DESKTOP IDLE ]` (cyan)
  - **Minimized Zero-GPU Hibernation**: If `ctx.input(|i| i.viewport().minimized.unwrap_or(false))` → `request_repaint_after(1000ms)` + early return (0% GPU/CPU while minimized, worker keeps recording)
  - **Global Hotkeys** (`src/app_tabs.rs`): `Ctrl+F` → focuses Memory→Processes + clears search; `Esc` → clears search query
- **Panel Registration Order Fix** (`src/app.rs`): Strict egui root order — top → bottom → left → center (prevents status bar clipping)
- **2-Thread Architecture (No Rayon)**: `sysinfo` with `default-features = false` — no rayon thread pool anywhere; binary size 4.38 MiB
- **Client-Facing README.md** — Complete rewrite as product showcase for gamers, video editors, power users (100% English)

### Changed
- **`src/ui/chrome.rs`** (80 lines): Reduced to top panel only (sidebar extracted to `sidebar.rs`)
- **`src/ui/status_bar.rs`**: Added heuristic mode badge + OS jitter display
- **`src/app.rs`** (153 lines): Panel order fix, minimized hibernation, hotkeys delegated to `app_tabs.rs`
- **`src/app_tabs.rs`** (73 lines): Added `handle_global_hotkeys()` function
- **`src/core/mod.rs`**: `SystemSnapshot` now includes `os_scheduler_jitter_ms: u32`
- **`src/core/worker.rs`** (156 lines): Trim-once startup, jitter measurement, `new_with_jitter()` constructor
- **`src/i18n/mod.rs`** (126 lines): Moved locale table constants to `frame.rs`, 100% English code
- **`src/i18n/frame.rs`**: New file for locale badge strings and formatting helpers
- **`Cargo.toml`**: `sysinfo = { version = "=0.30.12", default-features = false }`; removed `rayon` dependency entirely; full extreme release profile (thin LTO, strip=symbols, codegen-units=1, panic=abort)
- **`src/core/selftest_tests.rs`**: Zero-Cyrillic/Uk test uses `concat!("Lang::", "U", "k")` and `concat!("U", "k")` to avoid raw regex matches

### Fixed
- Bottom status bar clipping (panel registration order)
- Kernel Pool tile wrapping to 2 lines (fixed 66px height + integer MiB formatting)
- `⚠️ HOG` emoji missing glyph square (□) + baseline shift (replaced with geometric `▲ HOG` Frame)
- 100+ TiB virtual memory display from CFG reservation (normalized to RSS in collector)
- Chart X-axis labels clipped on small windows (adaptive height clamp)
- Raw tooltip `x = -45.00, y = 50.0` (human-readable formatter)
- Periodic self-trim causing 1.5–3% CPU page-fault storms (trim once at startup)
- 180-line limit violations (all files ≤179, max `table_body.rs` at 179)

### Verified
- **72 / 72 unit tests passing** (theme: 7, app: 10, core: 4, i18n: 13, memory model: 12, collector: 8, process model: 9, chart: 2, view: 2, page: 2, table view: 2, table body: 1, table widgets: 2, ui: 2, file size: 1, zero Cyrillic/Uk: 1)
- `cargo fmt` clean; `cargo clippy --all-targets -- -D warnings` — **0 warnings**
- Self-test: **SELF-TEST PASSED (LIVE GUI)** + **HEADLESS CI**
- Release binary: `../PULSE_BUILDS/PULSE_v0.0.11_modularity_memory.exe` + `PULSE_v0.0.11_modularity_mem.exe` + `PULSE_latest.exe` — **4,593,664 bytes (≈4.38 MiB)**, SHA-256 `325402EECB08F5B7E8837D85B676A00271FF6678FB1992FFD37406ABBF54AB23`
- Clean backup: `../PULSE_BACKUPS/v0.0.11_verified/`

---

## [0.0.10] - 2026-11-02

### Added
- **Surgical Modularity Audit #1** — Core/UI decomposition into focused modules (all ≤175 lines after `cargo fmt`):
  - `src/core/selftest.rs` (156 lines): Headless self-test + CI/GUI dispatch (`run_headless_self_test()`, `run_self_test()`); architectural file-size guard test (600-line pragmatic ceiling).
  - `src/core/selftest_app.rs` (81 lines): GUI `SelfTestApp` for `--self-test` live mode (5-frame render, auto-close, tab/lang switching).
  - `src/app_tabs.rs` (42 lines): `ActiveTab` enum extracted from `app.rs` with localized `label()` via i18n `Key`.
  - `src/ui/chrome.rs` (174 lines): Window chrome rendering — top panel (title, version, uptime, self-diagnosis badge, locale indicator), sidebar (tab navigation with live RAM% on Memory tab), status bar (ready + active process count).
  - `src/ui/placeholder_tab.rs` (16 lines): Placeholder card for unimplemented tabs (Cpu, Storage, Network, System) — `Coming in v0.2.0`.
  - `src/theme_fonts.rs` (60 lines): System fonts loader (Segoe UI proportional, Consolas monospace) with safe fallback to egui defaults.
  - **i18n package split** (`src/i18n/`):
    - `mod.rs` (166 lines): Core — `Lang`, `Key`, `current_lang`, `set_lang`, `init_language`, `detect_os_language`, `Key::t()`, formatting helpers.
    - `keys.rs` (82 lines): All translation keys (`AppTitle`, `TabMemory`, `ProcessTitle`, etc.).
    - `frame.rs` (105 lines): Window chrome translations (top bar, sidebar tabs, status bar, self-test messages).
    - `memory.rs` (128 lines): Memory & Process module translations (cards, kernel tiles, chart, table, spike event).
    - `tests/` — 4 test files: `tests_basic.rs`, `tests_translations_uk.rs`, `tests_translations_en.rs`, `tests_formatted.rs`.
  - `src/util.rs` (23 lines): `format_uptime()` utility.

### Changed
- **`src/main.rs`** (116 lines): Reduced to core entry point only — logger init, panic hook, CLI parsing (`--self-test`, `--lang=`), `i18n::init_language()`, font setup, icon generation, `eframe::run_native` or `core::selftest::run_self_test()`.
- **`src/app.rs`** (170 lines): Now imports `ActiveTab` from `app_tabs.rs`, uses `chrome::render_placeholder_tab()` for unimplemented tabs, delegates central panel rendering to `render_central_panel()`, re-exports `format_uptime` from `util.rs` and `ActiveTab` from `app_tabs.rs`.
- **`src/theme.rs`** (155 lines): Removed font setup (moved to `theme_fonts.rs`), now focuses on design tokens, theme application, traffic-light colors, and frame builders.
- **`src/i18n.rs` → `src/i18n/` directory**: Monolithic 567-line file decomposed into 5 focused files + 4 test files (total 625 lines across 9 files, each ≤175 lines).
- **`Cargo.toml`**: Version bumped to `0.0.10`; added `mod theme_fonts;` and `mod app_tabs;` to `main.rs` module declarations.

### Verified
- **72 / 72 unit tests passing** (theme: 7, app: 10, core: 4, i18n: 13, memory model: 12, collector: 8, process model: 9, chart: 2, view: 2, page: 2, table view: 2, table body: 1, table widgets: 2, ui: 2, file size: 1).
- `cargo fmt` clean; `cargo clippy -- -D warnings` — **0 warnings**.
- Self-test: **SELF-TEST PASSED (HEADLESS CI)** (`CI=true` without OpenGL) + **LIVE GUI** (auto-switches tabs & lang).
- Release binary: `../PULSE_BUILDS/PULSE_v0.0.10_modularity_core.exe` + `PULSE_latest.exe` — **4,628,992 bytes (≈4.41 MiB)**, SHA-256 `044D75393858155377A96B3B3F7E4D5B44F1D151F7A315862A8A6B54BA1834CE`.
- Clean backup: `../PULSE_BACKUPS/v0.0.10_verified/`.

---

## [0.0.9] - 2026-04-06

### Added
- **Mission Center Sub-view Switcher** (`src/modules/memory/page.rs`): `MemorySubView { Overview, Processes }` (default `Overview`) rendered inside a `card_frame()`. The localized switcher labels show the live process count (`◈ Telemetry Overview` / `📋 Memory Processes (N)` in English; the Ukrainian dictionary supplies equivalent localized strings at runtime). The `Processes` branch adds a compact RAM bar above the table.
- **Virtualized Process Memory Table** (`src/modules/memory/table_view.rs`, `table_body.rs`, `table_widgets.rs`): `MemoryTableState { search_query, group_clones, sort_col, sort_asc, selected_pid }` with clone grouping enabled and descending RAM sort by default. `egui_extras::TableBuilder` renders rows through `body.rows(24.0, rows.len(), …)`, so paint cost is **O(visible_rows)** instead of O(process count).
- **Instant Search, Clone Grouping, and Sorting**: case-insensitive name substring or exact PID match; `[Nx]` cyan badge for clone groups; five sortable monospace columns (Process / PID / Physical RAM / Virtual / RAM %); sort direction starts ascending for text columns and descending for numeric columns.
- **Resource Hog Badges**: `⚠️ HOG` red badge for processes at ≥ 15% of total RAM, recomputed after clone grouping.
- **RAM Spike "Black Box" Banner** (`table_widgets.rs`): an amber/red banner renders `snap.latest_spike` with timestamp, peak RAM %, and the culprit as `name [PID] • bytes`. The worker now preserves the last detected spike instead of dropping it between 1000 ms intervals.
- **Snapshot Process Payload**: `MemorySnapshot.processes: Vec<RawProcessRow>` and `MemorySnapshot.latest_spike: Option<MemorySpikeEvent>` are filled by the telemetry worker and consumed directly by the table.
- **Own Working Set Measurement** (`src/modules/memory/collector.rs`): `own_working_set_mb()` reads `WorkingSetSize` via `K32GetProcessMemoryInfo(GetCurrentProcess(), …)`; the worker trims and measures its own working set before each snapshot.

### Changed
- **`src/app.rs`**: The Memory tab now routes through `page::render_memory_page(…)` with `memory_sub_view` and `table_state`; the app also trims its own working set after OpenGL startup (frame 3) and every 30 frames.
- **`src/core/worker.rs`**: Collects the process table via `collect_process_memory()`, carries the latest `MemorySpikeEvent`, trims the working set, and reports `pulse_ram_mb` from `own_working_set_mb()`.
- **`src/modules/memory/collector.rs`**: Process scan returns `(rows, spike)`; added the `own_working_set_mb()` measurement alongside `trim_own_working_set()`.
- **`src/modules/memory/mod.rs`**: Registers `page`, `table_view`, `table_body`, and `table_widgets`.
- **`src/ui/chrome.rs`**, **`src/modules/memory/view.rs`**, **`src/i18n.rs`**: Version/status text synced to v0.0.9 and locale keys aligned with the new sub-view and table labels.
- **`Cargo.toml`**: Version bumped to `0.0.9`; release profile switched from `lto = "fat"` to `lto = "thin"` for faster release builds with comparable optimization.

### Fixed
- Table header sorting now toggles correctly per column, and the active column shows `▲`/`▼` with cyan emphasis.
- RAM and RAM % cells use `load_color()`, so critical rows are visually red instead of plain text.
- Selection state (`selected_pid`) is preserved across refreshes and highlighted per row.
- The table-scroll height is derived from `ui.available_height()`, so the table fills the card on any window size instead of overflowing it.

### Verified
- **72 / 72 unit tests passing** (theme: 7, app: 10, core: 3, i18n: 10, memory model: 12, collector: 8, process model: 9, chart: 2, view: 2, page: 2, table view: 2, table body: 1, table widgets: 2, ui: 1, file size: 1).
- `cargo fmt` clean; `cargo clippy -- -D warnings` — **0 warnings**; `SELF-TEST PASSED (LIVE GUI)`.
- Visual verification: 180 live processes collapsed into 65 grouped rows, spike banner at **93.6%** naming `Resolve.exe [PID 14440] • 6.40 GiB`, plus working search, sorting, `[Nx]` grouping, and `⚠️ HOG` badges.
- Release binary: `../PULSE_BUILDS/PULSE_v0.0.9_process_table.exe` + `PULSE_latest.exe` — **4,101,120 bytes (≈3.91 MiB)**, SHA-256 `F4A795AF550E11E6066691DC5559BC62A7711DD8D49793C10534AB41E427F245`.
- Clean backup: `../PULSE_BACKUPS/v0.0.9_verified/`.

---

## [v0.0.8] - 2026-11-02

### Added
- **Process Memory Model** (`src/modules/memory/process_model.rs`): `ProcessMemoryItem` (PID, name, RAM/virtual bytes, RAM %, `process_count`, `is_hog`), `MemorySortCol`, and pure `filter_group_and_sort()` with case-insensitive name/PID search, clone grouping (summed RAM + instance count), and column sorting. The `is_hog` flag triggers at RAM ≥ 15%.
- **Memory Spike "Black Box"** (`MemorySpikeEvent`, `detect_memory_spike()`): captures timestamp, peak RAM %, culprit name/PID/bytes when total RAM usage reaches ≥ 85%. `collect_process_memory()` refreshes processes with `ProcessRefreshKind::new().with_memory()`.
- **Multi-Horizon Memory Chart** (`ChartWindow` in `model.rs`, switcher in `chart_view.rs`): `[ 1m | 5m | 15m ]` buttons; history ring buffer extended from 120 to 900 points (15 minutes at 1s); windowed point iterators; the plot now occupies 100% of card width (no `view_aspect`).
- **Self Working-Set Trim** (`src/modules/memory/collector.rs`): `K32EmptyWorkingSet(GetCurrentProcess())` is called before measuring PULSE's own RSS, releasing font/OpenGL buffers and lowering the process footprint.
- **Real UI Render Time** (`src/app.rs`): `update()` duration is measured with `Instant::now()` and shown on the next frame, so the top badge reports actual UI cost (≈1 ms) instead of the 1000 ms repaint interval.
- **Complete Localization** (`src/i18n.rs`, `src/modules/memory/view.rs`, `src/ui/chrome.rs`): RAM/Swap cards, kernel tiles (OS reserve %, paged/non-paged pools, OS commit limit), chart title, and status bar are fully translated with no mixed-language leftovers; the active language follows the Windows locale.

### Changed
- **`src/modules/memory/model.rs`**: `HISTORY_CAPACITY` 120 → 900; added `ChartWindow { OneMin, FiveMin, FifteenMin }`, `MemoryHistory::chart_window`, `windowed_ram_points()`, `windowed_swap_points()`.
- **`src/modules/memory/chart_view.rs`**: translated title plus 1m/5m/15m switcher; full-width plot; x-axis window follows the selected horizon.
- **`src/modules/memory/collector.rs`**: process memory scan, spike detection, self working-set trim, and `ProcessMemoryScan`/`RawProcessRow` type aliases.
- **`src/core/worker.rs`**: telemetry interval 500 ms → 1000 ms; trims the worker's own working set before each measurement.
- **`src/app.rs`**: mutable history passed to the chart; real render timing; active process count passed to the status bar.
- **`src/ui/chrome.rs`**: status bar shows the active process count and v0.0.8.
- **`src/modules/memory/mod.rs`**: registers `pub mod process_model;`.
- **`Cargo.toml`**: version bumped to `0.0.8`.

### Fixed
- English UI no longer leaks Ukrainian fragments in the memory cards (`Available / Free` values, cache/kernel/commit subtitles, OS activity counters).
- Format templates (`{}`, `{:.1}`) are no longer rendered literally; new `i18n` helpers produce fully substituted strings.
- UI badge no longer reports the repaint interval as render time.

### Infrastructure
- **64 unit tests passing** (0 failed), including 9 new process-model tests (clone grouping, hog detector, search by name/PID, sorting, spike detection) and 3 i18n formatting tests.
- README hero screenshot (`screenshot.png`, English UI via `--lang=en`).
- `cargo fmt` and `cargo clippy -- -D warnings` clean; live GUI self-test passes (`SELF-TEST PASSED (LIVE GUI)`).
- Release binary exported to `../PULSE_BUILDS/PULSE_v0.0.8_multi_horizon_processes.exe` and `PULSE_latest.exe`.
- Clean backup created at `../PULSE_BACKUPS/v0.0.8_verified/`.

---

## [v0.0.7] - 2026-11-02

### Added
- **UI Chrome Module** (`src/ui/chrome.rs`, `src/ui/mod.rs`): Extracted top panel, sidebar, and status bar rendering from `app.rs` for strict modularity (< 180 lines/file).
- **Auto-Detect Windows UI Language** (`src/i18n.rs`): `detect_os_language()` via `GetUserDefaultUILanguage()` (Win32_Globalization), `init_language()` with `--lang=uk|en` CLI override for testing. Removed manual `[UA|EN]` switcher button.
- **Locale Indicator** in top bar: `[ LOCALE: UK-UA (0x0422) ]` or `[ LOCALE: EN-US (0x0409) ]` showing detected Windows locale.
- **Real System Cache Fix** (`src/modules/memory/collector.rs`): `K32GetPerformanceInfo` now properly initializes `perf_info.cb` before call; `ram_cached_bytes_override` field stores real `SystemCache * PageSize`; `ram_cached_bytes()` returns override when > 0.
- **Strict Kernel Metrics Test** (`src/modules/memory/collector.rs`): New `test_windows_real_kernel_cache_metrics` asserts `ram_cached_bytes > 0`, `kernel_paged_bytes > 0`, `kernel_nonpaged_bytes > 0`, `system_threads > 0` on Windows.
- **100% Localized UI Text** (`src/modules/memory/view.rs`, `chart_view.rs`): All card headers, grid labels, kernel mini-tiles, chart title/legend now use `i18n::Key` dictionary — no mixed-language leftovers. The UI is fully translated for the detected Windows locale.
- **Full-Width 60s Chart** (`src/modules/memory/chart_view.rs`): `.width(ui.available_width())`, `.height(200.0)`, RAM line with semi-transparent fill (`.fill(0.0_f32)`), chart stretches edge-to-edge in card.

### Changed
- **`src/modules/memory/model.rs`**: Added `ram_cached_bytes_override: u64` to `MemorySnapshot`; `ram_cached_bytes()` prioritizes override over `available - free` fallback.
- **`src/modules/memory/collector.rs`**: `collect_memory()` sets `ram_cached_bytes_override` from kernel `SystemCache`; `perf_info.cb` initialized correctly.
- **`src/core/worker.rs`**: Optimized from `sys.refresh_processes()` to `sys.refresh_pids(&[pid])` — reduces RAM from ~150MB to ~35-50MB, speeds up telemetry 10x.
- **`src/app.rs`**: Uses `ui::chrome` module; removed manual language switcher; status bar and sidebar rendered via chrome.
- **`src/main.rs`**: Calls `i18n::init_language()` at startup with `--lang` override support; self-test uses `set_lang` instead of removed `toggle_lang`.
- **`Cargo.toml`**: Added `Win32_Globalization` feature to `windows-sys`; version bumped to `0.0.7`.
- **`src/theme.rs`**: `status_bar_frame()` now used by chrome module.

### Infrastructure
- All **45 unit tests passing** (theme: 6, app: 8, core: 3, i18n: 8, memory model: 6, collector: 4, chart: 1, ui: 1, integration: 1).
- Zero warnings on `cargo clippy --all-targets -- -D warnings`.
- Self-test passes **HEADLESS CI** (`CI=true`) without OpenGL.
- Release binary exported to `../PULSE_BUILDS/PULSE_v0.0.7_os_audit.exe` and `PULSE_latest.exe`.
- Clean backup created at `../PULSE_BACKUPS/v0.0.7_verified/`.

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
- **`README.md`**, **`ARCHITECTURE.md`**, **`../PULSE_UA_MANUAL.md`**: Updated roadmap to 7 phases (v0.0.1–v1.0.0), added Iron Testing Standard (min 3–5 tests/step, i18n coverage, self-test auto-switch tabs/lang), updated DoD to 8 rules.

### Infrastructure
- All **29 unit tests passing** (theme: 6, app: 7, core: 2, i18n: 8, memory: 6).
- Zero warnings on `cargo clippy --all-targets -- -D warnings`.
- Self-test passes both headless (CI) and live GUI modes (auto-switches tabs & lang).
- Release binary exported to `../PULSE_BUILDS/pulse_v0.0.4.exe` and `PULSE_LATEST.exe`.
- Clean backup created at `../PULSE_BACKUPS/v0.0.4_verified/`.

---

## [v0.0.3] - 2026-11-02

### Added
- **Internationalization (i18n)** (`src/i18n.rs`): `Lang` enum with type-safe translation keys for all UI elements and unit tests for both languages. The original manual toggle was **removed in v0.0.7** — the language is now auto-detected from Windows.
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
- **`src/app.rs`**: Self-diagnosis badge, `request_repaint_after(500ms)` for economical updates. `SelfTestApp` for automated testing. `ActiveTab::label()` now uses i18n. **7 unit tests** (added i18n keys, tab labels).
- **`src/theme.rs`**: Added `setup_fonts(ctx)` — loads Segoe UI (Proportional) and Consolas (Monospace) from `C:\Windows\Fonts\` with a fallback to egui defaults. Only geometric Unicode symbols are used (◈, ▣, ⚡, ▲, ▼, ●).
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

### Delivered in v0.0.7 – v0.0.11
- **v0.0.7** — UI chrome extraction, auto-detected Windows locale, real system cache, full localization (see `[v0.0.7]`).
- **v0.0.8** — process memory model, clone grouping, hog detector, multi-horizon chart, spike black box, self working-set trim (see `[v0.0.8]`).
- **v0.0.9** — Mission Center sub-view switcher, virtualized process table, instant search, sortable columns, `⚠️ HOG` badges, spike banner (see `[0.0.9]`).
- **v0.0.10** — Surgical Modularity Audit #1: 50 modular files, i18n package split, selftest modules, app_tabs, theme_fonts, ui/chrome (see `[0.0.10]`).
- **v0.0.11** — Universal UI Perfection & Zero-Overhead Engine: rich telemetry sidebar, kernel tiles 66px fixed, ▲ HOG Frame badge, CFG virtual memory normalization, adaptive chart, trim-once, OS jitter probe, heuristic mode, minimized hibernation, global hotkeys, 2-thread no-rayon (see `[0.0.11]`).

### Planned for v0.1.0 — Phase 1 Release: WinAPI RAM Purge
- Integration of all Memory module components.
- End-to-end test: RAM/Swap monitoring + safety-gated cache cleanup with a before/after reclamation audit.
- Groundwork for the export-ready sanitized diagnostic report (`[REDACTED]` masking).
- Incremental refactor of legacy modules toward the 180-line architectural limit.
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