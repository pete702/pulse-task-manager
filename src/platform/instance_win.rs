//! Win32 single-instance mutex acquisition and existing-window wakeup.
use super::{
    basename_of, decide_already_exists_outcome, window_matches, AcquireOutcome,
    SingleInstanceGuard, INSTANCE_MUTEX_HANDLE,
};
use crate::platform::sys_metrics::is_pulse_main_window_title;
use std::sync::atomic::Ordering;

#[link(name = "user32")]
extern "system" {
    fn EnumWindows(cb: unsafe extern "system" fn(isize, isize) -> i32, lparam: isize) -> i32;
    fn GetWindowThreadProcessId(hwnd: isize, lppid: *mut u32) -> u32;
    fn GetWindowTextW(hwnd: isize, lpstring: *mut u16, nmaxcount: i32) -> i32;
    fn AllowSetForegroundWindow(dwprocessid: u32) -> i32;
    fn ShowWindow(hwnd: isize, ncmdshow: i32) -> i32;
    fn SetForegroundWindow(hwnd: isize) -> i32;
    fn IsWindowVisible(hwnd: isize) -> i32;
    fn GetClassNameW(hwnd: isize, lpclass: *mut u16, nmax: i32) -> i32;
    fn PostMessageW(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> i32;
}

/// Wake posted to the existing instance's tray host (PulseTrayHost message window).
const WM_APP_PULSE_WAKE: u32 = 0x0400 + 202;

#[link(name = "kernel32")]
extern "system" {
    fn SetLastError(dwerrcode: u32);
}

#[allow(unsafe_code)]
pub(crate) fn acquire() -> AcquireOutcome {
    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS};
    use windows_sys::Win32::System::Threading::CreateMutexW;
    let name: Vec<u16> = "Local\\PulseTaskManagerSingleInstance\0".encode_utf16().collect();
    unsafe { SetLastError(0) };
    let mutex = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
    if mutex == 0 {
        return AcquireOutcome::Primary(SingleInstanceGuard);
    }
    let err = unsafe { GetLastError() };
    if err != ERROR_ALREADY_EXISTS {
        INSTANCE_MUTEX_HANDLE.store(mutex as isize, Ordering::SeqCst);
        return AcquireOutcome::Primary(SingleInstanceGuard);
    }
    unsafe { CloseHandle(mutex) };
    let woken = wake_existing_pulse_window();
    if decide_already_exists_outcome(woken) {
        AcquireOutcome::AlreadyRunningWoken
    } else {
        AcquireOutcome::Primary(SingleInstanceGuard)
    }
}

#[allow(unsafe_code)]
fn wake_tray_host(hwnds: &[isize], owner_pid: u32, current_pid: u32) -> bool {
    for &hwnd in hwnds {
        let mut pid: u32 = 0;
        if unsafe { GetWindowThreadProcessId(hwnd, &mut pid) } == 0
            || pid == 0
            || pid == current_pid
            || pid != owner_pid
        {
            continue;
        }
        let mut buf = [0u16; 64];
        let n = unsafe { GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
        if n <= 0 {
            continue;
        }
        let class = String::from_utf16_lossy(&buf[..n as usize]);
        if class.eq_ignore_ascii_case("PulseTrayClass") {
            unsafe {
                PostMessageW(hwnd, WM_APP_PULSE_WAKE, 0, 0);
            }
            return true;
        }
    }
    false
}

#[allow(unsafe_code)]
fn wake_existing_pulse_window() -> bool {
    let current_pid = std::process::id();
    let mut hwnds: Vec<isize> = Vec::new();
    unsafe extern "system" fn cb(hwnd: isize, lparam: isize) -> i32 {
        let v = lparam as *mut Vec<isize>;
        if !v.is_null() {
            unsafe { (*v).push(hwnd) };
        }
        1
    }
    unsafe { EnumWindows(cb, &mut hwnds as *mut _ as isize) };
    for &hwnd in &hwnds {
        let mut pid: u32 = 0;
        if unsafe { GetWindowThreadProcessId(hwnd, &mut pid) } == 0
            || pid == 0
            || pid == current_pid
        {
            continue;
        }
        let mut buf = [0u16; 256];
        let n = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
        if n <= 0 {
            continue;
        }
        let title = String::from_utf16_lossy(&buf[..n as usize]);
        if !is_pulse_main_window_title(&title) {
            continue;
        }
        use windows_sys::Win32::Foundation::{CloseHandle as CH, HANDLE};
        use windows_sys::Win32::System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
        };
        let h = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
        let is_pulse = if h != 0 {
            let mut b = vec![0u16; 512];
            let mut sz = b.len() as u32;
            let ok = unsafe { QueryFullProcessImageNameW(h, 0, b.as_mut_ptr(), &mut sz) };
            let r = ok != 0
                && sz != 0
                && basename_of(&String::from_utf16_lossy(&b[..sz as usize]))
                    .eq_ignore_ascii_case("pulse_manager.exe");
            unsafe { CH(h as HANDLE) };
            r
        } else {
            false
        };
        if !is_pulse || !window_matches(current_pid, pid, &title, "pulse_manager.exe") {
            continue;
        }
        use windows_sys::Win32::UI::WindowsAndMessaging::SW_RESTORE;
        let visible = unsafe { IsWindowVisible(hwnd) } != 0;
        let woke_tray = wake_tray_host(&hwnds, pid, current_pid);
        unsafe {
            AllowSetForegroundWindow(pid);
            if !woke_tray || !visible {
                ShowWindow(hwnd, SW_RESTORE);
            }
            SetForegroundWindow(hwnd);
        }
        return true;
    }
    false
}
