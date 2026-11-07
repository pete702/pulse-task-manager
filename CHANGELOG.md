# Changelog

All notable PULSE changes are in this file.
Binary assets and checksums are published on [GitHub Releases](https://github.com/pete702/pulse-task-manager/releases).
Earlier 0.1.x entries are in CHANGELOG_ARCHIVE.md.

## [0.4.12] - 2026-03-31

- **Speedtest & Disk I/O Fixes**: network collector uses `total_received`/`total_transmitted` for true plateau; storage sums every `\\.\X:` per physical disk with `dt>0.0` guard; CPU cumulative I/O totals with `drive_letter` from `exe()`.
- **Universal 1-2-5 Auto-Scaler**: `nice_ceiling` (raw peak ×1.15, 1/2/2.5/5 decades, no cap) with `GB/s`/`Gbps` units; raw points ≤300, LTTB preserves peak; 52px gutter, labels at `left-6`.
- **Show All & UI Declutter**: Network H4 keeps `None` (no `first()` fallback); 4-col KPIs (no brackets); storage cards stacked header, `DISK` omitted for `Unknown`; inspector full-width activity.
- **Test Pruning**: removed 5 legacy tests, added `nice_ceiling` live test, `docs_guard` stable at 4.
- **Version bump to 0.4.12**.

## [0.4.11] - 2026-03-31

- **Y-Axis Units**: all 4 ticks share `Gbps`/`GB/s` vs `Mbps`/`MB/s` via `is_giga`.
- **Chart Cleanup**: `CEILINGS` removed, Storage uses `lttb_segments`, live `nice_ceiling` tested.
- **Version bump to 0.4.11**.

## [0.4.10] - 2026-03-31

- **Disk Sum**: sums every volume per disk (`dt > 0.0`, `cur >= prev`); CPU uses totals plus `drive_letter` from `exe()`.
- **Chart Gutter**: 1-2-5 ceiling (`nice_ceiling`, no cap) with `GB/s`/`Gbps`, raw <=300, 52px gutter.
- **Show All**: keeps `None`, 4-column KPIs, no brackets, cards stacked, full width.
- **Version bump to 0.4.10**.

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

## [0.4.3] - 2026-03-08

- **P0 Worker Fixes**: resize-trim removed (minimize plus 60-minute timer only), first-sample disk guard zeroes lifetime even when young, fallback rows sync CPU and memory together, delta tracker caps creation queries at 8 per scan, `OnceLock<String>` locale with `Arc<str>` uptime and cached status lines.
- **Network Audit Fixes**: `None` ping gaps never fake `0.0`, per-adapter chart scope with `[ SCOPE | Show All ]`, single adapter baseline with reused scratch map, persistent probe thread with `:443` fallback, traffic-aware grade above `0.25 Mbps`, real DNS flush states, row-major cards, responsive KPIs, LTTB to 300 points.
- **Storage Audit Fixes**: spike KPI shows process, peak, age plus PID/timestamp/R-W tooltip, ABA-safe `Arc` consumer cache with last-scan age, unified `[ HEALTHY/HIGH/LOW N% ]` badge, `3+2` narrow KPIs, LTTB to 300 points.
- **Version bump to 0.4.3**.


