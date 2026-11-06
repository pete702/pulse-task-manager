# Changelog Archive (0.1.x)

Archived entries from 0.1.x. Recent 0.2.x entries live in CHANGELOG.md.

## [0.3.1] - 2026-11-06

- **Live Storage HUD**: `[03] Storage` tab with spike banner, READ/WRITE KPIs, volume cards, and 1m/5m/15m throughput chart.
- **True Disk Rate**: per-process MB/s uses delta since last scan, not lifetime average; `StorageCollector` wired into worker.
- **Version bump to 0.3.1**.

## [0.3.0] - 2026-11-06

- **Storage Backend**: `[03] Storage` model with volume badges, critical-space flags, I/O spike events, and 900-sample history.
- **Single-Pass Disk I/O**: per-process read/write MB/s from `disk_usage()` in the existing scan; no second process scan.
- **Spike Capture**: aggregated I/O at or above 80 MB/s records the top process with HH:MM:SS stamp; version bump to 0.3.0.

## [0.2.10] - 2026-11-06

- **15m Chart Stability**: LTTB threshold 300 to 900 so 1m/5m/15m keep exact 1-second points; empty-history guards; 120ms warmup sleep replaces synthetic 0% startup ticks; X labels unified inside.
- **Fluid Core Grid**: Per-core grid stretches 100% width with up to 14 columns and fluid tile width.
- **True EcoQoS & Local Time**: Power Throttling EcoQoS via SetProcessInformation; unified PID gate with user-shell processes; local HH:MM:SS timestamps.
- **Audit Fixes**: Mutex poison-safe locks, no dropped LatestSlot frames, 5m default windows, on-exit flush, Ctrl+F/Esc route to Processes, stale i18n cleanup.
- **Version bump to 0.2.10**.

## [0.2.9] - 2026-11-06

- **True Win32 Locale**: Replaced hardcoded UK/EN shortcut with `GetUserDefaultLocaleName` into a stack buffer, uppercased ASCII cache; non-Windows fallback is EN-US.
- **System Telemetry Top Bar**: Top bar now shows `[ CPU: X% | RAM: X / XX GiB (XX%) ]` plus uptime and health pill; PULSE self-RAM/UI/locale moved to status bar and top-bar hover.
- **Smart Resizable Columns**: Processes numeric columns use compact `initial` widths with strict ranges and `resizable` dividers; `PROCESS NAME` remainder starts at 180px so long names stay readable.
- **App State Persistence**: `%LOCALAPPDATA%\PULSE\state.cfg` stores tab, grouping, density, sort, chart windows, grid view, and window size; change detection saves on toggle and resize.
- **Sci-Fi HUD Accents**: Corner brackets on cards, charts, and active sidebar; 5 percent tick segments plus 2px leading cap on main bars; dual-tone top scanline.
- **Version bump to 0.2.9**.

## [0.2.8] - 2026-11-06

- **Top Bar Always Self-RAM**: `[ PULSE: XX.X MB | UI: X.XX ms ]` now cached across ticks/tabs and shown on all tabs; short locale `[ LOCALE: UK-UA ]` with full details on hover; min window `860x560`.
- **Memory Responsive**: chart header hides duplicate RAM text below 960px, Y labels inset 24px, top cards stack below 880px with plain-English tooltips for Commit Charge / Kernel Pools.
- **CPU Responsive**: time labels moved to strip below plot (no line collision), bottleneck `Balanced` shortened, per-core grid uses dynamic `cols = width/84` clamp 4..10 + vertical scroll, idle cores dimmed.
- **Processes Hierarchy**: `PROCESS NAME` remainder `at_least 130` clip, CPU colors idle slate / white / cyan / amber thresholds, RAM heat bar alpha 62 + right cap, bars scaled to 4096 MiB cap, sidebar roadmap badges `v0.3/v0.4/v0.5`.
- **Version bump to 0.2.8**.

## [0.2.7] - 2026-11-06

- **Zero-Alloc UI**: Cached sidebar badges (RAM/CPU/count), cached version/locale, tooltip built only on hover, heat bars skipped under 0.5 pct or off-screen.
- **Health Pill**: Top bar combines Three Walls and compute bottleneck into one pill with detailed hover breakdown.
- **Density Toggle**: `[ Compact ]` / `[ Comfort ]` (22/28px rows) in Processes toolbar; sidebar badges 14px.
- **Open-Core Docs**: README Open-Core & Privacy note, ARCH monetization boundaries, SECURITY offline guarantee for core and future Pro, all synced to v0.2.7.
- **Version bump to 0.2.7**.

## [0.2.6] - 2026-11-06

