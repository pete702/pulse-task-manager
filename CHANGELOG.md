# Changelog
All notable PULSE changes are in this file. Earlier 0.1.x entries are in CHANGELOG_ARCHIVE.md.
## [0.6.7] - 2026-11-25
- **ui/honesty**: 60s sparklines, fixed scales, gap-safe GPU tail, no fake Processes line; bounded charts and Ctrl+K SelectProcess fix.
## [0.6.6] - 2026-11-25
- **gpu-telemetry**: real 1 Hz PDH cadence, adapter-wide engine load, independent VRAM recording and interactive chart HUD.
## [0.6.5] - 2026-11-24
- **ui/space**: tighter shell margins, inline Trim Memory, honest RAM composition, clean CPU cards and GPU legend.
## [0.6.4] - 2026-11-24
- **ui/gpu**: GPU telemetry in the 1 Hz worker, unified chart HUD, compact System bar and clickable GPU table.
## [0.6.3] - 2026-11-24
- **ui/memory**: fixed empty chart, single mono-readout header, clean Inter typography.
## [0.6.2] - 2026-11-24
- **ui-2.0**: distinct interaction colors, Inter labels, stable header/status layout, clean chassis pills.
## [0.6.1] - 2026-11-23
- **ui-2.0**: 1 Hz sidebar sparklines, compact header/status, restyled GPU/Memory/CPU cockpits.
## [0.6.0] - 2026-11-23
- **ui-2.0**: embedded fonts, Obsidian Cyan cockpit, collapsible sidebar/sparklines, Mini-HUD/Ctrl+K.
## [0.5.42] - 2026-11-23
- **pdh/storage**: LUID-less engine %, real probe traffic, disk rate once, CSV/UI/instance fixes.
## [0.5.41] - 2026-11-23
- **core**: drain async save on exit; per-worker controls, probe TTL, rate prune, blackbox restart fix.
## [0.5.40] - 2026-11-23
- **platform/pdh-gpu**: per-process telemetry keyed by (PID, LUID) so iGPU and dGPU usage never overwrite each other; per-PID counter fallback replaces silent zero.
- **ui/System GPU**: Spill Risk badge uses only the adapter's own Non-Local bytes; unavailable counters render as --.
## [0.5.39] - 2026-11-23
- **platform/win32**: ERROR_MORE_DATA is a BOOL failure in SCM/Run-key reads; clean empty lists refresh cache, incomplete keys are never cached silently; Restart Manager keys use 33 WCHARs.
- **platform/tray/hybrid**: tray modify falls back to add and restores a stranded hidden window; truncated CPU records fail closed.
- **app_tray/mini-hud/qos**: viewport hide fallback, primary-hotkey-only registration, preserved Mini-HUD geometry and clean EcoQoS disable.
## [0.5.38] - 2026-11-22
- **platform/pdh-gpu**: headless adapter totals of zero no longer fail the live smoke test; adapter-dominates-process invariant moved to a deterministic fixture test.
- **app_tray**: local Ctrl+Shift+F9 fallback yields to the registered global hotkey, eliminating rare double-toggle across tray and UI threads.
## [0.5.37] - 2026-11-22
- **platform/linux-ci**: cfg-gated Win32-only helpers and removed the unread history_len field so clippy -D warnings is clean on ubuntu-latest.
- **platform/pdh-gpu**: headless live sampling test only asserts adapter totals when hardware adapters are actually reported.
## [0.5.36] - 2026-11-22
- **platform/linux-ci**: cfg-gate Windows-only helpers and imports so clippy -D warnings passes on ubuntu-latest.
## [0.5.35] - 2026-11-22
- **platform/hotkey**: third Ctrl+Shift+F10 fallback, exact HK status label, tray host window HWND 0 with WM_NULL wake and maximized-to-Mini-HUD restore.
- **ui/mini-hud**: unmaximize before resizing, 124px strip height that fits verdict and metrics, maximized-state restore on exit.
## [0.5.34] - 2026-11-22
- **ui/System GPU**: bounded one-row filter strip, 265 px cockpit row and headless visibility assertions for card, chart and consumers table.
## [0.5.33] - 2026-11-22
- **ui/System GPU**: top-down cockpit columns so adapter card, chart and consumers table draw on frame 1; single filter strip; full-width striped tables.
- **platform/services**: treat ERROR_SUCCESS as success in EnumServicesStatusExW, byte-accurate buffer sizing, resume reset and wide-name bounds; live SCM test requires >= 10 services without partial data.
## [0.5.32] - 2026-11-22
- **core/memory/ui**: verdict expiry, leak slope, palette search.
## [0.5.31] - 2026-11-22
- **platform/gpu**: slot-14 proof, AMD APU, per-LUID spill, PDH backoff.
## [0.5.30] - 2026-11-22
- **platform/windows**: safe resume retention, clean mutex reacquire, tray warning.
## [0.5.29] - 2026-11-22
- **platform/windows**: aligned buffers, bounded reads, MORE_DATA retries, fail-closed identity.
## [0.5.28] - 2026-11-22
- **ui/System GPU**: symmetric 2-column cockpit with live VRAM/GPU history chart, sortable consumers table, fixed adapter pill and VRAM sidebar badge.
## [0.5.27] - 2026-11-21
- **platform/pdh**: system-wide adapter VRAM from `GPU Adapter Memory` counters matched by LUID, plus per-PID GPU %, paged estimate, iGPU spill suppression, HF `org/repo` tags, 5s-60s backoff and a VRAM/GPU history ring; DXGI `QueryVideoMemoryInfo` is budget-only.
## [0.5.26] - 2026-11-21
- **platform/dxgi**: verified COM slots (EnumAdapters1 12, GetDesc1 10, QueryVideoMemoryInfo 14, never 16) with guarded adapter3 QueryInterface and safe fallback to descriptor VRAM totals.
- **platform/pdh**: 8-byte aligned Vec<u64> item buffer, PDH_FMT_DOUBLE|NOCAP100, strict MORE_DATA/count/CStatus bounds and null-name checks, clean baseline on open.
- **ui/System**: honest `--` for zero-limit memory bars and live crash-free DXGI/PDH Windows smoke tests.
## [0.5.25] - 2026-11-21
- **ui/command-palette**: centered Ctrl+K palette with fuzzy command/process search, arrow navigation, Enter/click execution and auto-focused input.
- **chrome/app-render**: `[ Ctrl+K ]` header button, global shortcut wiring, and palette-aware Escape handling.
## [0.5.24] - 2026-11-21
- **platform/services-startup**: read-only SCM enumeration via advapi32 (RAII handles, 5 s cache) and HKCU/HKLM Run-key autostart values.
- **ui/System**: 3-mode switcher `[ GPU & VRAM ] [ Services (SCM) ] [ Startup (Run) ]` with filters, refresh, PID jump and click-to-copy commands.
## [0.5.23] - 2026-11-21
- **platform/hybrid-cores**: cached `GetLogicalProcessorInformationEx` EfficiencyClass scan; P/E mask split only on genuine hybrid CPUs (<=64 logical cores).
- **cpu menu**: verified `[ P-Cores ]`, `[ E-Cores ]`, `[ All Cores ]` affinity actions on the persistent MenuJob worker with 3-point identity binding.
## [0.5.22] - 2026-11-21
- **platform/pdh**: lazy persistent `PdhAddEnglishCounterW` query for per-process GPU Local/Non-Local usage, throttled to 2 s and closed via `PdhCloseQuery` on drop.
- **ui/System GPU**: live DXGI adapter cards and per-process VRAM table with `[VRAM Spill]`, `[ AI: <model> ]`, honest `--` and dwm.exe tooltip.
## [0.5.21] - 2026-11-21
- **platform/dxgi**: pure user-mode GPU enumeration via dxgi.dll with RAII COM release, LUID, VRAM budgets and integrated/discrete classification.
- **platform**: no D3DKMT calls, so sleeping laptop dGPUs stay at 0.0 W; software adapters used only when no hardware GPU exists.
## [0.5.20] - 2026-11-21
- **memory/leak-detector**: bounded 30s sampling of the top 128 processes detects slow 5-minute monotonic leaks at >= 1 MB/min without burst false positives.
- **ui**: `[LEAK +N MB/m]` badge in the process table memory column and in the memory chart header.
## [0.5.19] - 2026-11-21
- **core/blackbox**: 60-second RAM ring buffer scores disk, commit, single-core and jitter signals into timestamped freeze verdicts.
- **ui/export**: verdict shown in status bar and Mini-HUD; Lag-Receipt button writes manual ASCII JSON/CSV to the config directory.
## [0.5.18] - 2026-11-21
- **privacy/admin**: status bar ADMIN/USER badge, portable indicator and Restart as Admin with mutex handoff and UAC cancel fallback.
- **portable mode**: `--portable` or side-by-side `pulse.cfg` keeps config and panic logs in the executable directory.
## [0.5.17] - 2026-11-20
- **cpu/pick-window**: armable crosshair picker resolves an external window's root PID and selects its row with a status message.
- **ui**: persistent repaint feedback while picking; Escape cancels, filter clears on a valid pick.
## [0.5.16] - 2026-11-20
- **cpu/collector**: cache formatted CommandLine/User per (pid, start_time) in the bounded FIFO and reuse strings across ticks.
- **maintainability**: extract pure helpers from cpu table and storage collector for line-count headroom.

