# Changelog

All notable PULSE changes are in this file. Earlier 0.1.x entries are in CHANGELOG_ARCHIVE.md.

## [0.4.24] - 2026-03-31

- **Viewport Guard**: `last_viewport_size` only updates when rect exists -- no more `ZERO` shrink to 860x560; `set_minimized` encapsulation on worker.
- **Release Console**: `AttachConsole(u32::MAX)` on Windows for CLI flags, `args_os` lossy parsing, panic backtrace, `--self-test` help clarified.
- **Worker Tightening**: 950 ms floor via interval minus 50, `minimized_now` re-read before publish, explicit `collector_primed` latch, Builder spawn error resets `WORKER_ALIVE`, loop trimmed to 255 lines.
- **SelfTest Hardening**: shared `fail_selftest` helper, `started_at` 10 s timeout, theme re-applied only once, `serialize` NaN/Inf test.
- **Version bump to 0.4.24**.

## [0.4.23] - 2026-03-31

- **Ping Ladder**: `nice_ceiling` upgraded to 9-step ladder `[1.0,1.5,2.0,2.5,3.0,4.0,5.0,7.5,10.0]` with 5% tolerance; latency floor 10.0 to 5.0 -- 12 ms snaps to 15.0, 23 ms to 25.0, 95..105 Mbps stays at 100.0.
- **Name Interning**: `util::intern_name` shares one `Arc<str>` per process name across CPU and Memory collectors, capped at 512 entries.
- **Disk Handle Cache**: physical disk handles cached in `OnceLock` map (max 32), evicted and closed on `DeviceIoControl` failure -- no more per-tick `CreateFileW`/`CloseHandle`.
- **Test Polish**: uptime tests merged to table-driven, `test_colors_constants` removed, PID 0/4 handle tests merged, `is_app_elevated` determinism asserted; CI gets concurrency cancel and msrv reduced to `cargo check`.
- **Version bump to 0.4.23**.

## [0.4.22] - 2026-03-31

- **Chart Ceiling Fit**: removed `peak * 1.15` pre-scale and added 5% boundary tolerance in `nice_ceiling` -- 90..105 Mbps locks to 100.0, 900..1050 Mbps locks to 1000.0, no more ceiling jump to 200/2000.
- **Startup Unpark Race**: `request_process_scan()` now returns `bool` and clears `WORKER_THREAD` on exit; `TelemetryWorker` falls back to `thread_handle` unpark when the worker thread has not registered yet.
- **Core Cleanup**: `serialize()` guards non-finite window size, rename failure removes tmp, unused `headless` field dropped from `SelfTestApp`, duplicate `visit_dirs` merged into one helper.
- **Version bump to 0.4.22**.

## [0.4.21] - 2026-03-31

- **Ctrl+F Deadlock Fix**: `handle_global_hotkeys` snapshots keys outside `ctx.input` closure -- no `ctx.*`/`ui.*` calls while the input lock is held, eliminating the freeze on `Ctrl+F` and arrow navigation.
- **UI Shell 860px Fix**: top bar fixed at 36px height, compact `[ CPU: x% | RAM: x% ]` telemetry below 960px, status bar hides `Windows Kernel Monitor` and `OS Jitter` on narrow widths, zero-copy `Arc<str>` labels.
- **Sidebar Stabilization**: Storage/Network accents use 0.15/0.05 hysteresis instead of a single 0.1 threshold -- no more color flicker near idle; unused `_ping_ms`/`_volumes` args dropped.
- **Worker Startup Latch**: `WORKER_ALIVE` is set before `spawn()` so early `request_process_scan` calls are never dropped; dead `draw_mini_sparkline` and Three Walls helpers removed.
- **Line Ceiling**: `cpu/history.rs` 280 to 250, `cpu/qos_action.rs` 280 to 207.
- **Version bump to 0.4.21**.

## [0.4.20] - 2026-03-31

- **Config Hardening**: `state.cfg` capped at 64 KB, `win_w`/`win_h` reject `NaN`/`inf` via finite check, no unbounded read on UI thread.
- **Hotkey Stability**: no early blank frame on hotkey, `wants_keyboard_input` gates tab shortcuts while typing, `Ctrl+F` and search `Esc` still work.
- **Worker Cadence**: single `unpark`, startup scan only at `tick == 0` when not minimized, minimum 50 ms yield, `worker_loop.rs` trimmed to 258 lines.
- **Core Cleanup**: `LatestSlot::store` poison-tolerant, test-only helpers gated with `cfg(test)`, dead `SCAN_TICKS_STORAGE` removed.
- **Version bump to 0.4.20**.

## [0.4.19] - 2026-03-31

- **Pause-Safe History Sync**: Storage and Network ingest use worker `sample_elapsed_secs` with a `> last` gate only, so 995 ms ticks are never dropped; Storage pushes hardware rates even with no fresh process scan.
- **Disk I/O Dedup**: `per_disk_rates` queries each physical disk once via visited set, `aggregate_io` tie-breaks by name then pid, inspector reserves 118 px so top-5 never jumps.
- **1 Hz Probe**: fast ping interval 2000 to 1000 ms, jitter resets on timeout, delta tracker resets baseline after `dt > 15.0` s.
- **Version bump to 0.4.19**.

## [0.4.18] - 2026-03-31

- **Clean Chart Startup**: CPU and Memory no longer insert a fake full-width flat line when history is younger than the window; left-edge continuity only when `start_idx > 0` with prior sample.
- **Storage and Network Stabilization**: monotonic `push` guards, LTTB 300 to 920, removed single-sample stretch hack, stable `storage_io_plot` and `network_io_plot` IDs with forced `set_plot_bounds`.
- **Version bump to 0.4.18**.

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