- **Minimized 0 FPS**: `is_minimized` check moved above hotkeys; 30s safety repaint kept, worker 5s poll kept, restore triggers wakeup.
- **Full Linux Gating**: Removed fake `if_same_then_else` allow; gated `OnceLock` in i18n, kernel metrics import in memory collector, and Win32 CPU frequency/brand paths with non-Windows stubs.
- **Release Checksums**: `release.yml` now generates and publishes the `.exe.sha256` file alongside the binary.
- **Docs Sync**: Updated THIRD_PARTY_NOTICES, SECURITY, ARCHITECTURE, README; created CHANGELOG_ARCHIVE for 0.1.x and refreshed PULSE_AI_CONTEXT.
- **Version bump to 0.2.6**.

## [0.2.5] - 2026-11-06

- **Perf: Single Sysinfo Process Refresh**: `collect_cpu` uses `with_cpu_usage()`; `collect_process_memory` reuses the CPU pass when `sys.processes()` is already filled, removing the second 200+ process scan per tick.
- **Perf: Filter Before Formatting**: CPU table filters raw `RawProcessCpuRow` before `item_from_raw_cpu`, so excluded rows perform zero string allocations.
- **Perf & UI: Cached Elevation Flag**: Toolbar uses `cached_elevated()` (`OnceLock<bool>`), 0 Win32 token syscalls per frame.
- **Narrow-Window Fixes**: Row 2 actions render left-to-right with truncated status on the right; table scales fixed columns below 640px width to prevent clipping.
- **Honest Purge Status**: `[ Purge All RAM ]` reports trimmed row count; single trim reports success or block reason.
- **Version bump to 0.2.5**.

## [0.2.4] - 2026-11-05

- **Process Toolbar Complete**: `[00] Processes` Row 1 shows privilege badge `[ ADMIN ]` (cyan) or `[ USER ]` (dim) via `is_app_elevated()`; global `[ Purge All RAM ]` button trims working sets across all visible rows and refreshes scan.
- **Two-Step Kill Confirmation**: Row 2 `[ End Task ]` arms a 3-second confirmation window; shows `[ Cancel ]` + `[ Confirm Kill (3s) ]` (red); auto-expires on timeout or selection change; `confirm_kill` state in `CpuTableState`.
- **Open File Location**: New `[ Open Location ]` button (primary) in toolbar and context menu using `open_process_file_location(pid)`.
- **Context Menu Selection Reset**: `show_row_menu()` now clears `selected_pid`, `selected_name`, and `confirm_kill` on successful termination.
- **Clean Formatting**: Removed `rustfmt skip` from `table_state.rs::preserve_selection`; simplified `contains_ci` to single-expression sliding window; all `.rs` files format cleanly under 280 lines.
- **Version bump to 0.2.4**.

## [0.2.3] - 2026-11-05

- **Memory Cockpit Wrapping Fixed**: `top_cards.rs` - all captions ("AVAILABLE COMMIT", "KERNEL POOLS", etc.) and card titles now use `.wrap(false)`; narrowed "KERNEL POOLS" value to `"{:.2} GiB ({:.0}/{:.0}M)"`; legend square uses ASCII-safe `"\u{25A0}"`. No vertical letter columns on narrow windows.
- **Chart X-Axis Labels Restored**: `chart_view.rs` - `plot_height` now computed AFTER `render_header()` via `ui.available_height()`, so bottom time labels (5m..0s) fit 100% inside the card without clipping.
- **Memory Spike Blackbox Restored**: `chart_chrome.rs` - `render_header` accepts `latest_spike: Option<&str>`; right side shows `[ ! SPIKE: <msg> ]` (red) when armed, or `[ BLACKBOX: ARMED ]` (emerald) when idle.
- **Version bump to 0.2.3**.

## [0.2.2] - 2026-11-05

- **Processor Cockpit Overlap Fixed**: Replaced broken `ui.set_width()` horizontal layout in bottleneck card with `egui::Grid` (3 columns, 20px spacing); added "Logical thread bottleneck detector" subtitle so left/right cards have identical header height and vertical bar alignment.
- **Narrow-Window Wrapping Eliminated**: Both CPU cards now use `.wrap(false)` / `.truncate(true)` on headers, subtitles, and all Grid metric labels; text never wraps into vertical letter columns on narrow windows.
- **Startup 100% Chart Spike Fixed**: Worker clamps `total_pct`, `hottest_core_pct`, `hottest_core_idx` to zero for the first two ticks (`tick < 2`), removing the fake vertical wall on the CPU chart.
- **Instant Worker Unpark**: Removed unused local `CpuHistory` (UI owns history from `CpuSnapshot`); `thread::park_timeout()` already provides instant unpark wakeup.
- **Version bump to 0.2.2**.

## [0.1.26] - 2026-11-05

- **CPU Processes 0.0% Fixed**: Removed a second normalization by logical core count; clone groups again sum CPU and RAM.
- **Instant CPU Activity & Stable Count**: Startup ticks 0 and 1 force process scans, and `process_count` is retained between scans.
- **Clone Grouping & ASCII Polish**: Grouping is on by default; pause and sort indicators use ASCII glyphs.
- **CPU Actions & Layout**: Right-click QoS actions, full-width table, compact top cards, and a collision-free header badge.
- **Memory Resize Fixed**: Overview scrolls and the chart receives a stable computed height.

