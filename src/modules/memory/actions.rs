//! Deterministic Memory Actions Engine for PULSE.
//! Provides per-PID working set trim, batch operations, process termination, and file location opening.

use std::path::Path;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::ProcessStatus::{
    K32EmptyWorkingSet, K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
};
use windows_sys::Win32::System::Threading::{
    TerminateProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_QUOTA, PROCESS_TERMINATE,
};

use crate::modules::memory::process_model::ProcessMemoryItem;
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

/// Per-PID deterministic working set trim with TOCTOU handle identity verification.
/// Checks kernel critical flag and verifies handle still points to expected executable.
#[allow(unsafe_code)]
pub fn trim_single_pid(item: &ProcessMemoryItem) -> Result<u64, TrimSkipReason> {
    if verify_pid_safety_gate(item.pid, &item.name).is_err() {
        return Err(TrimSkipReason::Protected);
    }
    let access = PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_QUOTA;
    let handle = SafeHandle::open(access, item.pid).ok_or(TrimSkipReason::AccessDenied)?;
    if is_handle_critical(&handle) {
        return Err(TrimSkipReason::CriticalProcess);
    }
    if !verify_handle_identity(&handle, &item.name) {
        return Err(TrimSkipReason::AccessDenied);
    }
    let ws_before =
        unsafe { query_working_set(handle.as_raw()) }.ok_or(TrimSkipReason::AccessDenied)?;
    let trim_ok = unsafe { K32EmptyWorkingSet(handle.as_raw()) };
    if trim_ok == 0 {
        return Err(TrimSkipReason::AccessDenied);
    }
    let ws_after =
        unsafe { query_working_set(handle.as_raw()) }.ok_or(TrimSkipReason::AccessDenied)?;
    Ok(ws_before.saturating_sub(ws_after))
}

/// Batch trim for a group of processes.
pub fn trim_process_group(targets: &[ProcessMemoryItem]) -> ReclaimReport {
    let mut report = ReclaimReport::default();
    for item in targets {
        match trim_single_pid(item) {
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
pub fn purge_system_working_sets(targets: &[ProcessMemoryItem]) -> ReclaimReport {
    trim_process_group(targets)
}

/// Terminates a group of processes with TOCTOU handle identity verification.
/// Returns count of successfully terminated processes, or error if safety gate fails.
pub fn terminate_process_group(
    pids: &[u32],
    snapshot: &[ProcessMemoryItem],
) -> Result<usize, String> {
    for pid in pids {
        let name = snapshot
            .iter()
            .find(|item| item.pid == *pid)
            .map(|i| i.name.as_str())
            .unwrap_or("unknown");
        if let Err(reason) = verify_pid_safety_gate(*pid, name) {
            return Err(format!("PID {} ({}): {}", pid, name, reason));
        }
    }
    let mut terminated = 0;
    for pid in pids {
        let proc_item = match snapshot.iter().find(|item| item.pid == *pid) {
            Some(i) => i,
            None => continue,
        };
        let access = PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION;
        if let Some(handle) = SafeHandle::open(access, *pid) {
            if is_handle_critical(&handle) || !verify_handle_identity(&handle, &proc_item.name) {
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
/// Uses raw_arg for correct quoting of paths with spaces. Resolves explorer.exe via SystemRoot.
#[cfg(target_os = "windows")]
pub fn open_process_file_location(exe_path: &Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    if !exe_path.exists() {
        return Err("Executable path does not exist".to_string());
    }
    let clean = exe_path.to_string_lossy().replace('"', "");
    let trimmed = clean.trim_end_matches('\\');
    if trimmed.is_empty() {
        return Err("Invalid path after trimming".to_string());
    }
    let system_root = std::env::var_os("SystemRoot")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(r"C:\Windows"));
    std::process::Command::new(system_root.join("explorer.exe"))
        .raw_arg(format!("/select,\"{}\"", trimmed))
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Failed to launch Explorer: {}", e))
}

#[cfg(not(target_os = "windows"))]
pub fn open_process_file_location(_exe_path: &Path) -> Result<(), String> {
    Err("open_process_file_location is only supported on Windows".to_string())
}
