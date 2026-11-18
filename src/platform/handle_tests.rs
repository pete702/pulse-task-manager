//! Unit tests for path-aware safety gate and full-path identity helpers.

#[cfg(test)]
mod tests {
    use crate::platform::handle::{
        critical_query_failed_code, next_path_buf_cap, verify_pid_safety_gate_ex,
    };
    use crate::platform::protected::full_paths_match;

    #[test]
    fn test_path_aware_safety_gate_svchost() {
        assert!(verify_pid_safety_gate_ex(1234, "svchost.exe", None).is_err());
        assert!(verify_pid_safety_gate_ex(
            1234,
            "svchost.exe",
            Some(r"C:\Windows\System32\svchost.exe")
        )
        .is_err());
        assert!(verify_pid_safety_gate_ex(
            1234,
            "svchost.exe",
            Some(r"C:\Users\Alice\AppData\Local\Temp\svchost.exe")
        )
        .is_ok());
        assert!(verify_pid_safety_gate_ex(
            1234,
            "svchost.exe",
            Some(r"C:\WindowsFake\svchost.exe")
        )
        .is_ok());
    }

    #[test]
    fn test_path_aware_safety_gate_defender() {
        assert!(verify_pid_safety_gate_ex(
            1234,
            "MpCmdRun.exe",
            Some(r"C:\ProgramData\Microsoft\Windows Defender\Platform\4.18\MpCmdRun.exe")
        )
        .is_err());
        assert!(
            verify_pid_safety_gate_ex(1234, "MpCmdRun.exe", Some(r"C:\Temp\MpCmdRun.exe")).is_ok()
        );
    }

    #[test]
    fn test_full_path_comparison_rejects_same_basename_different_directory() {
        assert!(!full_paths_match(r"C:\App1\worker.exe", r"C:\App2\worker.exe"));
        assert!(full_paths_match(r"C:\App1\worker.exe", r"c:/app1/WORKER.EXE"));
        assert!(!full_paths_match(r"C:\App1\worker.exe", r"C:\App1\worker2.exe"));
    }

    #[test]
    fn test_next_path_buf_cap() {
        assert_eq!(next_path_buf_cap(512, 122), Some(1024));
        assert_eq!(next_path_buf_cap(1024, 122), Some(2048));
        assert_eq!(next_path_buf_cap(32768, 122), None);
        assert_eq!(next_path_buf_cap(512, 5), None);
        assert_eq!(next_path_buf_cap(512, 0), None);
    }

    #[test]
    fn test_critical_query_failed_code() {
        assert_eq!(critical_query_failed_code(0, 87), Some(87));
        assert_eq!(critical_query_failed_code(1, 87), None);
    }
}
