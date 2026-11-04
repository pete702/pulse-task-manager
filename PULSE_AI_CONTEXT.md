# PULSE AI Context

## Current Milestone: v0.1.11 (Phase 1 Memory Frozen & Verified)

**Status**: Phase 1 Memory subsystem complete and frozen. 130 tests passing (126 unit + 4 docs-guard).

**Key Deliverables in v0.1.11**:
- 3-segment RAM composition bar (In Use / System Cache / Free) with correct byte math
- Live Commit % history chart (1m/5m/15m) with frame-aligned time scale labels
- Unified O(1) process table cache keyed by `process_scan_id`
- Fail-closed 3-point identity verification (basename + creation_time + full_path)
- Background `Open Location` worker with `OnceLock` elevation cache
- dGPU sleep exports & zeroed 3D buffers (~29 MB Working Set reduction)
- Kernel tile geometry fixed — no layout overflow, bounded card geometry
- Zero debug prints, zero Cyrillic, zero `rustfmt::skip`, all files ≤180 lines after `cargo fmt`

**Next Phase**: Phase 2 Processor/CPU → v0.1.12 → v0.2.0
- Per-core grid, single-core bottleneck detector
- Win32 EcoQoS toggle
- CPU utilization history chart

## Architecture Summary

Two-thread decoupled telemetry model:
- UI thread: `egui` immediate-mode rendering, 60 FPS when visible, 1 Hz when minimized
- Worker thread: 1 Hz system RAM + kernel metrics, 0.5 Hz full process scan (2 s interval)
- Minimized mode: process scans paused, `Arc<Vec<RawProcessRow>>` shared zero-copy
- Channel: `sync_channel(2)` with `LatestSlot` overflow tracking

Safety invariants:
- `#![deny(unsafe_code)]` at crate root, 15 localized `#[allow(unsafe_code)]` blocks
- PID Safety Gate: blocks PID 0, 4, self, 27 critical OS, 11 anti-cheat
- 3-point identity verification before any mutating action (fail-closed)
- Frozen `PendingKillConfirm` targets on `[ End Task ]` confirmation

## Development Constraints

- `rust-version = "1.85"`, `edition = "2021"` — no newer MSRV features
- 6 pinned dependencies exact (`=`): `eframe` `egui` `egui_plot` `egui_extras` `sysinfo` `windows-sys`
- All `.rs` files ≤ 180 lines after `cargo fmt --all`
- Documentation files ≤ 120 lines / 8 KB (`README.md`, `ARCHITECTURE.md`, `SECURITY.md`, `CONTRIBUTING.md`)
- `CHANGELOG.md` ≤ 55 lines
- Zero Cyrillic (U+0400..=U+052F), zero NUL bytes, zero `rustfmt::skip`
- `cargo fmt --all -- --check`, `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings` must pass