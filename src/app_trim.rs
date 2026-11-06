//! Periodic working-set trim policy: minimize transition plus a 60-minute timer.
//! Resize must never trigger a trim (avoids page-fault storms while dragging).

use std::time::{Duration, Instant};

/// True when the periodic 60-minute trim is due.
pub fn periodic_due(last: &Instant) -> bool {
    last.elapsed() >= Duration::from_secs(3600)
}

/// Runs one trim and re-arms the timer.
pub fn run_trim(last: &mut Instant) {
    let _ = crate::core::win32::self_trim_working_set();
    *last = Instant::now();
}
