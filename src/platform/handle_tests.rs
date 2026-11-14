//! Unit tests for path-aware safety gate and full-path identity helpers.

#[cfg(test)]
mod tests {
    use crate::platform::handle::verify_pid_safety_gate_ex;
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
}
