//! Deterministic Memory Actions Engine for PULSE.
//! Provides verified working set trim, batch operations, termination, and file location opening.

use std::path::Path;
use windows_sys::Win32::Foundation::{GetLastError, HANDLE};
use windows_sys::Win32::System::ProcessStatus::{
    K32EmptyWorkingSet, K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
};
use windows_sys::Win32::System::Threading::{
    TerminateProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_QUOTA, PROCESS_TERMINATE,
};

use crate::modules::memory::process_target::ProcessTarget;
use crate::modules::memory::safe_handle::{
    is_handle_critical, verify_handle_identity, verify_pid_safety_gate, SafeHandle,
};

/// Report of a reclaim operation (trim/purge).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReclaimReport {
    pub trimmed_count: usize,
    pub skipped_protected: usize,
    pub skipped_access_denied: usize,
    pub reclaimed_bytes: u64,
}

/// Reason why a trim operation was skipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrimSkipReason {
    Protected,
    AccessDenied,
    CriticalProcess,
}

/// Queries the working set size of a process via K32GetProcessMemoryInfo.
#[allow(unsafe_code)]
unsafe fn query_working_set(handle: HANDLE) -> Option<u64> {
    let mut pmc: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
    pmc.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
    let ok = K32GetProcessMemoryInfo(
        handle,
        &mut pmc as *mut _ as *mut _,
        std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
    );
    if ok != 0 {
        Some(pmc.WorkingSetSize as u64)
    } else {
        None
    }
}

fn identity_matches(handle: &SafeHandle, target: &ProcessTarget) -> bool {
    verify_handle_identity(handle, &target.name, target.creation_time, target.exe_path.as_deref())
}

/// Per-process verified working set trim with TOCTOU identity verification.
#[allow(unsafe_code)]
pub fn trim_single_pid(target: &ProcessTarget) -> Result<u64, TrimSkipReason> {
    if verify_pid_safety_gate(target.pid, &target.name).is_err() {
        return Err(TrimSkipReason::Protected);
    }
    let access = PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_QUOTA;
    let handle = SafeHandle::open(access, target.pid).ok_or(TrimSkipReason::AccessDenied)?;
    if is_handle_critical(&handle) {
        return Err(TrimSkipReason::CriticalProcess);
    }
    if !identity_matches(&handle, target) {
        return Err(TrimSkipReason::AccessDenied);
    }
    let ws_before =
        unsafe { query_working_set(handle.as_raw()) }.ok_or(TrimSkipReason::AccessDenied)?;
    let trim_ok = unsafe { K32EmptyWorkingSet(handle.as_raw()) };
    if trim_ok == 0 {
        let err = unsafe { GetLastError() };
        return Err(match err {
            6 => TrimSkipReason::AccessDenied,
            87 => TrimSkipReason::Protected,
            _ => TrimSkipReason::AccessDenied,
        });
    }
    let ws_after =
        unsafe { query_working_set(handle.as_raw()) }.ok_or(TrimSkipReason::AccessDenied)?;
    Ok(ws_before.saturating_sub(ws_after))
}

/// Batch trim for verified process targets.
pub fn trim_process_group(targets: &[ProcessTarget]) -> ReclaimReport {
    let mut report = ReclaimReport::default();
    for target in targets {
        match trim_single_pid(target) {
            Ok(reclaimed) => {
                report.trimmed_count += 1;
                report.reclaimed_bytes += reclaimed;
            }
            Err(TrimSkipReason::Protected) | Err(TrimSkipReason::CriticalProcess) => {
                report.skipped_protected += 1
            }
            Err(TrimSkipReason::AccessDenied) => report.skipped_access_denied += 1,
        }
    }
    report
}

/// Alias for trim_process_group (system-wide working set purge).
pub fn purge_system_working_sets(targets: &[ProcessTarget]) -> ReclaimReport {
    trim_process_group(targets)
}

/// Terminates verified targets. Returns the count terminated, or a gate error.
pub fn terminate_process_group(targets: &[ProcessTarget]) -> Result<usize, String> {
    for target in targets {
        if let Err(reason) = verify_pid_safety_gate(target.pid, &target.name) {
            return Err(format!("PID {} ({}): {}", target.pid, target.name, reason));
        }
    }
    let mut terminated = 0;
    for target in targets {
        let access = PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION;
        if let Some(handle) = SafeHandle::open(access, target.pid) {
            if is_handle_critical(&handle) || !identity_matches(&handle, target) {
                continue;
            }
            #[allow(unsafe_code)]
            if unsafe { TerminateProcess(handle.as_raw(), 1) } != 0 {
                terminated += 1;
            }
        }
    }
    Ok(terminated)
}

/// Opens the containing folder of an executable in Explorer with the file selected.
#[cfg(target_os = "windows")]
pub fn open_process_file_location(exe_path: &Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let clean = exe_path.to_string_lossy().replace('"', "");
    let trimmed = clean.trim_end_matches('\\').trim().to_string();
    if trimmed.is_empty() || trimmed.contains('\0') {
        return Err("Invalid executable path (empty or contains NUL)".to_string());
    }
    #[cfg(test)]
    {
        if !std::path::Path::new(&trimmed).exists() {
            return Err("Executable path does not exist".to_string());
        }
    }
    #[cfg(not(test))]
    {
        let trimmed = trimmed.clone();
        std::thread::spawn(move || {
            let system_root = std::env::var_os("SystemRoot")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from(r"C:\Windows"));
            let _ = std::process::Command::new(system_root.join("explorer.exe"))
                .raw_arg(format!("/select,\"{}\"", trimmed))
                .spawn();
        });
        Ok(())
    }
    #[cfg(test)]
    {
        let system_root = std::env::var_os("SystemRoot")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from(r"C:\Windows"));
        std::process::Command::new(system_root.join("explorer.exe"))
            .raw_arg(format!("/select,\"{}\"", trimmed))
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("Failed to launch Explorer: {}", e))
    }
}

#[cfg(not(target_os = "windows"))]
pub fn open_process_file_location(_exe_path: &Path) -> Result<(), String> {
    Err("open_process_file_location is only supported on Windows".to_string())
}
