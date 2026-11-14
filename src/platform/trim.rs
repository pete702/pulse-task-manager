//! Working-set trim helpers for PULSE (split from actions.rs).

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{GetLastError, HANDLE};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::ProcessStatus::{
    K32EmptyWorkingSet, K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::{
    GetProcessId, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_QUOTA,
};

#[cfg(not(target_os = "windows"))]
use crate::platform::handle::verify_pid_safety_gate_ex;
#[cfg(target_os = "windows")]
use crate::platform::handle::{
    is_handle_critical, query_process_creation_time, query_process_image_path,
    verify_pid_safety_gate_ex, SafeHandle,
};
#[cfg(target_os = "windows")]
use crate::platform::protected::normalize_process_name;
use crate::platform::target::ProcessTarget;

/// Report of a reclaim operation (trim/purge) with structured skip categories.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReclaimReport {
    pub trimmed_count: usize,
    pub skipped_protected: usize,
    pub skipped_access_denied: usize,
    pub skipped_not_found_or_mismatch: usize,
    pub skipped_other: usize,
    pub reclaimed_bytes: u64,
}

impl ReclaimReport {
    #[cfg(test)]
    pub fn skipped_total(&self) -> usize {
        self.skipped_protected
            + self.skipped_access_denied
            + self.skipped_not_found_or_mismatch
            + self.skipped_other
    }
}

/// Reason why a trim/terminate operation was skipped or failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrimSkipReason {
    Protected,
    AccessDenied,
    NotFoundOrMismatch,
    Other,
}

/// Maps a Win32 error code for handle-open/operation failures.
pub fn map_win32_error(code: u32) -> TrimSkipReason {
    let _ = code;
    TrimSkipReason::AccessDenied
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
unsafe fn query_working_set(handle: HANDLE) -> Option<u64> {
    let mut pmc: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
    pmc.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
    let ok = K32GetProcessMemoryInfo(
        handle,
        &mut pmc as *mut _ as *mut _,
        std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
    );
    (ok != 0).then_some(pmc.WorkingSetSize as u64)
}

pub(crate) fn pre_handle_check(target: &ProcessTarget) -> Result<(), TrimSkipReason> {
    if target.pid == 0 || target.pid == 4 || target.pid == std::process::id() {
        return Err(TrimSkipReason::Protected);
    }
    if let Some(ep) = target.exe_path.as_deref().filter(|p| !p.trim().is_empty()) {
        if verify_pid_safety_gate_ex(target.pid, &target.name, Some(ep)).is_err() {
            return Err(TrimSkipReason::Protected);
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub(crate) fn open_raw_checked(access: u32, pid: u32) -> Result<SafeHandle, TrimSkipReason> {
    let raw = unsafe { OpenProcess(access, 0, pid) };
    if raw == 0 || raw == -1isize as HANDLE {
        let err = unsafe { GetLastError() };
        return Err(map_win32_error(err));
    }
    SafeHandle::from_raw(raw).ok_or(TrimSkipReason::AccessDenied)
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub(crate) fn verify_live_query_handle(
    query: &SafeHandle,
    target: &ProcessTarget,
) -> Result<(), TrimSkipReason> {
    let Some(exp_ct) = target.creation_time.filter(|&t| t != 0) else {
        return Err(TrimSkipReason::NotFoundOrMismatch);
    };
    if query_process_creation_time(query) != Some(exp_ct) {
        return Err(TrimSkipReason::NotFoundOrMismatch);
    }
    let live_path = query_process_image_path(query);
    if is_handle_critical(query) {
        return Err(TrimSkipReason::Protected);
    }
    let pid = unsafe { GetProcessId(query.as_raw()) };
    if pid == 0 {
        return Err(TrimSkipReason::NotFoundOrMismatch);
    }
    let live_name = live_path
        .as_deref()
        .map(normalize_process_name)
        .unwrap_or_else(|| normalize_process_name(&target.name));
    if verify_pid_safety_gate_ex(pid, &live_name, live_path.as_deref()).is_err() {
        return Err(TrimSkipReason::Protected);
    }
    if let Some(exp) = target.exe_path.as_deref().filter(|p| !p.trim().is_empty()) {
        let Some(live) = live_path.as_deref() else {
            return Err(TrimSkipReason::NotFoundOrMismatch);
        };
        if !crate::platform::protected::full_paths_match(exp, live) {
            return Err(TrimSkipReason::NotFoundOrMismatch);
        }
    }
    Ok(())
}

/// Per-process verified working set trim with query-first handle verification.
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn trim_single_pid(target: &ProcessTarget) -> Result<u64, TrimSkipReason> {
    pre_handle_check(target)?;
    let query = open_raw_checked(PROCESS_QUERY_LIMITED_INFORMATION, target.pid)?;
    verify_live_query_handle(&query, target)?;
    let mut_handle = open_raw_checked(PROCESS_SET_QUOTA, target.pid)?;
    let _keep_query_alive = &query;
    let ws_before =
        unsafe { query_working_set(mut_handle.as_raw()) }.ok_or(TrimSkipReason::Other)?;
    if unsafe { K32EmptyWorkingSet(mut_handle.as_raw()) } == 0 {
        let err = unsafe { GetLastError() };
        return Err(map_win32_error(err));
    }
    let ws_after =
        unsafe { query_working_set(mut_handle.as_raw()) }.ok_or(TrimSkipReason::Other)?;
    Ok(ws_before.saturating_sub(ws_after))
}

#[cfg(not(target_os = "windows"))]
pub fn trim_single_pid(target: &ProcessTarget) -> Result<u64, TrimSkipReason> {
    pre_handle_check(target)?;
    Err(TrimSkipReason::AccessDenied)
}

fn tally_skip(report: &mut ReclaimReport, reason: TrimSkipReason) {
    match reason {
        TrimSkipReason::Protected => report.skipped_protected += 1,
        TrimSkipReason::AccessDenied => report.skipped_access_denied += 1,
        TrimSkipReason::NotFoundOrMismatch => report.skipped_not_found_or_mismatch += 1,
        TrimSkipReason::Other => report.skipped_other += 1,
    }
}

/// Batch working-set trim with structured skip/failure counting.
pub fn trim_processes_working_sets(targets: &[ProcessTarget]) -> ReclaimReport {
    let mut report = ReclaimReport::default();
    for target in targets {
        match trim_single_pid(target) {
            Ok(reclaimed) => {
                report.trimmed_count += 1;
                report.reclaimed_bytes += reclaimed;
            }
            Err(reason) => tally_skip(&mut report, reason),
        }
    }
    report
}

/// Alias-free batch trim for selected/group targets.
pub fn trim_process_group(targets: &[ProcessTarget]) -> ReclaimReport {
    trim_processes_working_sets(targets)
}
