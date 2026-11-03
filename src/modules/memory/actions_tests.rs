//! Unit tests for the Memory Actions Engine (actions.rs) and Safety Gate (safe_handle.rs).

use crate::modules::memory::actions::{
    open_process_file_location, purge_system_working_sets, terminate_process_group,
    trim_process_group, ReclaimReport, TrimSkipReason,
};
use crate::modules::memory::safe_handle::{
    is_app_elevated, is_protected_process, verify_pid_safety_gate, SafeHandle,
};
use std::path::PathBuf;
use windows_sys::Win32::System::Threading::PROCESS_QUERY_LIMITED_INFORMATION;

#[test]
fn test_verify_pid_safety_gate_blocks_pid_0() {
    assert!(verify_pid_safety_gate(0, "system").is_err());
}

#[test]
fn test_verify_pid_safety_gate_blocks_pid_4() {
    assert!(verify_pid_safety_gate(4, "system").is_err());
}

#[test]
fn test_verify_pid_safety_gate_blocks_self_pid() {
    let self_pid = std::process::id();
    assert!(verify_pid_safety_gate(self_pid, "pulse_manager").is_err());
}

#[test]
fn test_verify_pid_safety_gate_blocks_csrss() {
    assert!(verify_pid_safety_gate(123, "csrss.exe").is_err());
}

#[test]
fn test_verify_pid_safety_gate_blocks_lsass() {
    assert!(verify_pid_safety_gate(456, "LSASS.EXE").is_err());
}

#[test]
fn test_verify_pid_safety_gate_blocks_memory_compression() {
    assert!(verify_pid_safety_gate(789, "Memory Compression").is_err());
}

#[test]
fn test_verify_pid_safety_gate_blocks_vgc() {
    assert!(verify_pid_safety_gate(111, "vgc.exe").is_err());
}

#[test]
fn test_verify_pid_safety_gate_blocks_easyanticheat_eos() {
    assert!(verify_pid_safety_gate(222, "EasyAntiCheat_EOS.exe").is_err());
}

#[test]
fn test_verify_pid_safety_gate_allows_notepad() {
    assert!(verify_pid_safety_gate(99999, "notepad.exe").is_ok());
}

#[test]
fn test_verify_pid_safety_gate_allows_chrome() {
    assert!(verify_pid_safety_gate(88888, "chrome.exe").is_ok());
}

#[test]
fn test_safe_handle_open_returns_none_on_pid_0() {
    assert!(SafeHandle::open(PROCESS_QUERY_LIMITED_INFORMATION, 0).is_none());
}

#[test]
fn test_safe_handle_open_returns_none_on_pid_4() {
    assert!(SafeHandle::open(PROCESS_QUERY_LIMITED_INFORMATION, 4).is_none());
}

#[test]
fn test_is_app_elevated_executes_cleanly() {
    // Just verify it runs without panic
    let _ = is_app_elevated();
}

#[test]
fn test_trim_process_group_skips_protected() {
    let targets = vec![
        (0, "system".to_string()),
        (4, "system".to_string()),
        (std::process::id(), "pulse".to_string()),
        (123, "csrss.exe".to_string()),
        (99999, "notepad.exe".to_string()),
    ];
    let report = trim_process_group(&targets);
    // First 4 are protected, last likely access denied
    assert_eq!(report.skipped_protected, 4);
    assert_eq!(report.trimmed_count, 0);
}

#[test]
fn test_purge_system_working_sets_matches_trim() {
    let targets = vec![(99999, "notepad.exe".to_string())];
    let r1 = purge_system_working_sets(&targets);
    let r2 = trim_process_group(&targets);
    assert_eq!(r1.trimmed_count, r2.trimmed_count);
    assert_eq!(r1.skipped_protected, r2.skipped_protected);
    assert_eq!(r1.skipped_access_denied, r2.skipped_access_denied);
}

#[test]
fn test_terminate_process_group_blocks_protected() {
    let targets = vec![(0, "system".to_string())];
    let result = terminate_process_group(&targets);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("OS kernel process is protected"));
}

#[test]
fn test_open_process_file_location_missing() {
    let result = open_process_file_location(&PathBuf::from("C:\\nonexistent\\fake.exe"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_reclaim_report_default_and_clone() {
    let r1 = ReclaimReport::default();
    let r2 = r1.clone();
    assert_eq!(r1, r2);
    assert_eq!(r1.trimmed_count, 0);
}

#[test]
fn test_trim_skip_reason_eq() {
    assert_eq!(TrimSkipReason::Protected, TrimSkipReason::Protected);
    assert_eq!(TrimSkipReason::AccessDenied, TrimSkipReason::AccessDenied);
    assert_ne!(TrimSkipReason::Protected, TrimSkipReason::AccessDenied);
}

#[test]
fn test_is_protected_process_returns_true_for_blocked() {
    assert!(is_protected_process(0, "system"));
    assert!(is_protected_process(4, "system"));
    assert!(is_protected_process(std::process::id(), "pulse"));
    assert!(is_protected_process(123, "csrss.exe"));
    assert!(is_protected_process(123, "vgc.exe"));
}

#[test]
fn test_is_protected_process_returns_false_for_allowed() {
    assert!(!is_protected_process(99999, "notepad.exe"));
    assert!(!is_protected_process(88888, "chrome.exe"));
}
