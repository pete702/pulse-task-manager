//! System metrics helpers + DWM dark title bar.

#[cfg(target_os = "windows")]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_os = "windows")]
static DARK_TITLEBAR_APPLIED: AtomicBool = AtomicBool::new(false);

/// Pure helper: a non-negative HRESULT means success (S_OK).
#[cfg(any(target_os = "windows", test))]
#[inline]
pub fn hr_succeeded(hr: i32) -> bool {
    hr >= 0
}

/// True only for the real main window title set in `main.rs`.
/// Rejects empty strings and helper titles such as "PulseTrayHost".
pub(crate) fn is_pulse_main_window_title(title: &str) -> bool {
    let t = title.trim();
    if t.is_empty() || t == "PulseTrayHost" {
        return false;
    }
    t == "\u{25B2} PULSE" || t == "PULSE"
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn get_hwnd(_ctx: &eframe::egui::Context) -> isize {
    use std::sync::atomic::{AtomicIsize, Ordering};
    static FOUND: AtomicIsize = AtomicIsize::new(0);
    if FOUND.load(Ordering::Relaxed) != 0 {
        return FOUND.load(Ordering::Relaxed);
    }
    #[link(name = "user32")]
    extern "system" {
        fn EnumWindows(cb: unsafe extern "system" fn(isize, isize) -> i32, lparam: isize) -> i32;
        fn IsWindowVisible(hwnd: isize) -> i32;
        fn GetWindowThreadProcessId(hwnd: isize, pid: *mut u32) -> u32;
        fn GetWindowTextW(hwnd: isize, lpstring: *mut u16, nmaxcount: i32) -> i32;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcessId() -> u32;
    }
    unsafe extern "system" fn cb(hwnd: isize, _lp: isize) -> i32 {
        if IsWindowVisible(hwnd) == 0 {
            return 1;
        }
        let mut pid: u32 = 0;
        let tid = GetWindowThreadProcessId(hwnd, &mut pid);
        if tid == 0 || pid != GetCurrentProcessId() {
            return 1;
        }
        let mut buf = [0u16; 256];
        let n = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
        if n <= 0 {
            return 1;
        }
        let title = String::from_utf16_lossy(&buf[..n as usize]);
        if is_pulse_main_window_title(&title) {
            FOUND.store(hwnd, Ordering::Relaxed);
            return 0;
        }
        1
    }
    unsafe {
        EnumWindows(cb, 0);
    }
    FOUND.load(Ordering::Relaxed)
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn set_dark_title_bar(hwnd: isize) -> bool {
    const DWMWA_USE_IMMERSIVE_DARK_MODE: i32 = 20;
    const DWMWA_USE_IMMERSIVE_DARK_MODE_BEFORE_20H1: i32 = 19;
    const DWMWA_CAPTION_COLOR: i32 = 35;
    #[link(name = "dwmapi")]
    extern "system" {
        fn DwmSetWindowAttribute(
            hwnd: isize,
            dw_attribute: i32,
            pv_attribute: *const i32,
            cb_attribute: u32,
        ) -> i32;
    }
    if hwnd == 0 {
        return false;
    }
    if DARK_TITLEBAR_APPLIED.load(Ordering::Relaxed) {
        return true;
    }
    let dark: i32 = 1;
    let color: i32 = 0x00100C0B_i32;
    unsafe {
        let hr = DwmSetWindowAttribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, &dark, 4);
        if !hr_succeeded(hr) {
            let _hr_fallback =
                DwmSetWindowAttribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE_BEFORE_20H1, &dark, 4);
        }
        // DWMWA_CAPTION_COLOR (35) is supported on Windows 11 (build 22000+);
        // Windows 10 with OS accent colors ignores it (best-effort).
        let _hr_caption = DwmSetWindowAttribute(hwnd, DWMWA_CAPTION_COLOR, &color, 4);
    }
    DARK_TITLEBAR_APPLIED.store(true, Ordering::Relaxed);
    true
}

#[cfg(test)]
mod tests {
    use super::{hr_succeeded, is_pulse_main_window_title};

    #[test]
    fn hr_succeeded_checks_non_negative() {
        assert!(hr_succeeded(0));
        assert!(hr_succeeded(1));
        assert!(!hr_succeeded(-1));
    }

    #[test]
    fn main_window_titles_are_matched_exactly() {
        assert!(is_pulse_main_window_title("\u{25B2} PULSE"));
        assert!(is_pulse_main_window_title("PULSE"));
        assert!(is_pulse_main_window_title("  \u{25B2} PULSE  "));
        assert!(!is_pulse_main_window_title(""));
        assert!(!is_pulse_main_window_title("PulseTrayHost"));
        assert!(!is_pulse_main_window_title("Some IME Helper"));
    }
}
