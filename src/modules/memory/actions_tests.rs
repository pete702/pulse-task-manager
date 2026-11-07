//! Unit tests for the Memory Actions Engine (actions.rs) and Safety Gate (safe_handle.rs).
mod tests {
    #[cfg(target_os = "windows")]
    use crate::modules::memory::actions::open_process_file_location;
    use crate::modules::memory::actions::{
        purge_system_working_sets, terminate_process_group, trim_process_group,
    };
    use crate::modules::memory::process_target::ProcessTarget;
    #[cfg(target_os = "windows")]
    use crate::modules::memory::safe_handle::{
        is_app_elevated, query_creation_time_by_pid, SafeHandle,
    };
    use crate::modules::memory::safe_handle::{is_protected_process, verify_pid_safety_gate};
    #[cfg(target_os = "windows")]
    use std::path::PathBuf;
    #[cfg(target_os = "windows")]
    use windows_sys::Win32::System::Threading::PROCESS_QUERY_LIMITED_INFORMATION;
    fn make_test_target(pid: u32, name: &str) -> ProcessTarget {
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
            let err = verify_pid_safety_gate(*pid, name).is_err();
            assert_eq!(err, *blocked, "pid={pid} name={name}");
        }
    }
    #[test]
    fn test_is_protected_process_table() {
        let self_pid = std::process::id();
        let cases: &[(u32, &str, bool)] = &[
            (0, "system", true),
            (4, "system", true),
            (self_pid, "pulse", true),
            (123, "csrss.exe", true),
            (123, "vgc.exe", true),
            (99999, "notepad.exe", false),
            (88888, "chrome.exe", false),
        ];
        for (pid, name, blocked) in cases {
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
    }
    #[cfg(target_os = "windows")]
    #[test]
    fn test_safe_handle_as_raw() {
        let handle = SafeHandle::from_raw(42 as _).expect("valid handle");
        assert_eq!(handle.as_raw(), 42 as _);
        std::mem::forget(handle);
        assert!(SafeHandle::from_raw(0).is_none());
    }
    #[cfg(target_os = "windows")]
    #[test]
    fn test_is_app_elevated_executes_cleanly() {
        let a = is_app_elevated();
        let b = is_app_elevated();
        assert_eq!(a, b);
    }
    #[test]
    fn test_trim_process_group_skips_protected() {
        let targets = vec![
            make_test_target(0, "system"),
            make_test_target(4, "system"),
            make_test_target(std::process::id(), "pulse"),
            make_test_target(123, "csrss.exe"),
            make_test_target(99999, "notepad.exe"),
        ];
        let report = trim_process_group(&targets);
        assert_eq!(report.skipped_protected, 4);
        assert_eq!(report.trimmed_count, 0);
    }
    #[test]
    fn test_purge_system_working_sets_matches_trim() {
        let targets = vec![make_test_target(99999, "notepad.exe")];
        let r1 = purge_system_working_sets(&targets);
        let r2 = trim_process_group(&targets);
        assert_eq!(r1.trimmed_count, r2.trimmed_count);
        assert_eq!(r1.skipped_protected, r2.skipped_protected);
        assert_eq!(r1.skipped_access_denied, r2.skipped_access_denied);
    }
    #[test]
    fn test_terminate_process_group_blocks_protected() {
        let result = terminate_process_group(&[make_test_target(0, "system")]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("OS kernel process is protected"));
    }
    #[cfg(target_os = "windows")]
    #[test]
    fn test_creation_time_is_stable_for_own_process() {
        let pid = std::process::id();
        let first = query_creation_time_by_pid(pid).expect("own creation time");
        assert_ne!(first, 0);
        assert_eq!(query_creation_time_by_pid(pid), Some(first));
    }
    #[cfg(target_os = "windows")]
    #[test]
    fn test_open_process_file_location_missing() {
        let result = open_process_file_location(&PathBuf::from(r"C:\nonexistent\fake.exe"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }
}
