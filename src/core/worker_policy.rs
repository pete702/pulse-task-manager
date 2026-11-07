//! Worker polling policy constants for PULSE telemetry.

/// Worker tick interval when UI is visible (1 second).
pub const WORKER_INTERVAL_VISIBLE: std::time::Duration = std::time::Duration::from_millis(1000);

/// Worker tick interval when UI is minimized (1 second telemetry).
pub const WORKER_INTERVAL_MINIMIZED: std::time::Duration = std::time::Duration::from_millis(1000);

/// Full process scans run every 3rd visible tick (3 seconds when Processes sub-tab is open).
/// Storage tab also uses this fast cadence so the 1s chart stays live.
pub const SCAN_TICKS_VISIBLE: u64 = 3;

/// Full process scans run every 5th tick when Overview tab is open (5 seconds).
pub const SCAN_TICKS_BACKGROUND: u64 = 5;

/// RAM usage threshold that triggers the high-pressure banner.
pub const HIGH_RAM_PRESSURE_PCT: f32 = 85.0;

/// Cadence gate for full process scans.
pub fn should_scan_processes(
    minimized: bool,
    tick: u64,
    wake: bool,
    requested: bool,
    fast: bool,
) -> bool {
    !minimized
        && (wake
            || requested
            || tick.is_multiple_of(if fast { SCAN_TICKS_VISIBLE } else { SCAN_TICKS_BACKGROUND }))
}
