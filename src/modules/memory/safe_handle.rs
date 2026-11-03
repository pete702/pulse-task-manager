//! Win32 RAII SafeHandle and PID/Process Safety Gate for PULSE.
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::Security::{
    GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, OpenProcess, OpenProcessToken, QueryFullProcessImageNameW,
};

pub struct SafeHandle(HANDLE);
impl SafeHandle {
    #[inline]
    pub fn from_raw(h: HANDLE) -> Option<Self> {
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

pub fn is_app_elevated() -> bool {
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

fn normalize_name(name: &str) -> String {
    let base = name.rsplit(['\\', '/']).next().unwrap_or(name).trim().to_ascii_lowercase();
    base.strip_suffix(".exe").unwrap_or(&base).to_string()
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

#[allow(unsafe_code)]
pub fn is_handle_critical(handle: &SafeHandle) -> bool {
    let mut critical: i32 = 0;
    let ok = unsafe {
        windows_sys::Win32::System::Threading::IsProcessCritical(handle.as_raw(), &mut critical)
    };
    ok != 0 && critical != 0
}

#[allow(unsafe_code)]
pub fn verify_handle_identity(handle: &SafeHandle, expected_name: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        let mut buf = [0u16; 1024];
        let mut size = buf.len() as u32;
        let ok =
            unsafe { QueryFullProcessImageNameW(handle.as_raw(), 0, buf.as_mut_ptr(), &mut size) };
        if ok == 0 || size == 0 {
            return false;
        }
        let actual_path = String::from_utf16_lossy(&buf[..size as usize]);
        normalize_name(&actual_path) == normalize_name(expected_name)
    }
    #[cfg(not(target_os = "windows"))]
    {
        !expected_name.trim().is_empty()
    }
}
