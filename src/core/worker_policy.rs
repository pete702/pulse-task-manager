//! Worker polling policy constants for PULSE telemetry.

/// Worker tick interval when UI is visible (1 second).
pub const WORKER_INTERVAL_VISIBLE: std::time::Duration = std::time::Duration::from_millis(1000);

/// Worker tick interval when UI is minimized (5 seconds idle watchdog).
pub const WORKER_INTERVAL_MINIMIZED: std::time::Duration = std::time::Duration::from_millis(5000);

/// Full process scans run every 3rd visible tick (3 seconds when Processes sub-tab is open).
pub const SCAN_TICKS_VISIBLE: u64 = 3;

/// Full process scans run every 5th tick when Overview tab is open (5 seconds).
pub const SCAN_TICKS_BACKGROUND: u64 = 5;

/// RAM usage threshold that triggers the high-pressure banner.
pub const HIGH_RAM_PRESSURE_PCT: f32 = 85.0;
