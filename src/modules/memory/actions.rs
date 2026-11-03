//! Deterministic Memory Actions Engine for PULSE.
//! Provides per-PID working set trim, batch operations, process termination, and file location opening.

use std::os::windows::process::CommandExt;
use std::path::Path;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::ProcessStatus::{
    K32EmptyWorkingSet, K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
};
use windows_sys::Win32::System::Threading::{
    TerminateProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_QUOTA, PROCESS_TERMINATE,
};

use crate::modules::memory::safe_handle::{is_handle_critical, verify_pid_safety_gate, SafeHandle};

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
}

/// Queries the working set size of a process via K32GetProcessMemoryInfo.
#[allow(unsafe_code)]
unsafe fn query_working_set(handle: HANDLE) -> Option<u64> {
    let mut pmc: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
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

/// Per-PID deterministic working set trim.
/// Checks kernel critical flag after opening handle; returns AccessDenied if post-trim query fails
/// (never inflates reclaimed bytes with unwrap_or(0)).
#[allow(unsafe_code)]
pub fn trim_single_pid(pid: u32, name: &str) -> Result<u64, TrimSkipReason> {
    if verify_pid_safety_gate(pid, name).is_err() {
        return Err(TrimSkipReason::Protected);
    }
    let access = PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_QUOTA;
    let handle = SafeHandle::open(access, pid).ok_or(TrimSkipReason::AccessDenied)?;
    // Harden: reject kernel-critical processes even if they pass name-based gate
    if is_handle_critical(&handle) {
        return Err(TrimSkipReason::Protected);
    }
    let ws_before =
        unsafe { query_working_set(handle.as_raw()) }.ok_or(TrimSkipReason::AccessDenied)?;
    #[allow(unsafe_code)]
    let trim_ok = unsafe { K32EmptyWorkingSet(handle.as_raw()) };
    if trim_ok == 0 {
        return Err(TrimSkipReason::AccessDenied);
    }
    // FIX: Do NOT use unwrap_or(0) — a failed post-trim query means we cannot measure,
    // so return AccessDenied instead of inflating reclaimed = ws_before.
    let ws_after =
        unsafe { query_working_set(handle.as_raw()) }.ok_or(TrimSkipReason::AccessDenied)?;
    Ok(ws_before.saturating_sub(ws_after))
}

/// Batch trim for a group of processes.
pub fn trim_process_group(targets: &[(u32, String)]) -> ReclaimReport {
    let mut report = ReclaimReport::default();
    for (pid, name) in targets {
        match trim_single_pid(*pid, name) {
            Ok(reclaimed) => {
                report.trimmed_count += 1;
                report.reclaimed_bytes += reclaimed;
            }
            Err(TrimSkipReason::Protected) => report.skipped_protected += 1,
            Err(TrimSkipReason::AccessDenied) => report.skipped_access_denied += 1,
        }
    }
    report
}

/// Alias for trim_process_group (system-wide working set purge).
pub fn purge_system_working_sets(targets: &[(u32, String)]) -> ReclaimReport {
    trim_process_group(targets)
}

/// Terminates a group of processes.
pub fn terminate_process_group(targets: &[(u32, String)]) -> Result<usize, String> {
    for (pid, name) in targets {
        if let Err(reason) = verify_pid_safety_gate(*pid, name) {
            return Err(format!("PID {} ({}): {}", pid, name, reason));
        }
    }
    let mut terminated = 0;
    for (pid, _) in targets {
        if let Some(h) = SafeHandle::open(PROCESS_TERMINATE, *pid) {
            #[allow(unsafe_code)]
            if unsafe { TerminateProcess(h.as_raw(), 1) } != 0 {
                terminated += 1;
            }
        }
    }
    Ok(terminated)
}

/// Opens the containing folder of an executable in Explorer with the file selected.
/// Uses raw_arg for correct quoting of paths with spaces (e.g., "C:\Program Files\...").
pub fn open_process_file_location(exe_path: &Path) -> Result<(), String> {
    if !exe_path.exists() {
        return Err("Executable path does not exist".to_string());
    }
    let clean = exe_path.to_string_lossy().replace('"', "");
    std::process::Command::new(r"C:\Windows\explorer.exe")
        .raw_arg(format!("/select,\"{}\"", clean))
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Failed to launch Explorer: {}", e))
}
