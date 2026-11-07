# Changelog

All notable PULSE changes are in this file. Earlier 0.1.x entries are in CHANGELOG_ARCHIVE.md.

## [0.4.29] - 2026-03-31

- **Safe Font Fallback**: `setup_fonts` starts from `FontDefinitions::default`, inserts Segoe UI and Consolas only when read cleanly and <= 16 MB, removed broken `default` pushes.
- **Hotkeys & Focus**: F5 triggers scan, `Ctrl+F` gated by `!wants_input`, Esc surrenders focus when search is empty, hotkey doc updated.
- **Startup & Repaint**: trim delayed to frame 60 with snapshot ready, redundant repaint removed, focus-aware 2000/5000 ms heartbeat, `collect_persisted` sanitizes non-finite win size.
- **CLI & NaN Guards**: console attach only for `--version`/`-V`/`--help`/`-h`/`--self-test`, panic log symlink check and 65536 cap, `sanitize_f32`/`sanitize_f64` helpers, `load_color` returns secondary on NaN.
- **Version bump to 0.4.29**.

## [0.4.28] - 2026-03-31

- **FIFO Ring Buffer**: `LatestSlot` holds `VecDeque` capped at 960 samples, `store` pushes back, `take` pops newest, `drain_all` returns chronological order and resets overflows.
- **Chronological Ingest**: `poll_snapshots` batches `snapshot_rx` plus `drain_overflow_snapshots`, sorts by `sample_elapsed_secs`, ingests every sample; called before the minimized early exit.
- **1 Hz Minimized Telemetry**: `WORKER_INTERVAL_MINIMIZED` set to 1000 ms, process scans stay paused while minimized, CPU refresh runs every tick, no repaint while minimized.
- **Version bump to 0.4.28**.

## [0.4.27] - 2026-03-31

- **CPU Async Menu**: Trim/EcoQoS/Open/Terminate spawn via `spawn_menu_action`, `poll_menu_action` sets message with timestamp, applies `eco_overrides` and `mark_pending_eco`, requests scan.
- **Unified Safety & EcoQoS**: `is_system_process` delegates to `verify_pid_safety_gate`, `SetProcessInformation` result checked via `info_ok`, End Task adds access-denied suffix when zero terminated.
- **Collector Budget**: `PENDING_ECO`/`ECO_WATCH`/`mark_pending_eco`/`set_eco_watch`/`probe_eco_qos_budgeted` moved to `qos_action.rs`, collector 239 and qos 238 lines.
- **Version bump to 0.4.27**.

## [0.4.26] - 2026-03-31

- **UI-Thread Syscalls Removed**: `ProcessTarget::from_raw` and `resolve_selection` do zero Win32 calls; `first_exe_path`/`first_pid` resolve lazily on the background thread via `query_process_exe_path`.
- **Context Menu Non-Blocking**: Open/Trim/Terminate route through `action_bar.spawn`, double-click spawns resolution, no per-frame `query_process_exe_path`.
- **Actions & Win32 Cleanup**: `open_process_file_location` merges test guards and returns `Failed to launch Explorer` errors; `query_process_exe_path` uses `SafeHandle` + `query_process_image_path`.
- **Version bump to 0.4.26**.

## [0.4.25] - 2026-03-31

- **Docs Modernization**: README and ARCHITECTURE rewritten for `[01]-[05]` (CPU, Memory, Storage, Network, System & GPU), strict ASCII-only, CPU/EcoQoS, Storage Freeze Hunter, and Ping/Jitter Radar marked Implemented [OK].
- **Deep Audit P3 Cleanups**: `worker.rx` uses `expect`, direct `worker_io` imports, `with_worker_and_state` avoids double `load_state`, true debounce via `dirty_state_snapshot`, `pulse_ram_mb` error clarifies `K32GetProcessMemoryInfo`.
- **ASCII Guard**: `docs_guard` verifies README, ARCHITECTURE, and CHANGELOG are strict ASCII (`is_ascii`).
- **Version bump to 0.4.25**.

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

<!-- rotated -->










