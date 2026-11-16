//! Self-trim policy for PULSE working set.
//!
//! Two-tier working-set trim policy:
//! 1. One-shot startup self-trim ~5 seconds after launch (`STARTUP_TRIM_DONE`), performed
//!    even while the window is visible, to shed startup init pages.
//! 2. Edge-triggered minimize trim when the window is minimized (guarded by cooldown).
//!
//! Also includes periodic timer and failed-trim backoff. Resize must never trigger a trim.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

pub const SELF_TRIM_PERIODIC_SECS: u64 = 60;
pub const SELF_TRIM_COOLDOWN_SECS: u64 = 10;
pub const SELF_TRIM_FAIL_BACKOFF_SECS: u64 = 30;
pub const STARTUP_SELF_TRIM_DELAY_SECS: u64 = 5;

static STARTUP_TRIM_DONE: AtomicBool = AtomicBool::new(false);
static STARTUP_INSTANT: OnceLock<Instant> = OnceLock::new();

pub fn should_trigger_self_trim(
    elapsed_since_last: Duration,
    _startup_frames: u32,
    just_minimized: bool,
) -> bool {
    if just_minimized && elapsed_since_last >= Duration::from_secs(SELF_TRIM_COOLDOWN_SECS) {
        return true;
    }
    elapsed_since_last >= Duration::from_secs(SELF_TRIM_PERIODIC_SECS)
}

pub fn next_last_after_attempt(now: Instant, ok: bool) -> Instant {
    if ok {
        now
    } else {
        now - Duration::from_secs(SELF_TRIM_PERIODIC_SECS - SELF_TRIM_FAIL_BACKOFF_SECS)
    }
}

pub fn maybe_self_trim(
    last: &mut Instant,
    startup_frames: u32,
    just_minimized: bool,
    is_minimized: bool,
) {
    let start = STARTUP_INSTANT.get_or_init(Instant::now);
    if !STARTUP_TRIM_DONE.load(Ordering::Relaxed)
        && start.elapsed() >= Duration::from_secs(STARTUP_SELF_TRIM_DELAY_SECS)
    {
        STARTUP_TRIM_DONE.store(true, Ordering::Relaxed);
        let _ = crate::core::win32::self_trim_working_set();
        *last = Instant::now();
        return;
    }
    if !should_trigger_self_trim(last.elapsed(), startup_frames, just_minimized) {
        return;
    }
    if is_minimized {
        let ok = crate::core::win32::self_trim_working_set();
        *last = next_last_after_attempt(Instant::now(), ok);
    } else {
        *last = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn frame_count_shortcut_does_not_trigger_early() {
        // The early ~1s frame-count shortcut must not fire before the 5s wall-clock timer.
        assert!(!should_trigger_self_trim(Duration::from_secs(1), 60, false));
        assert!(!should_trigger_self_trim(Duration::from_secs(4), 60, false));
        assert!(!should_trigger_self_trim(Duration::from_secs(1), 120, false));
    }

    #[test]
    fn startup_self_trim_delay_is_five_seconds() {
        assert_eq!(STARTUP_SELF_TRIM_DELAY_SECS, 5);
    }

    #[test]
    fn manual_trim_result_updates_last_via_backoff_helper() {
        use std::time::Instant;
        let now = Instant::now();
        assert_eq!(next_last_after_attempt(now, true), now);
        let failed = next_last_after_attempt(now, false);
        assert!(now.duration_since(failed) > Duration::from_secs(0));
    }

    #[test]
    fn minimized_trim_still_respects_cooldown() {
        assert!(!should_trigger_self_trim(Duration::from_secs(3), 0, true));
        assert!(should_trigger_self_trim(Duration::from_secs(10), 0, true));
    }
}
