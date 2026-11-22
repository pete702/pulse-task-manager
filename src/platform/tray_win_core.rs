use super::tray_win_thread::{find_main_hwnd, restore_window_now, tray_thread};
use super::TrayAction;
use egui::Context;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::{mpsc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

#[repr(C)]
pub struct NotifyIconDataW {
    pub(super) cb_size: u32,
    pub(super) hwnd: isize,
    pub(super) uid: u32,
    pub(super) u_flags: u32,
    pub(super) callback_message: u32,
    pub(super) h_icon: isize,
    pub(super) sz_tip: [u16; 128],
    pub(super) dw_state: u32,
    pub(super) dw_state_mask: u32,
    pub(super) sz_info: [u16; 256],
    pub(super) u_version: u32,
    pub(super) sz_info_title: [u16; 64],
    pub(super) dw_info_flags: u32,
    pub(super) guid_item: [u8; 16],
    pub(super) h_balloon_icon: isize,
}
#[repr(C)]
pub(super) struct WndClassW {
    pub(super) style: u32,
    pub(super) lpfn: unsafe extern "system" fn(isize, u32, usize, isize) -> isize,
    pub(super) cb_cls: i32,
    pub(super) cb_wnd: i32,
    pub(super) hinst: isize,
    pub(super) hicon: isize,
    pub(super) hcursor: isize,
    pub(super) hbr: isize,
    pub(super) menu: *const u16,
    pub(super) class: *const u16,
}
#[repr(C)]
pub(super) struct Point {
    pub(super) x: i32,
    pub(super) y: i32,
}
#[repr(C)]
pub(super) struct Msg {
    hwnd: isize,
    message: u32,
    wparam: usize,
    lparam: isize,
    time: u32,
    pt: Point,
}
pub struct TrayGuard {
    handle: Option<thread::JoinHandle<()>>,
    hwnd: isize,
    _cleanup: crate::platform::suspend::SuspendCleanupGuard,
}

pub(super) static TRAY_HWND: AtomicIsize = AtomicIsize::new(0);
pub(super) static MAIN_HWND: AtomicIsize = AtomicIsize::new(0);
pub(super) static TRAY_CTX: OnceLock<Context> = OnceLock::new();
pub(super) static TRAY_TX: OnceLock<Mutex<Option<mpsc::Sender<TrayAction>>>> = OnceLock::new();
static ALLOW_REAL_CLOSE: AtomicBool = AtomicBool::new(false);
pub(super) static TRAY_ICON_REMOVED: AtomicBool = AtomicBool::new(false);
pub(super) const SHUTDOWN_WAIT_MS: u64 = 250;

pub(super) fn hide_target(cached: isize, stored: isize, looked_up: isize) -> isize {
    [cached, stored, looked_up].into_iter().find(|&v| v != 0).unwrap_or(0)
}

#[link(name = "kernel32")]
extern "system" {
    pub(super) fn GetModuleHandleW(name: *const u16) -> isize;
}
#[link(name = "user32")]
extern "system" {
    pub(super) fn RegisterClassW(wc: *const WndClassW) -> u16;
    pub(super) fn CreateWindowExW(
        ex: u32,
        class: *const u16,
        title: *const u16,
        style: u32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        parent: isize,
        menu: isize,
        inst: isize,
        param: *mut std::ffi::c_void,
    ) -> isize;
    pub(super) fn DefWindowProcW(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize;
    pub(super) fn GetMessageW(msg: *mut Msg, hwnd: isize, min: u32, max: u32) -> i32;
    pub(super) fn TranslateMessage(msg: *const Msg) -> i32;
    pub(super) fn DispatchMessageW(msg: *const Msg) -> isize;
    pub(super) fn PostMessageW(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> i32;
    pub(super) fn PostQuitMessage(code: i32);
    pub(super) fn GetCursorPos(pt: *mut Point) -> i32;
    pub(super) fn SetForegroundWindow(hwnd: isize) -> i32;
    pub(super) fn LoadIconW(inst: isize, name: *const u16) -> isize;
    pub(super) fn CreatePopupMenu() -> isize;
    pub(super) fn AppendMenuW(menu: isize, flags: u32, id: usize, text: *const u16) -> i32;
    pub(super) fn TrackPopupMenu(
        menu: isize,
        flags: u32,
        x: i32,
        y: i32,
        reserved: i32,
        hwnd: isize,
        rect: *const std::ffi::c_void,
    ) -> i32;
    pub(super) fn DestroyMenu(menu: isize) -> i32;
    fn ShowWindow(hwnd: isize, ncmdshow: i32) -> i32;
}
#[link(name = "shell32")]
extern "system" {
    pub(super) fn Shell_NotifyIconW(dwmessage: u32, lpdata: *const NotifyIconDataW) -> i32;
}

pub fn allow_real_close() -> bool {
    ALLOW_REAL_CLOSE.load(Ordering::SeqCst)
}
pub fn set_allow_real_close(v: bool) {
    ALLOW_REAL_CLOSE.store(v, Ordering::SeqCst)
}
pub fn encode_tray_tip(text: &str) -> [u16; 128] {
    let mut out = [0u16; 128];
    let mut i = 0usize;
    for c in text.encode_utf16() {
        if i >= 127 {
            break;
        }
        out[i] = c;
        i += 1;
    }
    out[i] = 0;
    out
}
#[allow(unsafe_code)]
pub fn hide_main_window(hwnd: isize) -> bool {
    let target = hide_target(hwnd, MAIN_HWND.load(Ordering::SeqCst), find_main_hwnd());
    if target == 0 {
        return false;
    }
    unsafe {
        ShowWindow(target, windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE);
    }
    MAIN_HWND.store(target, Ordering::SeqCst);
    true
}
pub fn restore_main_window(hwnd: isize) {
    restore_window_now(hwnd, false);
}
#[allow(unsafe_code)]
pub fn update_tray_tooltip(text: &str) {
    let hwnd = TRAY_HWND.load(Ordering::SeqCst);
    if hwnd == 0 {
        return;
    }
    let nid = NotifyIconDataW {
        cb_size: std::mem::size_of::<NotifyIconDataW>() as u32,
        hwnd,
        uid: 1,
        u_flags: 4,
        callback_message: 0,
        h_icon: 0,
        sz_tip: encode_tray_tip(text),
        dw_state: 0,
        dw_state_mask: 0,
        sz_info: [0; 256],
        u_version: 0,
        sz_info_title: [0; 64],
        dw_info_flags: 0,
        guid_item: [0; 16],
        h_balloon_icon: 0,
    };
    const NIM_MODIFY: u32 = 1;
    unsafe {
        Shell_NotifyIconW(NIM_MODIFY, &nid);
    }
}
pub fn start_tray(main_hwnd: isize, ctx: egui::Context) -> (TrayGuard, mpsc::Receiver<TrayAction>) {
    MAIN_HWND.store(main_hwnd, Ordering::SeqCst);
    TRAY_ICON_REMOVED.store(false, Ordering::SeqCst);
    crate::app_tray::TRAY_ICON_FAILED.store(false, Ordering::SeqCst);
    let _ = TRAY_CTX.set(ctx);
    let (tx, rx) = mpsc::channel();
    let _ = TRAY_TX.set(Mutex::new(Some(tx)));
    let handle =
        thread::Builder::new().name("pulse-tray".into()).spawn(tray_thread).expect("spawn tray");
    let mut hwnd = 0isize;
    for _ in 0..50 {
        hwnd = TRAY_HWND.load(Ordering::SeqCst);
        if hwnd != 0 {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    (
        TrayGuard {
            handle: Some(handle),
            hwnd,
            _cleanup: crate::platform::suspend::SuspendCleanupGuard,
        },
        rx,
    )
}

impl Drop for TrayGuard {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        const WM_CANCELMODE: u32 = 0x001F;
        const WM_CLOSE: u32 = 0x0010;
        let hwnd = if self.hwnd != 0 { self.hwnd } else { TRAY_HWND.load(Ordering::SeqCst) };
        if hwnd != 0 {
            unsafe {
                PostMessageW(hwnd, WM_CANCELMODE, 0, 0);
                PostMessageW(hwnd, WM_CLOSE, 0, 0);
            }
            remove_icon(hwnd);
        }
        if let Some(h) = self.handle.take() {
            let deadline = Instant::now() + Duration::from_millis(SHUTDOWN_WAIT_MS);
            while !h.is_finished() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(5));
            }
            if h.is_finished() {
                let _ = h.join();
            }
        }
    }
}

#[allow(unsafe_code)]
pub(super) fn remove_icon(hwnd: isize) {
    if TRAY_ICON_REMOVED.swap(true, Ordering::SeqCst) {
        return;
    }
    let nid = NotifyIconDataW {
        cb_size: std::mem::size_of::<NotifyIconDataW>() as u32,
        hwnd,
        uid: 1,
        u_flags: 0,
        callback_message: 0,
        h_icon: 0,
        sz_tip: [0; 128],
        dw_state: 0,
        dw_state_mask: 0,
        sz_info: [0; 256],
        u_version: 0,
        sz_info_title: [0; 64],
        dw_info_flags: 0,
        guid_item: [0; 16],
        h_balloon_icon: 0,
    };
    const NIM_DELETE: u32 = 2;
    unsafe {
        Shell_NotifyIconW(NIM_DELETE, &nid);
    }
}
