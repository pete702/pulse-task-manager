# Changelog Archive (0.1.x)

Archived entries from 0.1.x. Recent 0.2.x entries live in CHANGELOG.md.

## [0.4.17] - 2026-03-31

- **Strict 1 Hz Worker Cadence**: `set_worker_paused` unparks only on `true` to `false` transition; loop enforces 950 ms minimum between ticks so 960 samples span 16 minutes.
- **Memory Full-Window Chart**: capacity 960, LTTB 920, monotonic `push` guard, always anchors `[-span, 0]`, stable `memory_load_plot` ID and forced `set_plot_bounds`.
- **Memory Table Stability**: deterministic name/pid tie-break in all sorts and clones, `TrimGuard` plus `spawn` bool so `TRIM_RUNNING` never sticks, fixed 280x18 status slot, strict `creation_matches` PID-reuse check.
- **Version bump to 0.4.17**.

## [0.4.16] - 2026-03-31

- **Full-Window CPU Chart**: history holds 960 samples, `build_windowed_series` always anchors to `[-span, 0]` even with 10s history, no stub clipping.
- **Forced Plot Bounds**: `set_plot_bounds([-span, 0], [0, 100])` every frame; zoom/drag/scroll stay disabled.
- **Single Clock**: `CpuHistory::push` drops duplicate timestamps; worker freezes `timestamp_s` on `< 450 ms` reuse so ingest never pushes stale samples.
- **Version bump to 0.4.16**.

## [0.4.15] - 2026-03-31

- **Pause-Safe Clock & CPU Guard**: `paused_accum` excludes pause time from `timestamp_s`; CPU refresh gated to `>= 450 ms` to drop short unpark spikes.
- **Chart Stability**: CPU LTTB trigger `900` to `920` (avoids 901 -> 900 shimmer); stable `cpu_load_plot` ID.
- **Table Determinism**: CPU quantized to 0.1% tenths, ties by `to_ascii_lowercase` name then pid in all modes + clones; no `rows[0]` auto-select, `preserve_selection` clears to `None`.
- **Table Layout**: constant `cpu_proc_table` ID, stable `4096.0 * 1024.0 * 1024.0` RAM reference, reserved `280x18` status slot.
- **Version bump to 0.4.15**.

## [0.4.14] - 2026-03-31

- **Worker Pause & Adaptive Probe**: `WorkerConfig.paused` skips all `collect_*` and waits 1000 ms; probe sleeps 250 ms steps (8x fast, 40x slow) via `FAST_PROBE_MODE`.
- **Peak WS & Win32 Input**: lazy query budget 8 to 64 per tick, `None` stays uncached for retry; `counters.cb` initialized before `K32GetProcessMemoryInfo`.
- **Heap Path Buffers**: `[0u16; 32767]` and `[0u16; 1024]` replaced with `vec![0u16; 4096]` on worker threads.
- **Purge RAII & Top-64**: `PurgeGuard` drops `PURGE_RUNNING` with `SeqCst`; only top 64 with `ram_bytes >= 10 MiB` sent to thread, `Purging RAM...` shown.
- **Bounded Maps & Disk Lookup**: `per_adapter`/`per_disk` capped at 32 keys; `group_physical_disks` finds groups via `iter_mut().find`.
- **Atomic Save**: serialized to `state.cfg.tmp` then `rename`, fallback to `write`.
- **Version bump to 0.4.14**.

## [0.4.11] - 2026-03-31

- **Y-Axis Units**: all 4 ticks share `Gbps`/`GB/s` vs `Mbps`/`MB/s` via `is_giga`.
- **Chart Cleanup**: `CEILINGS` removed, Storage uses `lttb_segments`, live `nice_ceiling` tested.
- **Version bump to 0.4.11**.

## [0.4.10] - 2026-03-31

- **Disk Sum**: sums every volume per disk (`dt > 0.0`, `cur >= prev`); CPU uses totals plus `drive_letter` from `exe()`.
- **Chart Gutter**: 1-2-5 ceiling (`nice_ceiling`, no cap) with `GB/s`/`Gbps`, raw <=300, 52px gutter.
- **Show All**: keeps `None`, 4-column KPIs, no brackets, cards stacked, full width.
- **Version bump to 0.4.10**.

## [0.4.12] - 2026-03-31

- **Speedtest & Disk I/O Fixes**: network collector uses `total_received`/`total_transmitted` for true plateau; storage sums every `\\.\X:` per physical disk with `dt>0.0` guard; CPU cumulative I/O totals with `drive_letter` from `exe()`.
- **Universal 1-2-5 Auto-Scaler**: `nice_ceiling` (raw peak x1.15, 1/2/2.5/5 decades, no cap) with `GB/s`/`Gbps` units; raw points <=300, LTTB preserves peak; 52px gutter, labels at `left-6`.
- **Show All & UI Declutter**: Network H4 keeps `None` (no `first()` fallback); 4-col KPIs (no brackets); storage cards stacked header, `DISK` omitted for `Unknown`; inspector full-width activity.
- **Test Pruning**: removed 5 legacy tests, added `nice_ceiling` live test, `docs_guard` stable at 4.
- **Version bump to 0.4.12**.

