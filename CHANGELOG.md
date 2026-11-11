# Changelog

All notable PULSE changes are in this file. Earlier 0.1.x entries are in CHANGELOG_ARCHIVE.md.

## [0.4.77] - 2026-04-17

- **CPU Table**: single-entry bounded eviction for `eco_overrides` at the 512-entry cap in `table_state_deltas.rs` and `table_menu_actions.rs` (preventing mass reset of active EcoQoS overrides).
- **Version bump to 0.4.77**.

## [0.4.76] - 2026-04-17

- **Charts**: clean-4 Y-axis scaling on Network and Storage charts (fixing `1 Mbps, 1, 0, 0` and fractional ticks), with dark HUD axis pills.
- **Storage UI**: `MB/s` unit on Storage `TOP I/O`.
- **Version bump to 0.4.76**.

## [0.4.75] - 2026-04-17

- **Ingest**: unified 4-module snapshot ingestion in `app_ingest.rs` and minimized snapshot draining without UI rendering.
- **Charts**: `push_gap_anchored` time-gap clamping across CPU, Storage, and Network chart histories.
- **Version bump to 0.4.75**.

## [0.4.74] - 2026-04-17

- **Trim/UI**: `core::win32::self_trim_working_set` result-aware 10s retry backoff and `OnceLock` app icon caching.
- **CPU/Menu**: tri-state `EcoQoS` menu status (active/off/unprobed).
- **Version bump to 0.4.74**.

## [0.4.73] - 2026-04-17

- **Trim/UI**: unified self-trim policy with 10s minimize cooldown and immediate repaint on snapshot ingest.
- **CPU/Panic/CLI**: `CpuSnapshot.process_scan_id` decoupling, re-entrancy-guarded PID-scoped `create_new` panic hook, and unknown CLI flag warning.
- **Version bump to 0.4.73**.

## [0.4.72] - 2026-04-17

- **Hotkeys/Resume**: pre-ingest hotkey handling, instant process scan on F10 and UI telemetry resume.
- **UX/Scope**: tab-scoped Escape, fast process cadence predicate excluding Memory overview, and CPU table freeze tooltip.
- **Ingest**: snapshot_id deduplication for ingest.
- **Version bump to 0.4.72**.

## [0.4.71] - 2026-04-17

- **Worker Loop**: edge-triggered pause unpark swap, early-wakeup guard, and timestamp monotonicity only on scheduled due ticks.
- **Process Tables**: scanned-gated `process_scan_id`/`last_scan_tick` and full CPU/Memory process cache restoration when not scanned.
- **Version bump to 0.4.71**.

## [0.4.70] - 2026-04-17

- **Worker Safety/Cadence**: `SpawnedWorker` optional handle, pause scan-request handling with instant resume unpark, monotonic snapshot timestamps, and elapsed-ticks scan cadence with active minimized tick interval.
- **Worker Invariants**: unified `raw_processes`/`process_count` assignment and due-gated storage/network collection reuse.
- **Version bump to 0.4.70**.

## [0.4.69] - 2026-04-17

- **CPU History**: O(1) revision-tagged series caches, removed dead `cached_revision`, and bounded ring-buffer peak decay on eviction.
- **CPU/Worker**: synced `cpu/mod.rs` 960-sample doc and added UI wakeup on `publish_payload` `Full`.
- **Version bump to 0.4.69**.

## [0.4.68] - 2026-03-31

- **CPU UI**: removed duplicate Overview sub-view button and clarified `LOGICAL CORES / PROCS` label.
- **Charts**: added dark backdrop pills behind axis labels and unified scanner time format in CPU/Memory HUDs.
- **Memory**: fixed composition bar segment math so In Use + Cached + Free never exceeds total RAM.
- **Version bump to 0.4.68**.

## [0.4.67] - 2026-03-31

- **Charts**: added vertical scanner, curve dots, and tactical tooltip to CPU chart.
- **Charts**: unified Memory, Storage, and Network charts to single vertical scanner, arrow cursor, and no horizontal crosshair.
- **Version bump to 0.4.67**.

## [0.4.66] - 2026-03-31

- **CPU Sub-view**: two-way toggle with `toggle_sub_view` and back-to-overview control from Processes.
- **Window Width**: unified 860.0 literals on canonical `MIN_WINDOW_WIDTH` and shared top-cards offset.
- **Version bump to 0.4.66**.

## [0.4.65] - 2026-03-31

- **CPU Hotkeys/UI**: Escape now surrenders CPU search focus unconditionally on the CPU tab.
- **CPU Hardening/Tests**: extracted grid/bottleneck helpers and added regression tests for grid sizing, labels, badges, and quick filters.
- **Version bump to 0.4.65**.

## [0.4.64] - 2026-03-31

- **CPU Chart**: cached Y-grid marks, preallocated time conversion/anchoring, and tactical scanner hover/cursor cleanup.
- **CPU UI/State**: single Escape path for CPU search, CPU sub-view toggle, and shared min window width constant.
- **Version bump to 0.4.64**.

## [0.4.63] - 2026-03-31

- **CPU Chart**: fixed legend rect lag/double frame/comment, separated legend state/button IDs, deduped/cached X grid marks, and added chart axis tests.
- **Tests**: asserted exact priority class restoration in live EcoQoS roundtrip.
- **Version bump to 0.4.63**.



















