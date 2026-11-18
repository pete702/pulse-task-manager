//! Latency probe tests.

use super::*;
static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
#[test]
fn probe_defaults_and_dns_guard() {
    let _lock = TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|e| e.into_inner());
    set_probe_enabled(true);
    assert!(probe_enabled());
    let _ = probe_state();
    {
        let mut g = dns_slot().lock().unwrap_or_else(|e| e.into_inner());
        g.0 = DnsFlushState::Success;
        g.1 = Some(std::time::Instant::now());
    }
    assert_eq!(dns_flush_state(), DnsFlushState::Success);
    {
        let g = dns_slot().lock().unwrap_or_else(|e| e.into_inner());
        assert!(
            g.0 == DnsFlushState::Success && g.1.is_some_and(|t| t.elapsed().as_millis() < 3000)
        );
    }
    {
        let mut g = dns_slot().lock().unwrap_or_else(|e| e.into_inner());
        g.0 = DnsFlushState::Idle;
        g.1 = None;
    }
    assert_eq!(dns_flush_state(), DnsFlushState::Idle);
    {
        let g = dns_slot().lock().unwrap_or_else(|e| e.into_inner());
        assert!(
            !(g.0 == DnsFlushState::Success && g.1.is_some_and(|t| t.elapsed().as_millis() < 3000))
        );
    }
}
#[test]
fn timeout_clears_ping_and_jitter() {
    let _lock = TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|e| e.into_inner());
    set_probe_enabled(true);
    record_sample(Some(20.0));
    assert_eq!(probe_state().0, Some(20.0));
    assert!(probe_state().1.is_none_or(|j| (0.0..=1000.0).contains(&j)));
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

#[test]
fn sanitize_probe_target_strict() {
    assert_eq!(sanitize_probe_target(" 1.1.1.1:53 "), Some("1.1.1.1:53"));
    assert_eq!(sanitize_probe_target("dns.google"), Some("dns.google"));
    assert_eq!(sanitize_probe_target(""), None);
    assert_eq!(sanitize_probe_target("bad host"), None);
    assert_eq!(sanitize_probe_target("a;b"), None);
    assert_eq!(sanitize_probe_target("x\"y"), None);
    assert_eq!(sanitize_probe_target("a\nb"), None);
    assert_eq!(sanitize_probe_target(&"a".repeat(254)), None);
}

#[test]
fn probe_helpers_basic() {
    let _lock = TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(probe_timeout_ms(), 2000);
    assert!(!resolve_probe_targets().is_empty());
    assert!(!is_probe_watchdog_expired());
}
