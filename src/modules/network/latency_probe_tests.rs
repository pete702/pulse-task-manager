//! Latency probe tests.

use super::*;
static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
#[test]
fn probe_defaults_and_dns_guard() {
    let _lock = TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|e| e.into_inner());
    set_probe_enabled(true);
    assert!(probe_enabled());
    let _ = probe_state();
    trigger_dns_flush();
    let _ = dns_flush_state();
    let _ = dns_flushed_recently();
}
#[test]
fn timeout_clears_ping_and_jitter() {
    let _lock = TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|e| e.into_inner());
    set_probe_enabled(true);
    record_sample(Some(20.0));
    assert_eq!(probe_state().0, Some(20.0));
    assert_eq!(probe_state().1, Some(0.0));
    record_sample(None);
    assert_eq!(probe_state().0, None);
    let _ = probe_state();
}
#[test]
fn disable_clears_history_without_fake_loss() {
    let _lock = TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|e| e.into_inner());
    set_probe_enabled(true);
    record_sample(None);
    record_sample(None);
    set_probe_enabled(false);
    assert_eq!(probe_sample_count(), 0);
    assert_eq!(probe_failures(), 0);
    assert_eq!(probe_state().0, None);
    set_probe_enabled(true);
}
