use super::tray_win_core::{
    encode_tray_tip, remove_icon, set_allow_real_close, AppendMenuW, CreatePopupMenu,
    CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW, GetCursorPos, GetMessageW,
    GetModuleHandleW, LoadIconW, Msg, NotifyIconDataW, Point, PostMessageW, PostQuitMessage,
    RegisterClassW, SetForegroundWindow, Shell_NotifyIconW, TrackPopupMenu, TranslateMessage,
    WndClassW, MAIN_HWND, TRAY_CTX, TRAY_HWND, TRAY_TX,
};
use super::TrayAction;
use crate::platform::hotkey;
use std::sync::atomic::Ordering;
use windows_sys::Win32::System::Threading::GetCurrentProcessId;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsIconic, ShowWindow,
    SW_RESTORE, SW_SHOW, WM_NULL,
};
const NIM_ADD: u32 = 0;
fn repaint_tray() {
    if let Some(ctx) = TRAY_CTX.get() {
        ctx.request_repaint();
    }
}
/// Adds the tray icon, retries once, and records a persistent failure flag.
fn add_icon_with(notify: impl Fn() -> bool) -> (bool, usize) {
    for attempt in 1..=2 {
        if notify() {
            return (true, attempt);
        }
    }
    crate::app_tray::TRAY_ICON_FAILED.store(true, Ordering::SeqCst);
    (false, 2)
}
#[allow(unsafe_code)]
pub(super) fn try_add_icon(nid: &NotifyIconDataW) -> bool {
    add_icon_with(|| unsafe { Shell_NotifyIconW(NIM_ADD, nid) != 0 }).0
}
fn send_action(a: TrayAction) {
    if let Some(m) = TRAY_TX.get() {
        if let Ok(g) = m.lock() {
            if let Some(tx) = g.as_ref() {
                let _ = tx.send(a);
            }
        }
    }
}