## [0.4.9] - 2026-03-31

- **Network Speedtest Plateau Fix**: `collector` now uses `total_received()` / `total_transmitted()` so steady 90 Mbps downloads show a true plateau and accurate `session_rx_gib` / `session_tx_gib`; steady throughput test added.
- **Chart HUD Cleanup**: Y-axis scale labels moved to the left (`rect.left() + 6.0` with dark pill background), compact top-left split legends (`RX / TX` / `Ping / Jitter`), active mode button uses dim Cyan fill with Cyan stroke, window buttons iterate `[900, 300, 60]` to read `1m  5m  15m`.
- **Sidebar Badges + Startup Trim**: `[03] Storage` always shows `{:.1} MB/s`; `[04] Network` always shows `{:.1} Mbps` (`{:.2} Gbps` at >= 1000.0); one-time `run_trim()` on `startup_frames == 10`.
- **Version bump to 0.4.9**.

## [0.4.8] - 2026-03-08

- **CI Delta Tracker Fix**: deterministic `track_with_resolver` with invalid PIDs (`u32::MAX`) so `twelve_new_pids_defer_then_resolve` no longer depends on live OS processes; worker timeout `5s -> 15s`.
- **Per-Disk I/O Fallback**: `aggregate_io_by_disk` attributes process I/O per physical disk, filling missing `0.0` hardware cases so Disk cards and scoped charts stay live.
- **Show All Disks Sync**: `sel_initial` re-read before cards fixes write-back, `Escape` clears Storage/Network scope, `clear_selected` test added.
- **Instant EcoQoS**: free `set_eco_cached` primes worker cache via pending map plus `request_process_scan` on success.
- **Version bump to 0.4.8**.

## [0.4.7] - 2026-03-08

- **Physical Disk Grouping**: `[03] Storage` renders Physical Disk Cards (`DISK N - SSD`) with live per-disk `R: / W: MB/s` (highlighted when active) and partitioned usage bars; clicking a disk or partition scopes chart, KPIs, and process filter to that disk.
- **Per-Disk I/O**: `collector` computes per-disk deltas via `prev_per_disk` so every volume carries its disk live `read_mb_s` / `write_mb_s`; `history` keeps scoped per-disk series with `Scope: Disk N (C:\, D:\) [ Show All Disks ]` reset.
- **CI Clippy Fix**: `#![allow(clippy::uninlined_format_args)]` in `main.rs` and `docs_guard.rs` so newer toolchains pass with `-D warnings` on `windows-latest`.
- **Version bump to 0.4.7**.

## [0.4.6] - 2026-03-08

- **Disk IOCTL Fix**: `dedupe_disk_mounts` keeps one mount per `physical_disk` so `DISK_PERFORMANCE` (per-disk counters) is not double-counted and duplicate volume handles are not opened every tick.
- **Cache Budget**: `fill_eco_qos` sets PID 0/4 to `Some(false)` without consuming the 12-query Win32 budget; `exe_cache` keeps half of 2048 entries instead of clearing to avoid syscall spikes.
- **Test Suite**: `actions_tests` consolidated to 2 table-driven tests; `latency_probe_tests` no longer spawns `ipconfig`; 5 worker sleep tests merged into 1 single-cycle test.
- **Version bump to 0.4.6**.

## [0.4.5] - 2026-03-08

- **Storage Physical Mapping**: `physical_disk.rs` maps volumes to `Disk 0` via `IOCTL_STORAGE_GET_DEVICE_NUMBER`, hardware 1s rates via `IOCTL_DISK_PERFORMANCE`, per-drive `[C:]` attribution with `[ All Drives ]` filter, and 950ms time-throttle live chart.
- **Network 25G + Split**: ceilings to 25 Gbps with Gbps bars, adaptive tier bars, and true `ChartMode::Split` side-by-side traffic plus ping/jitter.
- **EcoQoS + Perf**: real `is_process_eco_qos` detector with 12-query budget, zero-alloc `contains_ci`/`pid_contains`, PID-safe `(pid, start_time)` keys, `Ctrl+1..6` tabs, sidebar keyboard nav, health pill with storage/network.
- **Version bump to 0.4.5**.

## [0.4.4] - 2026-03-08

- **Audit Fixes P1-P3**: `[ Show All ]` read-back before cards, `OnceLock<String>` version, stateful `seen_pids` guard incl. tick-2 test, `Deferred` retry budget with 12-PID test, RAII probe guard plus clean disable, true `NAN` latency gaps, startup trim removed, ABA storage key with read/write bits.
- **Table Performance**: `Ctrl+F` `request_focus`, quick pills `[ All | Active CPU/Disk | RAM > 200 MB | EcoQoS ]`, `query_lower` once plus filter before formatting, hover `exe_path` cache, heat bars only (no per-cell tactical ticks).
- **Persistence**: new keys `storage_window_secs`, `network_window_secs`, `network_chart_mode`, `network_probe_enabled`, `process_quick_filter` with 500ms debounce and instant on-exit flush.
- **Version bump to 0.4.4**.

