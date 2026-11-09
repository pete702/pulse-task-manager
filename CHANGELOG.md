# Changelog

All notable PULSE changes are in this file. Earlier 0.1.x entries are in CHANGELOG_ARCHIVE.md.

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

## [0.4.42] - 2026-03-31

- **Master Height**: `horizontal_top` with 280px full-height master, pinned ALL selector outside scroll, `.max_height(full)` scroll, `set_min_width` full-width cards, right pane `(full_w - 290px)`.
- **Activity Columns**: `draw_table_row` with `set_min/max_width` fixed columns (`88/88/92px` + remainder), header separator `BORDER_SUBTLE`, card fills remaining height.
- **KPI Single Line**: `TOP I/O` gets remainder width after 3x `110px` columns, value uses `Label::truncate(true)` so it never wraps.
- **Version bump to 0.4.42**.

## [0.4.41] - 2026-03-31

- **Master-Detail Cockpit**: `render_storage_tab` splits at `>= 680px` into 275px master (ALL selector + vertical physical cards) and detail (KPI/chart/activity); narrow windows fall back to vertical stack.
- **Master Clickability**: disk headers and each partition use `Sense::click()`, two-item headers never collide, partition right text colored by `health_label`.
- **Dedup Inspector**: removed `SELECTED PARTITION` duplicate box; activity is always 100% width with `All Drives` / `On X:` toggle kept.
- **Tooltip Fix**: storage series renamed to `Read` / `Write`, custom clamped 240px hover box shows `Write: 9.5 MB/s (3.2m ago)` inside `plot_rect`, default rulers disabled.
- **Version bump to 0.4.41**.

## [0.4.40] - 2026-03-31

- **Extents Fallback**: `IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS` maps volumes when `STORAGE_DEVICE_NUMBER` fails; per-mount `DISK_PERFORMANCE` keyed `0x8000_0000 | letter` covers unmapped `L:\`.
- **Physical Disk Cards**: 2-column grid with compact 68px cards, header `DISK N (partitions/GiB)` + `R/W` + worst badge, partitions horizontal via `ui.columns`.
- **Activity Width**: full-width `TOP PROCESS DISK ACTIVITY` when `selected is None`, else 58/42 split with `Clear Selection / Show All` inspector.
- **Version bump to 0.4.40**.
