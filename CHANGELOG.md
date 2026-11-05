# Changelog

All notable changes to **PULSE** (`pulse_manager`) are documented in this file.
Binary assets and checksums are published on [GitHub Releases](https://github.com/pete702/pulse-task-manager/releases).

## [0.1.22] - 2026-11-05

- **OpenGL Driver Memory Optimization**: `eframe::NativeOptions` explicitly disables depth buffer, stencil buffer, and multisampling (`0`); enables `vsync` and `Preferred` hardware acceleration — eliminates ~30–50 MB driver Private Memory (`nvoglv64.dll`) on fullscreen windows.
- **Viewport Resize Trim**: `PulseApp` detects significant viewport size changes (e.g., restore from maximized/fullscreen) and calls `self_trim_working_set()` to reclaim stale driver resize buffers.
- **Unified X-Axis Grid**: Custom `x_grid_spacer` provides consistent major/minor grid lines for all windows: 1m (major 15s, minor 5s), 5m (major 1m, minor 15s), 15m (major 3m, minor 1m).
- **Removed Minus Signs on 1m Labels**: X-axis HUD labels now show positive "time ago" values (`60s`, `45s`, `30s`, `15s`, `0s`) instead of `-60s`..`-15s`; matches 5m/15m style.
- **Guaranteed Zero Left-Edge Chart Gap**: `build_windowed_series` anchors to exact `-span` when history spans `window_secs - 2s` OR first sample within 2s of `-span`; 1m/5m/15m touch left border with 0 px gap on fullscreen.
- **Version bump to 0.1.22**.

## [0.1.21] - 2026-11-05

- **Chart Left-Edge Gap Fix**: 1m/5m windows now clamp left edge to exact `-span` once history spans the full window (within 1.5s tolerance); startup still grows naturally from right to left.
- **Grouped Process RAM % Fix**: Clone groups (`chrome.exe [37x]`, `svchost.exe [58x]`) now show summed RAM percentage (e.g. `31.6%`) instead of single child's `0.1%`; `update_formatted` computes change flags before updating tracking fields.
- **FILETIME vs Unix Time-Base Fix**: `DeltaTracker` no longer falls back from Win32 `FILETIME` (100ns since 1601) to Unix seconds (`start_time`); `creation_time` returned as `Option<u64>` (FILETIME) with `None` when Win32 query fails.
- **Graceful Worker Shutdown**: `TelemetryWorker` implements `Drop` — sets shutdown flag, unparks thread, joins handle; `request_process_scan()` encapsulated on worker; clean exit in <1 ms on app close.
- **Named Constants**: Extracted `MAX_SANE_VIRTUAL_BYTES` (1 TiB), `MEMORY_HOG_THRESHOLD_PCT` (15%), `HEAT_BAR_FULL_SCALE_PCT` (20%), `CHART_ANCHOR_TOLERANCE_SECS` (1.5s) to `constants.rs`.
- **Version bump to 0.1.21**.

## [0.1.20] - 2026-11-05

- **High-Precision Continuous Telemetry Bars**: Replaced 28-segment LED bars with smooth sub-pixel continuous bars (3 draw calls); threshold coloring (cyan <70%, amber 70-85%, red >85%); 14x fewer draw calls.
- **Table Heat-Bar Fix**: Fixed text overlap on Physical RAM / RAM % columns — 8px left padding; deep teal/slate fills; hog rows use ice-white text.
- **Gaming Mode Glyph**: Status bar mode badge uses `▲` for consistent geometric symbol theme.
- **P0-1: `.without_exe()` Collector + On-Demand Win32 Path**: Background scans skip exe path queries; `query_process_exe_path(pid)` resolves on-demand for "Open Location".
- **P0-2 & P0-3: Precomputed Windowed Chart Slices + LTTB Downsampling**: Pure-Rust `lttb_downsample` (≤45 lines, zero deps); caches windowed & anchored series per window with 900→300 point LTTB downsampling.
- **Instant Worker Wake**: `thread::park_timeout` + `unpark()` in `request_process_scan()` — worker wakes in 0 ms when clicking RAM Processes tab.
- **Version bump to 0.1.20**.