## [0.1.25] - 2026-11-05

- **2-Second CPU Tab Lag Fixed**: Cached brand/base MHz, immediate first scan, and O(1) `scan_generation` invalidation.
- **Pause, Search & Sorting**: Pause freezes rows; `Ctrl+F` focuses search, `Esc` clears it; every column header sorts.
- **Chart & Header Polish**: Removed duplicate time-window buttons, fixed chart clipping, and replaced the Unicode clear glyph.

## [0.1.24] - 2026-11-05

- **CPU Process List Flickering Fixed**: Retained the last scan between ticks.
- **Live Frequency & Warmup**: Win32 `CurrentMhz` with PDH fallback and two-pass CPU warmup.
- **Virtualized Process Table & Cockpit**: Table row virtualization, clone grouping, dual-series chart, and per-core grid.
- **Version bump to 0.1.24**.

## [0.1.23] - 2026-11-05

- **Processor Telemetry Core**: CPU snapshot with total/per-core loads, hottest thread, frequency, and brand.
- **900-Second History**: Dual-series ring buffer with LTTB windows and live sidebar CPU badge.
- **Single-Pass Process CPU**: Per-process CPU collected in the same sysinfo pass as memory.
- **Version bump to 0.1.23**.

## [0.1.22] - 2026-11-05

- **OpenGL & Chart Polish**: Reduced driver memory, unified grid, and positive time labels.

## [0.1.21] - 2026-11-05

- **Chart & Group Fixes**: Clamped window edges, summed clone RAM, and stable delta timestamps.
- **Clean Shutdown**: Worker joins in under 1 ms; telemetry constants moved to `constants.rs`.

## [0.1.20] - 2026-11-05

- **Telemetry & Worker Speedups**: Smooth precision bars, on-demand exe paths, LTTB windows, and instant `unpark()` wakeups.

## [0.2.0] - 2026-11-05

- **Master `[00] Processes` Tab**: New sidebar entry with live count; Memory and Processor are full-width cockpits with `[ View Processes -> ]`.
- **Instant Sorting**: `CpuTableState::set_sort()` reorders in memory and never blanks the table.
- **No 1-2s Blank Table**: Worker retains last CPU process list on every tick.
- **Unified Table**: Responsive `TableBuilder`, striped rows, heat bars, `[32x]` clones, header sorting.
- **Action Toolbar**: `[ End Task ]`, `[ EcoQoS ]`, `[ Trim RAM ]`, pause/resume, status message.
- **Rich Context Menu**: Trim, EcoQoS, Open Location, Terminate, Copy Name & PID.
- **Compute Alert**: `[ ^ COMPUTE BOTTLENECK ]` amber top chrome at 85% unless memory wall active.
- **UI Polish**: Semantic buttons, sort arrows, zero right gap, scrollbar-free Memory cockpit.

## [0.2.1] - 2026-11-05

- **Process Selection Persistence**: Added `selected_name` tracking to `CpuTableState`; selection now survives 3-second scan rebuilds even when clone-group representative PID shifts or rows are re-filtered/sorted.
- **Toolbar Layout Fix**: Split toolbar into two rows (filter controls + action buttons) to eliminate text overlap with `[ Trim RAM ]`; shortened counter to `"64 / 196 rows"`; reduced search field width.
- **10s DELTA Column Clipping Fixed**: Changed to exact-width columns (`PID=62`, `CLONES=66`, `CPU %=84`, `RAM=114`, `DELTA=98`) with `remainder()` only for `PROCESS NAME`; all 6 columns now fit 100% inside table width.
- **Version bump to 0.2.1**.

## [0.2.12] - 2026-11-06

- **Responsive Table**: width bucket `("proc_table", width)` rebuilds the grid on resize; columns scale proportionally with draggable dividers.
- **Delta Spike Blackbox**: culprit is the fastest-growing process, fallback to largest RAM; 4 unit tests cover threshold, empty, growth, and fallback.
- **Non-Blocking Trim/Purge**: `TRIM_RUNNING`/`PURGE_RUNNING` guards; Win32 path resolution runs in worker threads.
- **Safety & Dead Code**: targeted unsafe allows only on Win32 FFI; removed write-only minimize flag and empty snapshot list; version bump to 0.2.12.

## [0.2.11] - 2026-11-06

- **Fluid Processes Table**: `PROCESS NAME` is pure `Column::remainder()` (96px); numeric ranges scale with width, table always fills window.
- **Linux CI & MSRV 1.88**: Added `lint-linux` job; removed unused `url` pin (6 direct dependencies); MSRV 1.88.
- **Instant Unpark & Debounce**: `WORKER_THREAD` unpark on `request_process_scan()`; `state.cfg` writes deferred while pointer down; `total_cmp` sorts.
- **Identity & Perf**: On-demand 3-point path query; async Purge All RAM; true process count; zero-alloc PID search; 12 new CPU tests.
- **Version bump to 0.2.11**.
