//! Verified suspend/resume actions with identity binding and exit cleanup.
use std::sync::{Mutex, OnceLock};
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
#[link(name = "ntdll")]
extern "system" {
    fn NtSuspendProcess(ProcessHandle: isize) -> i32;
    fn NtResumeProcess(ProcessHandle: isize) -> i32;
}
const SUSPEND_ACCESS: u32 = 0x0800 | 0x1000;
const TICKS: u64 = 10_000_000;
const EPOCH: u64 = 11_644_473_600;
const BLOCKLIST: &str = "dwm.exe explorer.exe csrss.exe audiodg.exe textinputhost.exe svchost.exe winlogon.exe lsass.exe smss.exe services.exe sihost.exe taskmgr.exe";
/// Outcome of a verified mutating action, suitable for the status bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionResult {
    pub ok: bool,
    pub message: String,
}
impl ActionResult {
    pub fn success(m: impl Into<String>) -> Self {
        Self { ok: true, message: m.into() }
    }
    pub fn failure(m: impl Into<String>) -> Self {
        Self { ok: false, message: m.into() }
    }
}
/// One frozen process bound to BOTH pid and its verified raw FILETIME creation time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SuspendedEntry {
    pub(crate) pid: u32,
    pub(crate) creation_time: u64,
}
static SUSPENDED: OnceLock<Mutex<Vec<SuspendedEntry>>> = OnceLock::new();
pub(crate) fn lock() -> std::sync::MutexGuard<'static, Vec<SuspendedEntry>> {
    SUSPENDED.get_or_init(|| Mutex::new(Vec::new())).lock().unwrap_or_else(|e| e.into_inner())
}
#[cfg(test)]
pub(crate) fn reset_registry() {
    lock().clear();
}
/// Pure identity check: a zero creation time can never confirm a binding.
pub(crate) fn matches_suspended_entry(e: &SuspendedEntry, pid: u32, ct: u64) -> bool {
    e.pid == pid && ct != 0 && e.creation_time == ct
}
/// True only for a (pid, creation_time) pair frozen by PULSE; never badges a reused pid.
pub fn is_process_suspended(pid: u32, ct: u64) -> bool {
    let mut reg = lock();
    let found = reg.iter().any(|e| matches_suspended_entry(e, pid, ct));
    if ct != 0 {
        reg.retain(|e| e.pid != pid || e.creation_time == ct);
    }
    found
}
/// Pure FILETIME to Unix-seconds conversion used for the passport check.
pub fn filetime_to_unix_secs(ft: u64) -> u64 {
    (ft / TICKS).saturating_sub(EPOCH)
}
/// Hard blocklist: critical desktop/audio/shell/OS hosts, protected paths and self.
pub fn is_suspend_blocklisted(name: &str, path: &str, pid: u32, self_pid: u32) -> bool {
    if pid <= 4 || pid == self_pid {
        return true;
    }
    let base = name.rsplit(['\\', '/']).next().unwrap_or(name).trim();
    if BLOCKLIST.split_whitespace().any(|b| base.eq_ignore_ascii_case(b)) {
        return true;
    }
    crate::platform::handle::verify_pid_safety_gate_ex(pid, base, Some(path)).is_err()
}
/// Passport check returning the live raw FILETIME; used as the registry identity.
#[cfg(target_os = "windows")]
fn passport(
    h: &crate::platform::handle::SafeHandle,
    start: u64,
    path: &str,
) -> Result<u64, &'static str> {
    use crate::platform::handle::{
        is_handle_critical, query_process_creation_time, query_process_image_path,
    };
    use crate::platform::protected::full_paths_match;
    let ft = query_process_creation_time(h).ok_or("creation time query failed")?;
    if start != 0 && filetime_to_unix_secs(ft).abs_diff(start) > 2 {
        return Err("start time mismatch");
    }
    let live = query_process_image_path(h).ok_or("image path query failed")?;
    if !path.trim().is_empty() && !full_paths_match(path, &live) {
        return Err("image path mismatch");
    }
    if is_handle_critical(h) {
        return Err("critical process");
    }
    Ok(ft)
}
#[cfg(target_os = "windows")]
type Verified = (crate::platform::handle::SafeHandle, u64);
#[cfg(target_os = "windows")]
fn open_verified(pid: u32, start: u64, path: &str, action: &str) -> Result<Verified, ActionResult> {
    use crate::platform::handle::SafeHandle;
    let h = SafeHandle::open(SUSPEND_ACCESS, pid)
        .ok_or_else(|| ActionResult::failure(format!("{action} blocked: cannot open process")))?;
    let ct = passport(&h, start, path)
        .map_err(|r| ActionResult::failure(format!("{action} blocked: {r}")))?;
    Ok((h, ct))
}
/// Suspends a process only after the identity passport and blocklist pass.
pub fn suspend_process_verified(pid: u32, start: u64, path: &str, name: &str) -> ActionResult {
    if is_suspend_blocklisted(name, path, pid, std::process::id()) {
        return ActionResult::failure("Suspend blocked: protected or critical process");
    }
    #[cfg(target_os = "windows")]
    {
        let (h, ct) = match open_verified(pid, start, path, "Suspend") {
            Ok(v) => v,
            Err(e) => return e,
        };
        #[allow(unsafe_code)]
        let status = unsafe { NtSuspendProcess(h.as_raw()) };
        if status != 0 {
            return ActionResult::failure(format!("Suspend failed (NT status 0x{status:x})"));
        }
        lock().push(SuspendedEntry { pid, creation_time: ct });
        ActionResult::success(format!("Suspended {name} (PID {pid})"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (pid, start, path, name);
        ActionResult::failure("Suspend unavailable on this platform")
    }
}
/// Resumes only a process previously suspended by PULSE, with identity re-check.
pub fn resume_process_verified(pid: u32, start: u64, path: &str, name: &str) -> ActionResult {
    #[cfg(target_os = "windows")]
    {
        let (h, ct) = match open_verified(pid, start, path, "Resume") {
            Ok(v) => v,
            Err(e) => return e,
        };
        if !is_process_suspended(pid, ct) {
            return ActionResult::failure("Resume blocked: not suspended by PULSE");
        }
        #[allow(unsafe_code)]
        let status = unsafe { NtResumeProcess(h.as_raw()) };
        if status != 0 {
            return ActionResult::failure(format!("Resume failed (NT status 0x{status:x})"));
        }
        lock().retain(|e| !(e.pid == pid && e.creation_time == ct));
        ActionResult::success(format!("Resumed {name} (PID {pid})"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (pid, start, path, name);
        ActionResult::failure("Resume unavailable on this platform")
    }
}
/// NT_SUCCESS is `status >= 0`; an already exited process needs no resume call.
pub(crate) fn should_remove_entry(st: Option<i32>, exited: bool) -> bool {
    exited || st.is_some_and(|s| s >= 0)
}
/// Pure per-entry decision: only confirmed exit or PID reuse drops a record.
pub(crate) fn resume_disposition(
    e: &SuspendedEntry,
    live: Option<u64>,
    st: Option<i32>,
    exited: bool,
) -> bool {
    if exited {
        return true;
    }
    match live {
        Some(ct) if matches_suspended_entry(e, e.pid, ct) => should_remove_entry(st, false),
        Some(_) => true,
        None => false,
    }
}
/// Probe: entry, live creation time, NtResumeProcess status, confirmed-exit flag.
pub(crate) type Probe = (SuspendedEntry, Option<u64>, Option<i32>, bool);
/// Applies probe outcomes to the registry; returns `(resumed, failed)` counts.
pub(crate) fn apply_results(r: &[Probe]) -> (usize, usize) {
    let removed: Vec<_> =
        r.iter().filter(|(e, live, st, ex)| resume_disposition(e, *live, *st, *ex)).collect();
    let resumed = removed.iter().filter(|(_, _, st, _)| st.is_some_and(|s| s >= 0)).count();
    lock().retain(|e| {
        !removed.iter().any(|(x, _, _, _)| x.pid == e.pid && x.creation_time == e.creation_time)
    });
    (resumed, r.len() - removed.len())
}
/// Best-effort resume of every frozen process; failed NtResumeProcess calls keep their entries.
#[allow(unsafe_code)]
pub fn resume_all_suspended() -> (usize, usize) {
    let entries: Vec<SuspendedEntry> = lock().clone();
    let mut results: Vec<Probe> = Vec::new();
    #[cfg(target_os = "windows")]
    {
        use crate::platform::handle::{query_process_creation_time, SafeHandle};
        use windows_sys::Win32::Foundation::{GetLastError, ERROR_INVALID_PARAMETER};
        for e in &entries {
            let Some(h) = SafeHandle::open(SUSPEND_ACCESS, e.pid) else {
                let exited = unsafe { GetLastError() } == ERROR_INVALID_PARAMETER;
                results.push((e.clone(), None, None, exited));
                continue;
            };
            let Some(live) = query_process_creation_time(&h) else {
                let exited = unsafe { GetLastError() } == ERROR_INVALID_PARAMETER;
                results.push((e.clone(), None, None, exited));
                continue;
            };
            let st = if live == e.creation_time {
                #[allow(unsafe_code)]
                Some(unsafe { NtResumeProcess(h.as_raw()) })
            } else {
                None
            };
            results.push((e.clone(), Some(live), st, false));
        }
    }
    #[cfg(not(target_os = "windows"))]
    for e in &entries {
        results.push((e.clone(), None, None, false));
    }
    apply_results(&results)
}
/// Held for the application lifetime; resumes every frozen process on drop.
#[derive(Default)]
pub struct SuspendCleanupGuard;
impl Drop for SuspendCleanupGuard {
    fn drop(&mut self) {
        resume_all_suspended();
    }
}
