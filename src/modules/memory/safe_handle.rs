//! Win32 RAII SafeHandle, process identity queries, and PID safety gate for PULSE.
use super::protected_lists::{ANTI_CHEAT_PROCESSES, CRITICAL_OS_PROCESSES};
#[cfg(target_os = "windows")]
use std::sync::OnceLock;
#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{CloseHandle, FILETIME, HANDLE};
#[cfg(target_os = "windows")]
use windows_sys::Win32::Security::{
    GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetProcessTimes, OpenProcess, OpenProcessToken, QueryFullProcessImageNameW,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
#[cfg(not(target_os = "windows"))]
pub type HANDLE = isize;

pub struct SafeHandle(HANDLE);
impl SafeHandle {
    #[inline]
    pub(crate) fn from_raw(h: HANDLE) -> Option<Self> {
        if h == 0 || h == -1isize as HANDLE {
            None
        } else {
            Some(Self(h))
        }
    }
    #[inline]
    pub fn as_raw(&self) -> HANDLE {
        self.0
    }
    #[cfg(target_os = "windows")]
    #[allow(unsafe_code)]
    pub fn open(access: u32, pid: u32) -> Option<Self> {
        if pid == 0 || pid == 4 {
            return None;
        }
        Self::from_raw(unsafe { OpenProcess(access, 0, pid) })
    }
    #[cfg(not(target_os = "windows"))]
    pub fn open(_access: u32, _pid: u32) -> Option<Self> {
        None
    }
    /// Opens a query-only handle for identity inspection.
    #[cfg(target_os = "windows")]
    pub fn open_process(pid: u32) -> Option<Self> {
        Self::open(PROCESS_QUERY_LIMITED_INFORMATION, pid)
    }
    #[cfg(not(target_os = "windows"))]
    pub fn open_process(_pid: u32) -> Option<Self> {
        None
    }
}
#[cfg(target_os = "windows")]
impl Drop for SafeHandle {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        if self.0 != 0 && self.0 != -1isize as HANDLE {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}
#[cfg(not(target_os = "windows"))]
impl Drop for SafeHandle {
    fn drop(&mut self) {}
}

#[cfg(target_os = "windows")]
pub fn is_app_elevated() -> bool {
    static CACHE: OnceLock<bool> = OnceLock::new();
    *CACHE.get_or_init(|| {
        #[allow(unsafe_code)]
        unsafe {
            let mut raw_token = HANDLE::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw_token) == 0 {
                return false;
            }
            let _token_guard = match SafeHandle::from_raw(raw_token) {
                Some(h) => h,
                None => return false,
            };
            let mut elevation: TOKEN_ELEVATION = std::mem::zeroed();
            let mut return_length = 0u32;
            let ok = GetTokenInformation(
                raw_token,
                TokenElevation,
                &mut elevation as *mut _ as *mut _,
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut return_length,
            );
            ok != 0 && elevation.TokenIsElevated != 0
        }
    })
}
#[cfg(not(target_os = "windows"))]
pub fn is_app_elevated() -> bool {
    false
}

fn normalize_name(name: &str) -> String {
    let base = name.rsplit(['\\', '/']).next().unwrap_or(name).trim().to_ascii_lowercase();
    base.strip_suffix(".exe").unwrap_or(&base).to_string()
}

/// Returns the full executable path for a process handle, or `None` on failure.
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn query_process_image_path(handle: &SafeHandle) -> Option<String> {
    // Heap-allocated to eliminate stack pressure on worker threads.
    let mut buf = vec![0u16; 4096];
    let mut size = buf.len() as u32;
    let ok = unsafe { QueryFullProcessImageNameW(handle.as_raw(), 0, buf.as_mut_ptr(), &mut size) };
    if ok == 0 || size == 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&buf[..size as usize]))
}
#[cfg(not(target_os = "windows"))]
pub fn query_process_image_path(_handle: &SafeHandle) -> Option<String> {
    None
}

/// Returns the process creation time as a 64-bit FILETIME value.
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn query_process_creation_time(handle: &SafeHandle) -> Option<u64> {
    let mut create = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let (mut exit, mut kernel, mut user) = (create, create, create);
    let ok =
        unsafe { GetProcessTimes(handle.as_raw(), &mut create, &mut exit, &mut kernel, &mut user) };
    if ok == 0 {
        return None;
    }
    let value = ((create.dwHighDateTime as u64) << 32) | create.dwLowDateTime as u64;
    (value != 0).then_some(value)
}
#[cfg(not(target_os = "windows"))]
pub fn query_process_creation_time(_handle: &SafeHandle) -> Option<u64> {
    None
}

/// Opens a query-limited handle and returns its creation time.
#[cfg(target_os = "windows")]
pub fn query_creation_time_by_pid(pid: u32) -> Option<u64> {
    query_process_creation_time(&SafeHandle::open_process(pid)?)
}
#[cfg(not(target_os = "windows"))]
pub fn query_creation_time_by_pid(_pid: u32) -> Option<u64> {
    None
}

pub fn verify_pid_safety_gate(pid: u32, process_name: &str) -> Result<(), &'static str> {
    if pid == 0 || pid == 4 {
        return Err("OS kernel process is protected");
    }
    if pid == std::process::id() {
        return Err("Cannot target PULSE itself");
    }
    let norm = normalize_name(process_name);
    if CRITICAL_OS_PROCESSES.iter().any(|&p| normalize_name(p) == norm) {
        return Err("Critical Windows OS process is protected");
    }
    if ANTI_CHEAT_PROCESSES.iter().any(|&p| normalize_name(p) == norm) {
        return Err("Anti-cheat protected process skipped");
    }
    Ok(())
}

pub fn is_protected_process(pid: u32, process_name: &str) -> bool {
    verify_pid_safety_gate(pid, process_name).is_err()
}

/// Fail-closed critical check: a failed Win32 query blocks the mutation.
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn is_handle_critical(handle: &SafeHandle) -> bool {
    let mut critical: i32 = 0;
    let ok = unsafe {
        windows_sys::Win32::System::Threading::IsProcessCritical(handle.as_raw(), &mut critical)
    };
    ok == 0 || critical != 0
}
#[cfg(not(target_os = "windows"))]
pub fn is_handle_critical(_handle: &SafeHandle) -> bool {
    true
}

/// Fail-closed identity check: name, creation time, and executable path must match.
/// Missing or zero creation_time fails closed -- no bypass for instance verification.
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn verify_handle_identity(
    handle: &SafeHandle,
    expected_name: &str,
    expected_creation_time: Option<u64>,
    expected_path: Option<&str>,
) -> bool {
    let Some(actual_path) = query_process_image_path(handle) else {
        return false;
    };
    if normalize_name(&actual_path) != normalize_name(expected_name) {
        return false;
    }
    // Fail-closed: require a valid non-zero creation_time to pass verification.
    let Some(exp_time) = expected_creation_time.filter(|&t| t != 0) else {
        return false;
    };
    if query_process_creation_time(handle) != Some(exp_time) {
        return false;
    }
    if let Some(exp_path) = expected_path {
        if !exp_path.trim().is_empty() && !actual_path.trim().eq_ignore_ascii_case(exp_path.trim())
        {
            return false;
        }
    }
    true
}
#[cfg(not(target_os = "windows"))]
pub fn verify_handle_identity(
    _handle: &SafeHandle,
    _expected_name: &str,
    _expected_creation_time: Option<u64>,
    _expected_path: Option<&str>,
) -> bool {
    false
}
