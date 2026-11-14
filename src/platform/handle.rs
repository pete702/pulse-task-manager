//! Win32 RAII SafeHandle, process identity queries, and PID safety gate for PULSE.
#[cfg(target_os = "windows")]
use super::protected::full_paths_match;
use super::protected::{
    is_any_protected_name, is_kernel_pseudo, is_security_defender_anticheat,
    is_under_security_trusted_root, is_under_system_root, is_windows_os_binary,
    normalize_process_name, protection_reason,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{CloseHandle, FILETIME, HANDLE};
#[cfg(target_os = "windows")]
use windows_sys::Win32::Security::{
    GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetProcessId, GetProcessTimes, OpenProcess, OpenProcessToken,
    QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
};
#[cfg(target_os = "windows")]
pub struct SafeHandle(HANDLE);
#[cfg(target_os = "windows")]
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
    #[allow(unsafe_code)]
    pub fn open(access: u32, pid: u32) -> Option<Self> {
        if pid == 0 || pid == 4 {
            return None;
        }
        Self::from_raw(unsafe { OpenProcess(access, 0, pid) })
    }
    pub fn open_process(pid: u32) -> Option<Self> {
        Self::open(PROCESS_QUERY_LIMITED_INFORMATION, pid)
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

static ELEVATED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn is_app_elevated() -> bool {
    *ELEVATED.get_or_init(|| unsafe {
        let mut raw_token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw_token) == 0 {
            return false;
        }
        let _guard = match SafeHandle::from_raw(raw_token) {
            Some(h) => h,
            None => return false,
        };
        let mut elevation: TOKEN_ELEVATION = std::mem::zeroed();
        let mut ret = 0u32;
        let ok = GetTokenInformation(
            raw_token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut ret,
        );
        ok != 0 && elevation.TokenIsElevated != 0
    })
}
#[cfg(not(target_os = "windows"))]
pub fn is_app_elevated() -> bool {
    *ELEVATED.get_or_init(|| false)
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn query_process_image_path(handle: &SafeHandle) -> Option<String> {
    let mut buf = vec![0u16; 4096];
    let mut size = buf.len() as u32;
    let ok = unsafe { QueryFullProcessImageNameW(handle.as_raw(), 0, buf.as_mut_ptr(), &mut size) };
    if ok == 0 || size == 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&buf[..size as usize]))
}

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

#[cfg(target_os = "windows")]
pub fn query_creation_time_by_pid(pid: u32) -> Option<u64> {
    query_process_creation_time(&SafeHandle::open_process(pid)?)
}
#[cfg(not(target_os = "windows"))]
pub fn query_creation_time_by_pid(_pid: u32) -> Option<u64> {
    None
}

/// Path-aware safety gate. PID 0/4/self are always protected.
/// When `exe_path` is missing/empty, any protected-list name is treated as protected (fail-closed).
/// When `exe_path` is present, OS/kernel names are protected only under SystemRoot and
/// security/defender/anti-cheat names only under trusted system/defender roots.
pub fn verify_pid_safety_gate_ex(
    pid: u32,
    process_name: &str,
    exe_path: Option<&str>,
) -> Result<(), &'static str> {
    if pid == 0 || pid == 4 {
        return Err("OS kernel process is protected");
    }
    if pid == std::process::id() {
        return Err("Cannot target PULSE itself");
    }
    let norm = normalize_process_name(process_name);
    if !exe_path.is_some_and(|p| !p.trim().is_empty()) {
        return if is_any_protected_name(&norm) { Err(protection_reason(&norm)) } else { Ok(()) };
    }
    let path = exe_path.unwrap_or_default();
    if is_kernel_pseudo(&norm) || is_windows_os_binary(&norm) {
        return if is_under_system_root(path) { Err(protection_reason(&norm)) } else { Ok(()) };
    }
    if is_security_defender_anticheat(&norm) {
        return if is_under_security_trusted_root(path) {
            Err(protection_reason(&norm))
        } else {
            Ok(())
        };
    }
    Ok(())
}

pub fn verify_pid_safety_gate(pid: u32, process_name: &str) -> Result<(), &'static str> {
    verify_pid_safety_gate_ex(pid, process_name, None)
}

pub fn is_protected_process(pid: u32, process_name: &str) -> bool {
    verify_pid_safety_gate(pid, process_name).is_err()
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn is_handle_critical(handle: &SafeHandle) -> bool {
    let mut critical: i32 = 0;
    let ok = unsafe {
        windows_sys::Win32::System::Threading::IsProcessCritical(handle.as_raw(), &mut critical)
    };
    ok == 0 || critical != 0
}

/// Fail-closed live process identity check.
/// Always verifies non-zero `expected_creation_time` equality first. Queries the live full
/// executable image path and runs `verify_pid_safety_gate_ex` against live handle data.
/// When `expected_path` is Some(non-empty), compares the FULL normalized path case-insensitively
/// against the live handle path (fails closed if missing or different directory).
/// When `expected_path` is None, enforces creation_time, expected basename match, and live
/// path-aware safety gate only.
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn verify_handle_identity(
    handle: &SafeHandle,
    expected_name: &str,
    expected_creation_time: Option<u64>,
    expected_path: Option<&str>,
) -> bool {
    let Some(exp_time) = expected_creation_time.filter(|&t| t != 0) else {
        return false;
    };
    if query_process_creation_time(handle) != Some(exp_time) {
        return false;
    }
    let Some(actual_path) = query_process_image_path(handle) else {
        return false;
    };
    let pid = unsafe { GetProcessId(handle.as_raw()) };
    if pid == 0 {
        return false;
    }
    let live_name_norm = normalize_process_name(&actual_path);
    if verify_pid_safety_gate_ex(pid, &live_name_norm, Some(&actual_path)).is_err() {
        return false;
    }
    if let Some(exp_path) = expected_path.filter(|p| !p.trim().is_empty()) {
        return full_paths_match(exp_path, &actual_path);
    }
    live_name_norm == normalize_process_name(expected_name)
}
