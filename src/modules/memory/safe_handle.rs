//! Win32 RAII SafeHandle and PID/Process Safety Gate for PULSE.
//! Provides automatic handle cleanup and comprehensive process protection checks.

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::Security::{
    GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcess, OpenProcessToken};

/// RAII wrapper for Win32 HANDLE that guarantees CloseHandle on drop.
pub struct SafeHandle(pub HANDLE);

impl SafeHandle {
    #[allow(unsafe_code)]
    pub fn open(access: u32, pid: u32) -> Option<Self> {
        if pid == 0 || pid == 4 {
            return None;
        }
        let h = unsafe { OpenProcess(access, 0, pid) };
        if h == 0 || h == -1isize as HANDLE {
            None
        } else {
            Some(Self(h))
        }
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
pub fn is_app_elevated() -> bool {
    #[allow(unsafe_code)]
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }
        let mut elevation: TOKEN_ELEVATION = std::mem::zeroed();
        let mut return_length = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut return_length,
        );
        CloseHandle(token);
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

fn normalize_name(name: &str) -> String {
    name.to_lowercase().trim_end_matches(".exe").to_string()
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
