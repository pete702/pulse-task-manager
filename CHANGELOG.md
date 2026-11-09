# Changelog

All notable PULSE changes are in this file. Earlier 0.1.x entries are in CHANGELOG_ARCHIVE.md.

## [0.4.57] - 2026-03-31

- **Eco Rollback**: optimistic overrides restored on failure, `apply_eco_rollbacks` via action channel.
- **Delta and Hover**: 10s scale clamped to [1/30, 10] with 1.0s floor, hover uses `resolve_exe_cached`.
- **Version bump to 0.4.57**.

## [0.4.56] - 2026-03-31

- **Group Identity**: `GroupTarget` with start/creation/exe, imposter path guard, group trim/eco/kill/open with reuse checks.
- **Menu Split**: actions moved to `table_menu_actions.rs` to keep files <= 265 lines, tests compacted.
- **Version bump to 0.4.56**.

## [0.4.55] - 2026-03-31

- **Sort and Search**: navigation uses `ensure_sort`, Unicode `contains_ci` with ASCII fast path, header still toggles.
- **Hover Cache**: TTL fallback, no sticky insert when saturated, Drop-guarded in-flight, baseline eviction and 0.05s dt floor.
- **Version bump to 0.4.55**.

## [0.4.54] - 2026-03-31

- **Grouped Hog Fix**: single `from_raw` path, clamped group CPU, per-member hog flag, zero-alloc `pid_contains`.
- **Creation Cache Bound**: max 64 Win32 probes per scan, no permanent `None`, `collect_process_cpu` call simplified.
- **Version bump to 0.4.54**.

## [0.4.53] - 2026-03-31

- **Round-Robin Eco**: new `collector_eco` cache (budget 12, TTL 300s, cursor, prune), `last_scan` field, `disk_rate` helper, exe paths live.
- **Model Harden**: NaN-safe `new`, `from_raw` without double format, removed duplicate builder.
- **Version bump to 0.4.53**.

## [0.4.52] - 2026-03-31

- **EcoQoS Harden**: single-key eviction, `probe->Option<bool>`, no tautological path, ORIG priority remove, prio==0 guard.
- **Open Location Gates**: system/gate/critical/identity checks, `decode_utf16_cstr` NUL strip, dead telemetry removed.
- **Version bump to 0.4.52**.

## [0.4.51] - 2026-03-31

- **Non-Blocking Hover**: exe cache never queries on UI thread (bg thread, 4 in-flight, fallback insert).
- **Persisted Widths**: `PERSISTED_WIDTHS` slot + `cpu_col_widths` in state.cfg, saved on drag/autofit.
- **Version bump to 0.4.51**.

## [0.4.50] - 2026-03-31

- **Purge Scope**: heavy raw rows >=10MiB sorted/truncated 64, enable only when unpaused/idle/non-empty, via `spawn_menu_action`.
- **Menu Guard**: context Terminate arms 3s confirm via `arm_context_menu_kill`, group Trim via `resolve_group_targets`, 8s msg expiry.
- **Version bump to 0.4.50**.

## [0.4.49] - 2026-03-31

- **Column Engine**: drag 1 left gives freed to immediate neighbor, `[f32;7]` temp storage, zero-alloc `Cow` truncate.
- **Virtualized Rows**: `show_rows` virtualization, dynamic `max_ram` heat bars, striping/hover, `selected_start_time` on click.
- **Quick Filter**: active-pill no-op guard, removed dead `passes`, version bump to 0.4.49.

## [0.4.48] - 2026-03-31

- **CPU Table Guards**: normalized search key, generation-gated cache, `scan_changed` delta with 15s cap, PID reuse clear with `selected_start_time`, `baseline_scan_gen`.
- **Eco Overrides**: `eco_overrides` read for display and quick filter, 512 cap, case-insensitive name sort, disconnected action channel recovery.
- **Version bump to 0.4.48**.

## [0.4.47] - 2026-03-31

- **Linux CI Clean**: `cfg`-gated Windows imports in `qos_action.rs` and `physical_disk.rs`; `HANDLE` unified to `isize` in `safe_handle.rs`; Win32 calls wrapped with non-Windows stubs in `actions.rs`.
- **Version bump to 0.4.47**.

## [0.4.46] - 2026-03-31

- **Single-Source Layout**: `table_body.rs` computes `col_w/col_x` once per frame; headers, cells, and dividers at `col_x[i+1]` share exact X positions; `TableBuilder` removed; cells inset 6px, RAM bar capped in `widths[4]`.
- **Proportional Fit**: `fit_column_widths` scales slack above min widths to keep `sum == avail`; `MIN_COL_W` tightened; divider 1 left distributes to right columns (no giant CLONES).
- **Version bump to 0.4.46**.

## [0.4.45] - 2026-03-31

- **Toolbar 2-Tier**: `table.rs` uses 1 row >= 1180px with 300px reserved right region; below it uses badge/filter/pills row + group/density + Purge/Pause/counter row, no Pause/Purge overlap.
- **Draggable Columns**: new `table_cols.rs` 7-column engine with min widths, sum == avail, ResizeHorizontal dividers, drag cascade, double-click autofit; CLONES compact, PROCESS NAME takes remainder.
- **Dynamic Truncation**: name chars = `((w-16)/7.2)` min 12, RAM bar fits inside `widths[4]`, separators align header/body.
- **Version bump to 0.4.45**.

## [0.4.44] - 2026-03-31

- **KPI Staircase**: single `34px` strip paints headers at `top_y` and values at `val_y` via `painter().text`, deterministic `x0..x3` offsets, `proportional(10.0)` / `monospace(13.0)`.
- **Synthetic ID**: `scope_label_for` decodes `>= 0x8000_0000` to `Disk [L:]` (letter from low byte), never prints `2147483724`; test asserts no `214748` substring.
- **Header Title**: full `DISK THROUGHPUT` kept when `header_rect.width() >= 380.0`, short `THROUGHPUT` only below `380px`.
- **Bar-Matched Pct**: partition capacity in `TEXT_SECONDARY`, percentage in `bar_color` (Cyan/Amber/Red) matching the bar below.
- **Version bump to 0.4.44**.

## [0.4.43] - 2026-03-31

- **Header Overlap**: chart header split into `header_rect` left/right zones, right controls first (`15m 5m 1m` + `[ Disk 0 x ]` chip), left title clipped to `(width - controls - 8).max(120)`, short title below 460px.
- **Y Decimals**: ceiling `< 10.0` prints `{:.1}` ticks (`1.0 MB/s, 0.8, 0.5, 0.3`) instead of `1, 1, 0, 0`.
- **KPI Alignment**: short `BLACKBOX/READ MB/s/WRITE MB/s/TOP I/O` headers, `10/13` fonts on one baseline, widths `(w*0.18).clamp(68,95)`, short `Disk 0` scope max 14 chars.
- **Ultra-Compact Disks**: `Margin(8,4)`, `16px` header, `15px` partitions, `3px` bars, `H: NTFS` + `used/total (Free) %`, 2-column `chunks(2)` when `>= 440px`.
- **Version bump to 0.4.43**.
