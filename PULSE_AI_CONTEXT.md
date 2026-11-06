# PULSE AI Context

## Current Milestone: v0.2.9 (Stage A Finale Golden Baseline)

**Status**: `[00] Processes`, `[01] Memory (RAM)`, `[02] Processor (CPU)`.
Tests: 159 automated checks (155 unit + 4 guard).

**Key Deliverables in v0.2.9**:
- Zero-crate state persistence: tab, grouping, density, sort, chart windows, grid view, and window size in `%LOCALAPPDATA%\PULSE\state.cfg` (`src/core/persist.rs`).
- Smart resizable Processes columns: compact `Column::initial(...).range(...).resizable(true)` widths via `src/modules/cpu/table_menu.rs`.
- Live System Telemetry TopBar (`[ CPU | RAM | UPTIME | HEALTH ]`) plus true Win32 `GetUserDefaultLocaleName` OS locale; self-RAM/UI moved to status bar.
- Sci-Fi tactical HUD: `draw_cyber_corner_brackets`, `draw_tactical_ticks`, dual-tone top scanline.
- 159 automated checks passing; 0 FPS minimized, 6 pinned crates, single MIT `LICENSE`.

**Next Phases**: `[03] Storage`, `[04] Network`, `[05] System and GPU`, then Stage B: Total Sci-Fi Cockpit UI 2.0 Redesign after core modules are online.

## Architecture Summary

Two-thread decoupled telemetry model:
- UI thread: `egui` immediate-mode rendering, 2s visible repaint, 30s minimized watchdog
- Worker thread: 1 Hz RAM + kernel metrics, 3s Processes tab / 5s background process scan
- Minimized: 0 FPS UI, 5s poll, `Arc<Vec<RawProcessRow>>` shared zero-copy
- Channel: `sync_channel(2)` with `LatestSlot` overflow tracking

Safety invariants:
- `#![deny(unsafe_code)]` at crate root, localized `#[allow(unsafe_code)]` for Win32 only
- PID Safety Gate: blocks PID 0, 4, self, 27 critical OS, 11 anti-cheat
- 3-point identity verification before any mutating action (fail-closed)
- Frozen `PendingKillConfirm` targets on `[ End Task ]` confirmation

## Development Constraints

- `rust-version = "1.85"`, `edition = "2021"` - no newer MSRV features
- 6 pinned dependencies exact (`=`): `eframe` `egui` `egui_plot` `egui_extras` `sysinfo` `windows-sys`
- All `.rs` files <= 280 lines after `cargo fmt --all`
- Documentation files <= 120 lines / 8192 bytes (`README.md`, `ARCHITECTURE.md`, `CHANGELOG.md`)
- Zero Cyrillic (U+0400..=U+052F), zero NUL bytes, zero `rustfmt::skip`
- `cargo fmt --all -- --check`, `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings` must pass