/// Enumerates top-level windows WITHOUT an IsWindowVisible filter, so a hidden
/// "\u{25B2} PULSE" window is still found after SW_HIDE.
#[allow(unsafe_code)]
pub(super) fn find_main_hwnd() -> isize {
    unsafe extern "system" fn cb(hwnd: isize, lparam: isize) -> i32 {
        let found = lparam as *mut isize;
        if !found.is_null() && unsafe { *found } == 0 {
            let mut pid: u32 = 0;
            if unsafe { GetWindowThreadProcessId(hwnd, &mut pid) } != 0
                && pid == unsafe { GetCurrentProcessId() }
            {
                let mut buf = [0u16; 256];
                let n = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
                if n > 0 {
                    let title = String::from_utf16_lossy(&buf[..n as usize]);
                    if crate::platform::sys_metrics::is_pulse_main_window_title(&title) {
                        unsafe { *found = hwnd };
                        return 0;
                    }
                }
            }
        }
        1
    }
    let mut found: isize = 0;
    unsafe { EnumWindows(Some(cb), &mut found as *mut _ as isize) };
    found
}
pub(super) fn restore_target(hwnd: isize, cached: isize, looked_up: isize) -> isize {
    [hwnd, cached, looked_up].into_iter().find(|&v| v != 0).unwrap_or(0)
}
/// Restores from the tray thread: App::update() is paused while SW_HIDE is active.
#[allow(unsafe_code)]
pub(super) fn restore_window_now(hwnd: isize, notify: bool) {
    let target = restore_target(hwnd, MAIN_HWND.load(Ordering::SeqCst), find_main_hwnd());
    if target == 0 {
        return;
    }
    crate::core::worker_policy::set_tray_hidden(false);
    unsafe {
        if IsIconic(target) == 0 {
            ShowWindow(target, SW_SHOW);
        }
        ShowWindow(target, SW_RESTORE);
        BringWindowToTop(target);
        SetForegroundWindow(target);
        PostMessageW(target, WM_NULL, 0, 0);
    }
    repaint_tray();
    if notify {
        send_action(TrayAction::Open);
    }
}
#[allow(unsafe_code)]
pub(super) fn tray_thread() {
    const WM_TRAY: u32 = 0x0400 + 200;
    const IDI_APPLICATION: usize = 32512;
    const NIF_MESSAGE: u32 = 1;
    const NIF_ICON: u32 = 2;
    const NIF_TIP: u32 = 4;
    unsafe extern "system" fn wnd_proc(
        hwnd: isize,
        msg: u32,
        wparam: usize,
        lparam: isize,
    ) -> isize {
        const WM_COMMAND: u32 = 0x0111;
        const WM_CLOSE: u32 = 0x0010;
        const WM_DESTROY: u32 = 0x0002;
        const WM_RBUTTONUP: u32 = 0x0205;
        const WM_CONTEXTMENU: u32 = 0x007B;
        const WM_LBUTTONUP: u32 = 0x0202;
        const WM_LBUTTONDBLCLK: u32 = 0x0203;
        const WM_HOTKEY: u32 = hotkey::WM_HOTKEY;
        match msg {
            x if x == WM_TRAY => {
                let ev = lparam as u32;
                if ev == WM_RBUTTONUP || ev == WM_CONTEXTMENU {
                    show_menu(hwnd);
                } else if ev == WM_LBUTTONUP || ev == WM_LBUTTONDBLCLK {
                    restore_window_now(MAIN_HWND.load(Ordering::SeqCst), true);
                }
                0
            }
            WM_COMMAND => {
                let id = (wparam & 0xFFFF) as u32;
                match id {
                    1 => restore_window_now(MAIN_HWND.load(Ordering::SeqCst), true),
                    2 => send_action(TrayAction::TogglePause),
                    3 => {
                        set_allow_real_close(true);
                        remove_icon(hwnd);
                        let main = MAIN_HWND.load(Ordering::SeqCst);
                        if main != 0 {
                            PostMessageW(main, WM_CLOSE, 0, 0);
                        }
                        repaint_tray();
                        PostQuitMessage(0);
                        send_action(TrayAction::Exit);
                    }
                    _ => {}
                }
                0
            }
            x if x == WM_HOTKEY => {
                if hotkey::is_global_hotkey_enabled() && wparam as i32 == hotkey::HOTKEY_ID_MINI_HUD
                {
                    let main = MAIN_HWND.load(Ordering::SeqCst);
                    if crate::core::worker_policy::is_tray_hidden() {
                        restore_window_now(main, true);
                        hotkey::set_mini_hud_active(true);
                    } else {
                        hotkey::request_mini_hud_toggle();
                    }
                    // Wake winit/eframe even when the main window is unfocused.
                    if main != 0 {
                        PostMessageW(main, WM_NULL, 0, 0);
                    }
                    repaint_tray();
                }
                0
            }
            x if x == hotkey::WM_APP_REBIND_HOTKEY => {
                hotkey::rebind_global_hotkey(hwnd);
                repaint_tray();
                0
            }
            WM_CLOSE | WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
    fn show_menu(hwnd: isize) {
        unsafe {
            let menu = CreatePopupMenu();
            for (i, label) in ["Open PULSE\0", "Pause / Resume\0", "Exit\0"].iter().enumerate() {
                AppendMenuW(menu, 0, i + 1, to_wide(label).as_ptr());
            }
            let mut pt = Point { x: 0, y: 0 };
            GetCursorPos(&mut pt);
            SetForegroundWindow(hwnd);
            const TPM_RIGHTBUTTON: u32 = 0x0002;
            const TPM_BOTTOMALIGN: u32 = 0x0020;
            TrackPopupMenu(
                menu,
                TPM_RIGHTBUTTON | TPM_BOTTOMALIGN,
                pt.x,
                pt.y,
                0,
                hwnd,
                std::ptr::null(),
            );
            PostMessageW(hwnd, 0, 0, 0);
            DestroyMenu(menu);
        }
    }
    fn to_wide(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }
    let hinst = unsafe { GetModuleHandleW(std::ptr::null()) };
    let class_name = to_wide("PulseTrayClass\0");
    let wc = WndClassW {
        style: 0,
        lpfn: wnd_proc,
        cb_cls: 0,
        cb_wnd: 0,
        hinst,
        hicon: 0,
        hcursor: 0,
        hbr: 0,
        menu: std::ptr::null(),
        class: class_name.as_ptr(),
    };
    unsafe {
        RegisterClassW(&wc);
        let title = to_wide("PulseTray\0");
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            hinst,
            std::ptr::null_mut(),
        );
        TRAY_HWND.store(hwnd, Ordering::SeqCst);
        hotkey::set_tray_hwnd(hwnd);
        let _hotkey_registered = hotkey::register_global_hotkey(hwnd);
        let icon = LoadIconW(0, IDI_APPLICATION as *const u16);
        let icon = if icon == 0 { LoadIconW(hinst, IDI_APPLICATION as *const u16) } else { icon };
        let mut nid: NotifyIconDataW = std::mem::zeroed();
        nid.cb_size = std::mem::size_of::<NotifyIconDataW>() as u32;
        nid.hwnd = hwnd;
        nid.uid = 1;
        nid.u_flags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        nid.callback_message = WM_TRAY;
        nid.h_icon = icon;
        nid.sz_tip = encode_tray_tip("PULSE");
        let _added = try_add_icon(&nid);
        let mut msg: Msg = std::mem::zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        hotkey::unregister_global_hotkey(hwnd);
        remove_icon(hwnd);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn try_add_icon_fail_retry_failed_sets_error_state() {
        use crate::app_tray::{tray_icon_failed, TRAY_ICON_FAILED};
        TRAY_ICON_FAILED.store(false, Ordering::SeqCst);
        assert_eq!(add_icon_with(|| false), (false, 2));
        assert!(tray_icon_failed());
        TRAY_ICON_FAILED.store(false, Ordering::SeqCst);
        assert_eq!(add_icon_with(|| true), (true, 1));
        assert!(!tray_icon_failed());
    }
}
