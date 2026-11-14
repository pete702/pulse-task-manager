//! System metrics helpers + DWM dark title bar.

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
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == GetCurrentProcessId() {
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

#[cfg(not(target_os = "windows"))]
pub fn get_hwnd(_ctx: &eframe::egui::Context) -> isize {
    0
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn set_dark_title_bar(hwnd: isize) -> bool {
    const DWMWA_USE_IMMERSIVE_DARK_MODE: i32 = 20;
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
    let dark: i32 = 1;
    let color: i32 = 0x00100C0B_i32;
    unsafe {
        let _ = DwmSetWindowAttribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, &dark, 4);
        DwmSetWindowAttribute(hwnd, DWMWA_CAPTION_COLOR, &color, 4) == 0
    }
}

#[cfg(not(target_os = "windows"))]
pub fn set_dark_title_bar(_hwnd: isize) -> bool {
    false
}
