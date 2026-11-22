//! Tests for the pure SCM/startup filter helpers and live query safety.
use super::*;

#[test]
fn service_state_labels_are_exact() {
    assert_eq!(format_service_state(4), "RUNNING");
    assert_eq!(format_service_state(1), "STOPPED");
    assert_eq!(format_service_state(2), "START_PENDING");
    assert_eq!(format_service_state(3), "STOP_PENDING");
    assert_eq!(format_service_state(5), "CONTINUE_PENDING");
    assert_eq!(format_service_state(6), "PAUSE_PENDING");
    assert_eq!(format_service_state(7), "PAUSED");
    assert_eq!(format_service_state(99), "UNKNOWN");
    assert!(is_pending_state(2));
    assert!(is_pending_state(3));
    assert!(is_pending_state(5));
    assert!(is_pending_state(6));
    assert!(!is_pending_state(4));
    assert!(!is_pending_state(99));
}

#[test]
fn service_filter_matches_all_fields() {
    assert!(matches_service_filter("", "svc", "Display", "RUNNING", 0));
    assert!(matches_service_filter("SV", "svc", "Display", "RUNNING", 0));
    assert!(matches_service_filter("display", "svc", "Display", "RUNNING", 0));
    assert!(matches_service_filter("running", "svc", "Display", "RUNNING", 0));
    assert!(matches_service_filter("4242", "svc", "Display", "RUNNING", 4242));
    assert!(!matches_service_filter("missing", "svc", "Display", "RUNNING", 4242));
}

#[test]
fn startup_filter_matches_scope_name_and_command() {
    assert!(matches_startup_filter("", "HKCU", "App", "C:\\app.exe"));
    assert!(matches_startup_filter("hkcu", "HKCU", "App", "C:\\app.exe"));
    assert!(matches_startup_filter("app", "HKCU", "App", "C:\\app.exe"));
    assert!(matches_startup_filter("APP.EXE", "HKCU", "App", "C:\\app.exe"));
    assert!(!matches_startup_filter("zzz", "HKCU", "App", "C:\\app.exe"));
}

#[test]
#[cfg(target_os = "windows")]
fn more_data_retry_grows_and_caps() {
    use super::{retry_capacity, ERROR_MORE_DATA};
    assert_eq!(retry_capacity(0, 256, 4096, 8192), None);
    assert_eq!(retry_capacity(ERROR_MORE_DATA, 256, 1024, 4096), Some(1024));
    assert_eq!(retry_capacity(ERROR_MORE_DATA, 256, 128, 4096), Some(512));
    assert_eq!(retry_capacity(ERROR_MORE_DATA, 4096, 8192, 4096), None);
}

#[test]
#[cfg(target_os = "windows")]
fn utf16le_bytes_decode_stops_at_nul() {
    use super::decode_utf16_le;
    let units: [u16; 4] = ['A' as u16, 'B' as u16, 0, 'C' as u16];
    let bytes: Vec<u8> = units.iter().flat_map(|unit| unit.to_le_bytes()).collect();
    assert_eq!(decode_utf16_le(&bytes), "AB");
    assert_eq!(decode_utf16_le(&[]), "");
}

#[test]
#[cfg(target_os = "windows")]
fn byte_counts_convert_to_u64_words() {
    assert_eq!(bytes_to_words(0), 0);
    assert_eq!(bytes_to_words(1), 1);
    assert_eq!(bytes_to_words(8), 1);
    assert_eq!(bytes_to_words(9), 2);
    assert_eq!(bytes_to_words(40_000), 5000);
}

#[test]
#[cfg(target_os = "windows")]
fn live_scm_lists_real_services_without_partial_data() {
    let services = query_windows_services(true);
    assert!(services.len() >= 10, "expected >= 10 real services, got {}", services.len());
    assert!(!services_partial(), "live SCM enumeration must not be partial");
    for entry in &services {
        assert!(!entry.name.is_empty());
        assert!(!entry.display_name.is_empty());
    }
    let _ = query_startup_entries(false);
}

#[test]
#[cfg(not(target_os = "windows"))]
fn live_queries_do_not_panic() {
    assert!(query_windows_services(false).is_empty());
    assert!(query_startup_entries(false).is_empty());
}
