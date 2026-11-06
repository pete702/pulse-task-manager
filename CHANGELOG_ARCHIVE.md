# Changelog Archive (0.1.x)

Archived entries from 0.1.x. Recent 0.2.x entries live in CHANGELOG.md.

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
