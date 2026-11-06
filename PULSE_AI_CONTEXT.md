# PULSE AI Context

## Current Milestone: v0.2.11 (Stage A Finale Golden Baseline)

**Status**: `[00] Processes`, `[01] Memory (RAM)`, `[02] Processor (CPU)`.
Tests: 171 automated checks (167 unit + 4 guard).

**Key Deliverables in v0.2.11**:
- 15m chart stability: LTTB threshold 900 with exact 1-second points, empty-history guards, 120ms warmup sleep, unified X labels.
- Fluid core grid: up to 14 columns with tile width filling 100% of window width.
- True EcoQoS: Power Throttling via SetProcessInformation, unified PID gate (31 critical OS), local clock timestamps.
- Audit fixes: poison-safe mutex locks, blocking LatestSlot take, 5m default windows, on-exit flush, Ctrl+F/Esc route to Processes.
- 171 automated checks passing; 0 FPS minimized, 6 direct dependencies, single MIT `LICENSE`.

**Next Phases**: `[03] Storage`, `[04] Network`, `[05] System and GPU`, then Stage B: Total Sci-Fi Cockpit UI 2.0 Redesign after core modules are online.

## Architecture Summary

Two-thread decoupled telemetry model:
- UI thread: `egui` immediate-mode rendering, 2s visible repaint, 30s minimized watchdog
- Worker thread: 1 Hz RAM + kernel metrics, 3s Processes tab / 5s background process scan
- Minimized: 0 FPS UI, 5s poll, `Arc<Vec<RawProcessRow>>` shared zero-copy
- Channel: `sync_channel(2)` with `LatestSlot` overflow tracking

Safety invariants:
- `#![deny(unsafe_code)]` at crate root, localized `#[allow(unsafe_code)]` for Win32 only
- PID Safety Gate: blocks PID 0, 4, self, 31 critical OS, 11 anti-cheat
- 3-point identity verification before any mutating action (fail-closed)
- Frozen `PendingKillConfirm` targets on `[ End Task ]` confirmation

## Development Constraints

- `rust-version = "1.88"`, `edition = "2021"` - no newer MSRV features
- 6 direct dependencies exact (`=`): `eframe` `egui` `egui_plot` `egui_extras` `sysinfo` `windows-sys`
- All `.rs` files <= 280 lines after `cargo fmt --all`
- Documentation files <= 120 lines / 8192 bytes (`README.md`, `ARCHITECTURE.md`, `CHANGELOG.md`)
- Zero Cyrillic (U+0400..=U+052F), zero NUL bytes, zero `rustfmt::skip`
- `cargo fmt --all -- --check`, `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings` must pass
