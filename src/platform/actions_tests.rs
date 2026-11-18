//! Unit tests for the Memory Actions Engine and Explorer path sanitizer.
mod tests {
    use crate::platform::actions::{open_process_file_location, terminate_process_group};
    #[cfg(target_os = "windows")]
    use crate::platform::handle::{is_app_elevated, query_creation_time_by_pid, SafeHandle};
    use crate::platform::handle::{is_protected_process, verify_pid_safety_gate};
    use crate::platform::location::{
        open_verified_process_location, sanitize_explorer_location_path,
    };
    use crate::platform::target::ProcessTarget;
    use crate::platform::trim::{
        map_win32_error, trim_process_group, trim_processes_working_sets, trim_working_set_outcome,
        TrimSkipReason,
    };
    #[cfg(target_os = "windows")]
    use windows_sys::Win32::System::Threading::PROCESS_QUERY_LIMITED_INFORMATION;

    fn target(pid: u32, name: &str) -> ProcessTarget {
        ProcessTarget {
            pid,
            creation_time: Some(133_000_000_000_000_000),
            name: name.to_string(),
            exe_path: None,
        }
    }

    #[test]
    fn test_safety_gate_protected_and_allowed_table() {
        let self_pid = std::process::id();
        let cases: &[(u32, &str, bool)] = &[
            (0, "system", true),
            (4, "system", true),
            (self_pid, "pulse_manager", true),
            (123, "csrss.exe", true),
            (456, "LSASS.EXE", true),
            (789, "Memory Compression", true),
            (111, "vgc.exe", true),
            (222, "EasyAntiCheat_EOS.exe", true),
            (99999, "notepad.exe", false),
            (88888, "chrome.exe", false),
            (123, r"C:\Windows\System32\csrss.exe", true),
            (222, "D:/Games/EasyAntiCheat_EOS.exe", true),
            (456, r"C:\Windows\System32\lsass.exe", true),
            (100, "lsaiso.exe", true),
            (101, "startmenuexperiencehost.exe", true),
            (102, "searchhost.exe", true),
            (103, "shellexperiencehost.exe", true),
            (104, "textinputhost.exe", true),
            (105, "conhost.exe", true),
        ];
        for (pid, name, blocked) in cases {
            assert_eq!(
                verify_pid_safety_gate(*pid, name).is_err(),
                *blocked,
                "pid={pid} name={name}"
            );
            assert_eq!(is_protected_process(*pid, name), *blocked, "pid={pid} name={name}");
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_safe_handle_refuses_system_pids() {
        for pid in [0, 4] {
            assert!(
                SafeHandle::open(PROCESS_QUERY_LIMITED_INFORMATION, pid).is_none(),
                "pid {pid}"
            );
        }
        let handle = SafeHandle::from_raw(42 as _).expect("valid handle");
        assert_eq!(handle.as_raw(), 42 as _);
        std::mem::forget(handle);
        assert!(SafeHandle::from_raw(0).is_none());
        let a = is_app_elevated();
        assert_eq!(a, is_app_elevated());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_win32_error_mapping() {
        assert_eq!(map_win32_error(5), TrimSkipReason::AccessDenied);
        assert_ne!(map_win32_error(5), TrimSkipReason::Protected);
        assert_eq!(map_win32_error(6), TrimSkipReason::NotFoundOrMismatch);
        assert_eq!(map_win32_error(87), TrimSkipReason::NotFoundOrMismatch);
        assert_eq!(map_win32_error(123), TrimSkipReason::Other);
        assert_eq!(map_win32_error(0xFFFF), TrimSkipReason::Other);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_explorer_sanitizer_rejects_and_accepts() {
        for bad in [
            "C:\\Temp\\bad\"quote.exe",
            "C:\\Temp\\bad\0nul.exe",
            "C:\\Temp\\bad\r\n.exe",
            "",
            "   ",
            "relative\\path.exe",
        ] {
            assert!(sanitize_explorer_location_path(bad).is_err(), "expected reject: {bad:?}");
        }
        assert!(sanitize_explorer_location_path(r"C:\nonexistent\fake.exe").is_err());
        let exe = std::env::current_exe().expect("current exe");
        let exe_s = exe.to_string_lossy().to_string();
        let ok = sanitize_explorer_location_path(&exe_s).expect("current exe should be valid");
        assert!(std::path::Path::new(&ok).exists());
        // Drive root backslash must be preserved.
        let root = sanitize_explorer_location_path(r"C:\").expect("C:\\ should be valid root");
        assert_eq!(root, r"C:\");
        let root2 = normalize_check_drive_root(r"C:\\");
        assert_eq!(root2, r"C:\");
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_explorer_sanitizer_rejects_invalid() {
        for bad in ["C:\\Temp\\bad\"quote.exe", "bad\0nul", "bad\r\n", "", "   ", "relative\\x"] {
            assert!(sanitize_explorer_location_path(bad).is_err(), "expected reject: {bad:?}");
        }
    }

    #[cfg(target_os = "windows")]
    fn normalize_check_drive_root(raw: &str) -> String {
        // Test helper mirrors drive-root normalization behavior via sanitizer-visible outcome.
        // Accept only existing roots in env; for synthetic assertion use string logic via Path.
        let s = raw.trim_end_matches('\\');
        if s.len() == 2 && s.as_bytes()[1] == b':' {
            format!(r"{}\", s)
        } else {
            raw.to_string()
        }
    }

    #[test]
    fn test_trim_batch_structured_counts_no_silent_skip() {
        let targets = vec![
            target(0, "system"),
            target(4, "system"),
            target(std::process::id(), "pulse_manager"),
            target(999_999_991, "notepad.exe"),
        ];
        let r = trim_processes_working_sets(&targets);
        let r2 = trim_process_group(&targets);
        assert_eq!(r, r2);
        assert_eq!(r.trimmed_count, 0);
        assert!(r.skipped_protected >= 3, "PIDs 0/4/self must be protected");
        assert_eq!(
            r.skipped_total(),
            r.skipped_protected
                + r.skipped_access_denied
                + r.skipped_not_found_or_mismatch
                + r.skipped_other
        );
        assert!(r.skipped_total() >= 4);
    }

    #[test]
    fn test_terminate_batch_structured_counts_no_abort() {
        let targets = vec![
            target(0, "system"),
            target(4, "system"),
            target(std::process::id(), "pulse_manager"),
            target(999_999_992, "notepad.exe"),
        ];
        let r = terminate_process_group(&targets);
        assert_eq!(r.succeeded, 0);
        assert_eq!(r.total, 4);
        assert!(r.skipped_protected >= 3, "PIDs 0/4/self must be protected");
        assert_eq!(
            r.skipped_total(),
            r.skipped_protected
                + r.skipped_access_denied
                + r.skipped_not_found_or_mismatch
                + r.skipped_other
        );
        assert!(r.skipped_total() == 4);
        let msg = r.format_status();
        assert!(msg.contains("Terminated 0/4"));
        assert!(!msg.eq("Terminated 0/4"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_creation_time_is_stable_for_own_process() {
        let pid = std::process::id();
        let first = query_creation_time_by_pid(pid).expect("own creation time");
        assert_ne!(first, 0);
        assert_eq!(query_creation_time_by_pid(pid), Some(first));
    }

    #[test]
    fn test_open_verified_rejects_missing_zero_mismatched_without_launch() {
        assert!(open_verified_process_location(&[(12345, None)], "notepad.exe").is_err());
        assert!(open_verified_process_location(&[(12345, Some(0))], "notepad.exe").is_err());
        assert!(open_verified_process_location(&[(999_999_999, Some(123_456_789))], "notepad.exe")
            .is_err());
        assert!(open_verified_process_location(&[(4, Some(123_456_789))], "System").is_err());
        assert!(
            open_process_file_location(std::path::Path::new(r"C:\nonexistent\fake.exe")).is_err()
        );
    }

    #[test]
    fn test_sanitizer_rejects_quotes_nul_control_and_relative() {
        assert!(sanitize_explorer_location_path("C:\\Temp\\has\"quote.exe").is_err());
        assert!(sanitize_explorer_location_path("C:\\Temp\\has\0nul.exe").is_err());
        assert!(sanitize_explorer_location_path("C:\\Temp\\has\r.exe").is_err());
        assert!(sanitize_explorer_location_path("C:\\Temp\\has\n.exe").is_err());
        assert!(sanitize_explorer_location_path("").is_err());
        assert!(sanitize_explorer_location_path("relative\\path.exe").is_err());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_win32_error_mapping_never_protected() {
        for code in [0u32, 2, 5, 6, 87, 123, 0xFFFF] {
            assert_ne!(map_win32_error(code), TrimSkipReason::Protected, "code {code}");
        }
        assert_eq!(map_win32_error(5), TrimSkipReason::AccessDenied);
        assert_eq!(map_win32_error(6), TrimSkipReason::NotFoundOrMismatch);
        assert_eq!(map_win32_error(87), TrimSkipReason::NotFoundOrMismatch);
        assert_eq!(map_win32_error(123), TrimSkipReason::Other);
        assert_eq!(map_win32_error(0xFFFF), TrimSkipReason::Other);
    }

    #[test]
    fn test_trim_batch_all_protected_reports_primary_reason() {
        let r = trim_processes_working_sets(&[target(0, "system"), target(4, "system")]);
        assert_eq!(r.trimmed_count, 0);
        assert_eq!(r.skipped_protected, 2);
        assert_eq!(r.skipped_total(), 2);
    }

    #[test]
    fn test_terminate_report_format_status_cases() {
        let mut r = crate::platform::actions::TerminateReport {
            succeeded: 0,
            total: 1,
            ..Default::default()
        };
        r.skipped_access_denied = 1;
        assert_eq!(r.format_status(), "Terminated 0/1 (access denied)");
        let partial = crate::platform::actions::TerminateReport {
            succeeded: 2,
            total: 3,
            skipped_access_denied: 1,
            ..Default::default()
        };
        assert_eq!(partial.format_status(), "Terminated 2/3 (1 access denied)");
        let ok = crate::platform::actions::TerminateReport {
            succeeded: 2,
            total: 2,
            ..Default::default()
        };
        assert_eq!(ok.format_status(), "Terminated 2/2");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_trim_post_sample_missing_classifies_not_found() {
        let res = trim_working_set_outcome(Some(4096), true, 0, None);
        assert_eq!(res, Err(TrimSkipReason::NotFoundOrMismatch));
    }
}
