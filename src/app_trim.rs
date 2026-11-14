//! Self-trim policy: one-shot startup trim (~5s after launch), minimize cooldown trim,
//! periodic timer, and failed-trim backoff. Resize must never trigger a trim.

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
