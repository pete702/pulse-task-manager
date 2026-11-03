//! Win32 RAII SafeHandle and PID/Process Safety Gate for PULSE.
//! Provides automatic handle cleanup and comprehensive process protection checks.

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::Security::{
    GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcess, OpenProcessToken};

/// RAII wrapper for Win32 HANDLE that guarantees CloseHandle on drop.
/// The inner HANDLE is private; use `from_raw` / `as_raw` for controlled access.
pub struct SafeHandle(HANDLE);

impl SafeHandle {
    /// Creates a SafeHandle from a raw HANDLE. Returns None if the handle is null/invalid.
    #[inline]
    pub fn from_raw(h: HANDLE) -> Option<Self> {
        if h == 0 || h == -1isize as HANDLE {
            None
        } else {
            Some(Self(h))
        }
    }

    /// Returns the raw HANDLE for FFI calls.
    #[inline]
    pub fn as_raw(&self) -> HANDLE {
        self.0
    }

    #[allow(unsafe_code)]
    pub fn open(access: u32, pid: u32) -> Option<Self> {
        if pid == 0 || pid == 4 {
            return None;
        }
        let h = unsafe { OpenProcess(access, 0, pid) };
        Self::from_raw(h)
    }
}

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

/// Checks if the current process is running elevated (as Administrator).
/// Uses SafeHandle RAII for the token to guarantee CloseHandle on all paths.
pub fn is_app_elevated() -> bool {
    #[allow(unsafe_code)]
    unsafe {
        let mut raw_token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw_token) == 0 {
            return false;
        }
        // Wrap token in SafeHandle so CloseHandle is guaranteed on Drop
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
}

const CRITICAL_OS_PROCESSES: &[&str] = &[
    "system",
    "registry",
    "memory compression",
    "smss.exe",
    "csrss.exe",
    "wininit.exe",
    "services.exe",
    "lsass.exe",
    "lsaiso.exe",
    "svchost.exe",
    "dwm.exe",
    "winlogon.exe",
    "fontdrvhost.exe",
    "sihost.exe",
    "taskhostw.exe",
    "msmpeng.exe",
    "nissrv.exe",
    "securityhealthservice.exe",
    "spoolsv.exe",
    "audiodg.exe",
    "wmiprvse.exe",
    "searchindexer.exe",
    "startmenuexperiencehost.exe",
    "searchhost.exe",
    "shellexperiencehost.exe",
    "textinputhost.exe",
    "conhost.exe",
];
const ANTI_CHEAT_PROCESSES: &[&str] = &[
    "vgc.exe",
    "vgtray.exe",
    "easyanticheat.exe",
    "easyanticheat_eos.exe",
    "beservice.exe",
    "bedaisy.sys",
    "zksvc.exe",
    "faceitclient.exe",
    "eseaclient.exe",
    "ricochet.exe",
    "hoyokprotect.sys",
];

/// Normalizes a process name for comparison: extracts filename, lowercases, strips .exe.
/// Prevents path bypass (e.g., "C:\\Windows\\System32\\csrss.exe" -> "csrss").
fn normalize_name(name: &str) -> String {
    let base = name
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(name)
        .trim()
        .to_ascii_lowercase();
    base.strip_suffix(".exe").unwrap_or(&base).to_string()
}

/// Verifies if a PID/process_name pair is safe to target.
pub fn verify_pid_safety_gate(pid: u32, process_name: &str) -> Result<(), &'static str> {
    if pid == 0 || pid == 4 {
        return Err("OS kernel process is protected");
    }
    if pid == std::process::id() {
        return Err("Cannot target PULSE itself");
    }
    let norm = normalize_name(process_name);
    if CRITICAL_OS_PROCESSES
        .iter()
        .any(|&p| normalize_name(p) == norm)
    {
        return Err("Critical Windows OS process is protected");
    }
    if ANTI_CHEAT_PROCESSES
        .iter()
        .any(|&p| normalize_name(p) == norm)
    {
        return Err("Anti-cheat protected process skipped");
    }
    Ok(())
}

/// Returns true if the process is protected (cannot be targeted).
pub fn is_protected_process(pid: u32, process_name: &str) -> bool {
    verify_pid_safety_gate(pid, process_name).is_err()
}

/// Checks if a process handle represents a kernel-critical process.
/// Uses IsProcessCritical (Windows 8.1+) to detect processes the OS considers critical.
#[allow(unsafe_code)]
pub fn is_handle_critical(handle: &SafeHandle) -> bool {
    let mut critical: i32 = 0;
    // SAFETY: Queries kernel critical flag on a valid open process handle.
    let ok = unsafe {
        windows_sys::Win32::System::Threading::IsProcessCritical(handle.as_raw(), &mut critical)
    };
    ok != 0 && critical != 0
}
