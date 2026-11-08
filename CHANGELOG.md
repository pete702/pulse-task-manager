# Changelog

All notable PULSE changes are in this file. Earlier 0.1.x entries are in CHANGELOG_ARCHIVE.md.

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

## [0.4.39] - 2026-03-31

- **Compact Volume Rows**: replaced tall 2-column disk boxes with sleek single-row strips, `Margin::symmetric(10.0, 6.0)`, spacing 4.0, slim 5px bars, one-line `R/W` + `Used/Total` + badge.
- **Badge Wording**: critical `LOW FREE (% USED)`, `>=80%` `HIGH USE`, else `OK (% USED)`; `None` title `VOLUME L:` without verbatim `L:\` duplicate.
- **Process Table**: idle groups filtered, top-8 retained, `scan_key` hardened, KPI truncate 32, process column 48% with truncate 38, dedup truncate.
- **Version bump to 0.4.39**.

## [0.4.38] - 2026-03-31

- **Handle Cache**: evicted handles closed before clear, race-safe insert closes displaced handle, non-Windows stubs for Win32 IOCTL mapping.
- **Unmapped Volume**: per-letter process I/O stamps `physical_disk == None` volumes like `L:\` live, no Disk 0 inflation, `prev_per_disk` pruned, unmapped totals included.
- **History Scope**: timestamp finite guards, single-key eviction, synthetic `0x8000_0000` key scopes unmapped chart series, per-letter aggregate test added.
- **Version bump to 0.4.38**.

## [0.4.37] - 2026-03-31

- **Interactive Legend**: Storage Read/Write toggles with at-least-one guard, ceiling rescales to enabled series only.
- **Live Header**: throughput readout `R: X MB/s | W: Y MB/s`, time labels backed so fill never obscures text.
- **NaN Guards**: history and chart sanitize non-finite throughput to zero, toggle peak test added.
- **Version bump to 0.4.37**.

## [0.4.36] - 2026-03-31

- **Chrome Harden**: self-RAM sanitized, CPU pcts clamped, health pill hidden below 720px, scanline guarded/clamped.
- **Status & Walls**: diag sanitized, width measured before horizontal, Three Walls NaN/Inf rejected, i18n pcts clamped.
- **Unified Count**: sidebar and status bar both use cpu.process_count.max(processes.len()).
- **Version bump to 0.4.36**.

## [0.4.35] - 2026-03-31

- **NaN Geometry Guard**: `primitives.rs` early-returns on non-finite/tiny rects, sanitizes ratios, verifies inner size before clamp, `cyber_accent` sanitizes input.
- **Sidebar Unload**: badge painting moved to `sidebar_badges.rs::draw_tab_badge`, width clamped, keyboard input hoisted, badges hidden below 140px.
- **Badge Sanitization**: cached formatters and hysteresis clamp non-finite inputs to zero, NaN/Inf never latches state.
- **Version bump to 0.4.35**.

## [0.4.34] - 2026-03-31

- **Storage Attribution**: CPU scan uses `with_exe(UpdateKind::OnlyIfNotSet)` for one-time exe queries, disk guards test-only, brand fallback deduped.
- **Table Split**: bottom action bar moved to `table_menu.rs`, both table files under 235 lines.
- **Non-Windows Stubs**: `SafeHandle` and EcoQoS Win32 calls gated with fail-closed stubs for clean Linux linking.
- **License**: Proprietary notice permits personal non-commercial binary use, strict source/binary restrictions otherwise, `publish = false`.
- **Version bump to 0.4.34**.

## [0.4.33] - 2026-03-31

- **GUI Self-Test Exit**: `SelfTestApp` stores `passed: Arc<AtomicBool>` on valid snapshot, headless checks storage/network/jitter finiteness, GUI failure exits 1.
- **Worker Cleanup**: `WorkerExitGuard` resets scan request, removed redundant interval branches and pre-tick wait, `worker_loop` 240 lines.
- **Persist & Ingest**: `serialize` clamps tab/chart/filter fields, test constant private, `poll_snapshots` uses `worker.drain_overflow_snapshots` in both branches.
- **Version bump to 0.4.33**.

## [0.4.32] - 2026-03-31

- Switched repository and binary license to strict Proprietary (`All Rights Reserved`, explicit written permission required) with `publish = false` in `Cargo.toml`.
- Gated `F5` refresh hotkey behind `!wants_keyboard_input()` in `src/app_tabs.rs` and compacted `src/core/persist.rs`.
- **Version bump to 0.4.32**.
