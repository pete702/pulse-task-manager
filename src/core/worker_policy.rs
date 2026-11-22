//! Worker polling policy constants for PULSE telemetry.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

static TRAY_HIDDEN: AtomicBool = AtomicBool::new(false);

pub fn set_tray_hidden(hidden: bool) {
    TRAY_HIDDEN.store(hidden, Ordering::SeqCst);
}

pub fn is_tray_hidden() -> bool {
    TRAY_HIDDEN.load(Ordering::SeqCst)
}

/// Worker tick interval when UI is visible (1 second).
pub const WORKER_INTERVAL_VISIBLE: Duration = Duration::from_millis(1000);

/// Worker tick interval when UI is minimized (1 second telemetry).
pub const WORKER_INTERVAL_MINIMIZED: Duration = Duration::from_millis(1000);

pub fn worker_tick_interval(minimized: bool) -> Duration {
    if minimized {
        WORKER_INTERVAL_MINIMIZED
    } else {
        WORKER_INTERVAL_VISIBLE
    }
}

/// Full process scans run every 3rd visible tick (3 seconds when the Cpu tab is open).
/// Only the Cpu and Processes tabs use this fast process cadence.
pub const SCAN_TICKS_VISIBLE: u64 = 3;

/// Full process scans run every 5th tick when Overview tab is open (5 seconds).
pub const SCAN_TICKS_BACKGROUND: u64 = 5;

/// RAM usage threshold that triggers the high-pressure banner.
pub const HIGH_RAM_PRESSURE_PCT: f32 = 85.0;

/// Cadence gate for full process scans.
pub fn should_scan_processes(
    ticks_since_scan: u64,
    fast: bool,
    force_request: bool,
    minimized: bool,
    is_first_tick: bool,
) -> bool {
    if minimized || is_tray_hidden() {
        return false;
    }
    if is_first_tick || force_request {
        return true;
    }
    if ticks_since_scan == 0 {
        return false;
    }
    let interval = if fast { SCAN_TICKS_VISIBLE } else { SCAN_TICKS_BACKGROUND };
    ticks_since_scan >= interval
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tray_hidden_pauses_scan() {
        set_tray_hidden(true);
        assert!(!should_scan_processes(10, true, false, false, false));
        set_tray_hidden(false);
        assert!(should_scan_processes(10, true, false, false, false));
    }
}
