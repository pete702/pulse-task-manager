# Changelog
All notable PULSE changes are in this file. Earlier 0.1.x entries are in CHANGELOG_ARCHIVE.md.
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
## [0.5.15] - 2026-11-20
- **hotkey/worker**: single Ctrl+Shift+F9 hotkey with silent fallback, persisted toggle and panic-safe menu jobs.
## [0.5.12] - 2026-11-20
- **tray/hud**: tray-thread restore wakes hidden window, icon visibility verified, two-line HUD metrics.
## [0.5.11] - 2026-11-20
- **mini-hud/tray/hotkey**: reliable drag start, conditional tray hide, bounded shutdown and hotkey fallback status.
## [0.5.10] - 2026-11-20
- **instance/window**: exact window-title match wakes the real PULSE window and rejects tray/IME helpers.
## [0.5.9] - 2026-11-19
- **process/tools**: read-only Restart Manager file-lock finder for dropped paths and click-to-pick window PID.
## [0.5.8] - 2026-11-19
- **process/suspend**: verified Suspend/Resume with hard blocklist, [SUSPENDED] badge and exit auto-resume guard.
## [0.5.7] - 2026-11-19
- **cpu/table**: Flat/Clones/Tree view modes with cycle-safe hierarchy, lazy CommandLine/User metadata and extended Ctrl+C row copy.
## [0.5.6] - 2026-11-19
- **cpu/background**: persistent exe-path and menu-action workers; explicit resolve/denied states.
## [0.5.5] - 2026-11-19
- **mini-hud**: 330x104 frameless always-on-top strip with Alt+F9, safe viewport restore.
## [0.5.4] - 2026-11-19
- **hotkey**: Alt+F9 global Mini-HUD toggle; bare F9 stays local when focused.
## [0.5.3] - 2026-11-19
- **exit/ux**: tray on X toggle, header tray/exit buttons, Ctrl+Q, instant tray exit.
- **instance/latency**: trim window titles, SW_RESTORE, bounded shutdown joins, storage refresh.
## [0.5.2] - 2026-11-19
- **instance/tray**: fail-open single-instance guard; restore window via SW_RESTORE.
## [0.5.1] - 2026-11-19
- **tray**: system tray daemon with hide-to-tray and sleep flag.
## [0.5.0] - 2026-11-19
- **instance**: single-instance guard wakes existing PULSE window to foreground.
## [0.4.163] - 2026-04-12
- **network/status**: split latency probe net helpers; expose dropped snapshots in status bar.
## [0.4.162] - 2026-04-12
- **storage**: 5s disk refresh, 350 MB/s spike threshold, 300s handle cache TTL eviction.