## [0.4.0] - 2026-11-06

- **Network `[04]` HUD**: esports grade banner (Tournament Ready/Stable/Bufferbloat/Packet Loss/Offline), DOWNLOAD/UPLOAD/PING/JITTER/LOSS KPIs, clickable adapter cards, dual-mode traffic/latency chart, sidebar `Mbps`/`ms` badge.

- **Telemetry Backend**: `NetworkCollector` over sysinfo counters with virtual-interface filtering, non-blocking TCP probe (`1.1.1.1:53` / `8.8.8.8:53`, 800 ms) with EWMA jitter and windowed loss, `ipconfig /flushdns` action.

- **Version bump to 0.4.0**.

## [0.4.3] - 2026-03-08

- **P0 Worker Fixes**: resize-trim removed (minimize plus 60-minute timer only), first-sample disk guard zeroes lifetime even when young, fallback rows sync CPU and memory together, delta tracker caps creation queries at 8 per scan, `OnceLock<String>` locale with `Arc<str>` uptime and cached status lines.
- **Network Audit Fixes**: `None` ping gaps never fake `0.0`, per-adapter chart scope with `[ SCOPE | Show All ]`, single adapter baseline with reused scratch map, persistent probe thread with `:443` fallback, traffic-aware grade above `0.25 Mbps`, real DNS flush states, row-major cards, responsive KPIs, LTTB to 300 points.
- **Storage Audit Fixes**: spike KPI shows process, peak, age plus PID/timestamp/R-W tooltip, ABA-safe `Arc` consumer cache with last-scan age, unified `[ HEALTHY/HIGH/LOW N% ]` badge, `3+2` narrow KPIs, LTTB to 300 points.
- **Version bump to 0.4.3**.

## [0.4.2] - 2026-11-06

- **Network Audit Fixes**: `prev` now holds raw lifetime counters so `rx/tx Mbps` reflect true intervals; timeouts clear stale ping, 3 consecutive failures flip grade to OFFLINE/PACKET_LOSS fast; traffic history pushes even with probe OFF/probing.
- **100% Width Layouts**: Network KPIs render in 6 fixed columns, chart mode `[ TRAFFIC | PING & JITTER ]` buttons sit in the header, single adapter spans full width with activity bar; Storage KPIs use 5 fixed columns with truncated TOP I/O.
- **Storage Selection Works**: media badges + filesystem restored, 3-tier health (`LOW SPACE`/`HIGH`/`HEALTHY`), row-major volume pairs, bottom split with selected-volume details and cached deterministic process table.
- **Version bump to 0.4.2**.

## [0.4.1] - 2026-11-06

- **Chart-First Ergonomics**: Storage and Network place the HUD chart right below the KPI strip; volume and adapter cards render in 2-column grids; Storage consumers are grouped by process name.
- **UX Fixes**: sidebar split into `sidebar_badges.rs` with short `[03] Storage` / `[04] Network` titles, disks sorted `C:\` first, honest critical rule for mid-size drives, Npcap/WFP adapters filtered.
- **Network Correctness**: session traffic baseline since PULSE start, startup shows `PROBING...` instead of false OFFLINE, disabled probe shows `--`, `0.0 ms` fakes removed, `DNS Flushed OK` feedback for 3 seconds.
- **Version bump to 0.4.1**.

## [0.3.4] - 2026-11-06

- **Storage Audit Closure**: stepped Y-ceiling up to 10000 MB/s (no NVMe clipping), `HLine` tied to spike threshold, one history point per process scan, focus-free progress bars.
- **Sortable DISK I/O Column**: `[00] Processes` gains a `DISK I/O` header/cell (dim/Cyan/Amber) with summed clone-group I/O; table sorting extracted to `table_sort.rs`.
- **Version bump to 0.3.4**.

## [0.3.3] - 2026-11-06

- **Storage Chart Parity**: timestamped cached history in minutes-ago coordinates, anchored HUD chart with `HLine 80`, fixed Y grid, and corner brackets.
- **Audit Fixes**: background ingestion on every snapshot, spike expires after 120 ticks, fresh-scan spike gate, finite clamps, small USB sticks no longer flagged critical.
- **Version bump to 0.3.3**.

## [0.3.2] - 2026-11-06

- **Clickable Volumes & Inspector**: volume cards are click-selectable (`[ SELECTED ]` cyan border); inspector shows mount/label/media/filesystem/health, GiB breakdown, and top 6 disk consumers.
- **First-Tick Guard**: first-seen processes with lifetime I/O are zeroed to kill startup multi-GB/s spikes; sidebar Storage badge shows live `MB/s` or volume count.
- **Version bump to 0.3.2**.

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
