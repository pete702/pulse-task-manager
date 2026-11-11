//! Self-trim policy: startup, minimize cooldown, and periodic timer.
//! Resize must never trigger a trim (avoids page-fault storms while dragging).

use std::time::{Duration, Instant};

pub const SELF_TRIM_PERIODIC_SECS: u64 = 60;
pub const SELF_TRIM_COOLDOWN_SECS: u64 = 10;

pub fn should_trigger_self_trim(
    elapsed_since_last: Duration,
    startup_frames: u32,
    just_minimized: bool,
) -> bool {
    if startup_frames == 60 {
        return true;
    }
    if just_minimized && elapsed_since_last >= Duration::from_secs(SELF_TRIM_COOLDOWN_SECS) {
        return true;
    }
    elapsed_since_last >= Duration::from_secs(SELF_TRIM_PERIODIC_SECS)
}

pub fn maybe_self_trim(last: &mut Instant, startup_frames: u32, just_minimized: bool) {
    if should_trigger_self_trim(last.elapsed(), startup_frames, just_minimized) {
        let ok = crate::core::win32::self_trim_working_set();
        if ok {
            *last = Instant::now();
        } else {
            *last = Instant::now()
                .checked_sub(Duration::from_secs(SELF_TRIM_PERIODIC_SECS - SELF_TRIM_COOLDOWN_SECS))
                .unwrap_or_else(Instant::now);
        }
    }
}
