//! Termination actions for PULSE (trim helpers live in platform/trim.rs).

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::GetLastError;
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::{
    TerminateProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
};

use crate::platform::target::ProcessTarget;
#[cfg(target_os = "windows")]
use crate::platform::trim::{map_win32_error, open_raw_checked, verify_live_query_handle};
use crate::platform::trim::{pre_handle_check, TrimSkipReason};

/// Structured batch termination result.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TerminateReport {
    pub succeeded: usize,
    pub total: usize,
    pub skipped_protected: usize,
    pub skipped_access_denied: usize,
    pub skipped_not_found_or_mismatch: usize,
    pub skipped_other: usize,
}

impl TerminateReport {
    pub fn skipped_total(&self) -> usize {
        self.skipped_protected
            + self.skipped_access_denied
            + self.skipped_not_found_or_mismatch
            + self.skipped_other
    }
    fn tally(&mut self, reason: TrimSkipReason) {
        match reason {
            TrimSkipReason::Protected => self.skipped_protected += 1,
            TrimSkipReason::AccessDenied => self.skipped_access_denied += 1,
            #[cfg(target_os = "windows")]
            TrimSkipReason::NotFoundOrMismatch => self.skipped_not_found_or_mismatch += 1,
            #[cfg(target_os = "windows")]
            TrimSkipReason::Other => self.skipped_other += 1,
        }
    }
    fn primary_reason(&self) -> &'static str {
        if self.skipped_protected > 0 {
            "protected"
        } else if self.skipped_access_denied > 0 {
            "access denied"
        } else if self.skipped_not_found_or_mismatch > 0 {
            "exited/identity mismatch"
        } else {
            "other"
        }
    }
    pub fn format_status(&self) -> String {
        if self.succeeded == 0 {
            format!("Terminated 0/{} ({})", self.total, self.primary_reason())
        } else if self.skipped_total() > 0 {
            format!(
                "Terminated {}/{} ({} {})",
                self.succeeded,
                self.total,
                self.skipped_total(),
                self.primary_reason()
            )
        } else {
            format!("Terminated {}/{}", self.succeeded, self.total)
        }
    }
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
fn terminate_single_target(target: &ProcessTarget) -> Result<(), TrimSkipReason> {
    pre_handle_check(target)?;
    let query = open_raw_checked(PROCESS_QUERY_LIMITED_INFORMATION, target.pid)?;
    verify_live_query_handle(&query, target)?;
    let mut_handle = open_raw_checked(PROCESS_TERMINATE, target.pid)?;
    let _keep_query_alive = &query;
    if unsafe { TerminateProcess(mut_handle.as_raw(), 1) } == 0 {
        let err = unsafe { GetLastError() };
        return Err(map_win32_error(err));
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn terminate_single_target(target: &ProcessTarget) -> Result<(), TrimSkipReason> {
    pre_handle_check(target)?;
    Err(TrimSkipReason::AccessDenied)
}

/// Terminates verified targets with structured reporting. Never aborts remaining targets.
pub fn terminate_process_group(targets: &[ProcessTarget]) -> TerminateReport {
    let mut report = TerminateReport { total: targets.len(), ..Default::default() };
    for target in targets {
        match terminate_single_target(target) {
            Ok(()) => report.succeeded += 1,
            Err(reason) => report.tally(reason),
        }
    }
    report
}

pub use crate::platform::location::open_process_file_location;
