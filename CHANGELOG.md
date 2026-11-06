# Changelog

All notable PULSE changes are in this file.
Binary assets and checksums are published on [GitHub Releases](https://github.com/pete702/pulse-task-manager/releases).
Earlier 0.1.x entries are in CHANGELOG_ARCHIVE.md.

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

## [0.4.0] - 2026-11-06

- **Network `[04]` HUD**: esports grade banner (Tournament Ready/Stable/Bufferbloat/Packet Loss/Offline), DOWNLOAD/UPLOAD/PING/JITTER/LOSS KPIs, clickable adapter cards, dual-mode traffic/latency chart, sidebar `Mbps`/`ms` badge.

- **Telemetry Backend**: `NetworkCollector` over sysinfo counters with virtual-interface filtering, non-blocking TCP probe (`1.1.1.1:53` / `8.8.8.8:53`, 800 ms) with EWMA jitter and windowed loss, `ipconfig /flushdns` action.

- **Version bump to 0.4.0**.

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
