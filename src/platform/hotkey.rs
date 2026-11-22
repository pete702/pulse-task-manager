//! Single global Mini-HUD hotkey with registration fallback and a runtime toggle.
//! Primary Ctrl+Shift+F9 avoids Alt+F9 / Ctrl+Alt+F9 (NVIDIA Overlay).

#[cfg(target_os = "windows")]
use std::sync::atomic::AtomicIsize;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[cfg(target_os = "windows")]
pub const HOTKEY_ID_MINI_HUD: i32 = 1;
#[cfg(any(target_os = "windows", test))]
pub const MOD_ALT: u32 = 0x0001;
#[cfg(any(target_os = "windows", test))]
pub const MOD_CONTROL: u32 = 0x0002;
#[cfg(any(target_os = "windows", test))]
pub const MOD_SHIFT: u32 = 0x0004;
#[cfg(any(target_os = "windows", test))]
pub const MOD_NOREPEAT: u32 = 0x4000;
#[cfg(any(target_os = "windows", test))]
pub const VK_F9: u32 = 0x78;
#[cfg(any(target_os = "windows", test))]
pub const VK_F10: u32 = 0x79;
#[cfg(target_os = "windows")]
pub const WM_HOTKEY: u32 = 0x0312;
#[cfg(target_os = "windows")]
pub const WM_APP_REBIND_HOTKEY: u32 = 0x0400 + 201;
#[cfg(any(target_os = "windows", test))]
pub const PRIMARY_HOTKEY: (u32, u32) = (MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, VK_F9);
#[cfg(any(target_os = "windows", test))]
pub const FALLBACK_HOTKEY: (u32, u32) = (MOD_ALT | MOD_SHIFT | MOD_NOREPEAT, VK_F9);
#[cfg(any(target_os = "windows", test))]
pub const THIRD_HOTKEY: (u32, u32) = (MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, VK_F10);
pub const HOTKEY_TOOLTIP: &str =
    "Global Mini-HUD hotkey: Ctrl+Shift+F9 (fallbacks Alt+Shift+F9, Ctrl+Shift+F10)";

/// Registration order: primary first, then two silent fallbacks when occupied.
#[cfg(any(target_os = "windows", test))]
pub fn registration_candidates() -> [(u32, u32); 3] {
    [PRIMARY_HOTKEY, FALLBACK_HOTKEY, THIRD_HOTKEY]
}

static MINI_HUD_TOGGLE_REQUEST: AtomicBool = AtomicBool::new(false);
static MINI_HUD_ACTIVE: AtomicBool = AtomicBool::new(false);
static GLOBAL_HOTKEY_ENABLED: AtomicBool = AtomicBool::new(true);
static ACTIVE_CANDIDATE: AtomicUsize = AtomicUsize::new(0);
static HOTKEY_BUSY: AtomicBool = AtomicBool::new(false);
#[cfg(target_os = "windows")]
static TRAY_HWND_HOTKEY: AtomicIsize = AtomicIsize::new(0);

pub fn is_global_hotkey_enabled() -> bool {
    GLOBAL_HOTKEY_ENABLED.load(Ordering::SeqCst)
}

pub fn set_global_hotkey_enabled(enabled: bool) {
    GLOBAL_HOTKEY_ENABLED.store(enabled, Ordering::SeqCst);
}

/// Short state label for the status bar and tray tooltip.
pub fn hotkey_status_label() -> &'static str {
    if !is_global_hotkey_enabled() {
        "Off"
    } else if HOTKEY_BUSY.load(Ordering::SeqCst) {
        "Busy"
    } else {
        match ACTIVE_CANDIDATE.load(Ordering::SeqCst) {
            1 => "Alt+Shift+F9",
            2 => "Ctrl+Shift+F10",
            _ => "Ctrl+Shift+F9",
        }
    }
}

#[cfg(target_os = "windows")]
pub fn set_tray_hwnd(hwnd: isize) {
    TRAY_HWND_HOTKEY.store(hwnd, Ordering::SeqCst);
}

/// Posts a rebind request to the tray message thread; no restart required.
#[allow(unsafe_code)]
pub fn request_rebind() {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW;
        let hwnd = TRAY_HWND_HOTKEY.load(Ordering::SeqCst);
        if hwnd != 0 {
            unsafe {
                PostMessageW(
                    hwnd as windows_sys::Win32::Foundation::HWND,
                    WM_APP_REBIND_HOTKEY,
                    0,
                    0,
                );
            }
        }
    }
}

#[cfg(any(target_os = "windows", test))]
pub fn request_mini_hud_toggle() {
    MINI_HUD_TOGGLE_REQUEST.store(true, Ordering::SeqCst);
}

pub fn take_mini_hud_toggle_request() -> bool {
    MINI_HUD_TOGGLE_REQUEST.swap(false, Ordering::SeqCst)
}

pub fn is_mini_hud_active() -> bool {
    MINI_HUD_ACTIVE.load(Ordering::SeqCst)
}

pub fn set_mini_hud_active(active: bool) {
    MINI_HUD_ACTIVE.store(active, Ordering::SeqCst);
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn register_global_hotkey(hwnd: isize) -> bool {
    use windows_sys::Win32::Foundation::{GetLastError, HWND};
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey;
    if !is_global_hotkey_enabled() {
        ACTIVE_CANDIDATE.store(0, Ordering::SeqCst);
        HOTKEY_BUSY.store(false, Ordering::SeqCst);
        return true;
    }
    for (idx, (mods, vk)) in registration_candidates().into_iter().enumerate() {
        if unsafe { RegisterHotKey(hwnd as HWND, HOTKEY_ID_MINI_HUD, mods, vk) } != 0 {
            ACTIVE_CANDIDATE.store(idx, Ordering::SeqCst);
            HOTKEY_BUSY.store(false, Ordering::SeqCst);
            return true;
        }
        let _last_error = unsafe { GetLastError() };
    }
    ACTIVE_CANDIDATE.store(0, Ordering::SeqCst);
    HOTKEY_BUSY.store(true, Ordering::SeqCst);
    false
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn rebind_global_hotkey(hwnd: isize) {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::UnregisterHotKey;
    unsafe { UnregisterHotKey(hwnd as HWND, HOTKEY_ID_MINI_HUD) };
    register_global_hotkey(hwnd);
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn unregister_global_hotkey(hwnd: isize) {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::UnregisterHotKey;
    unsafe { UnregisterHotKey(hwnd as HWND, HOTKEY_ID_MINI_HUD) };
    ACTIVE_CANDIDATE.store(0, Ordering::SeqCst);
    HOTKEY_BUSY.store(false, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn primary_and_fallback_combos_are_exact() {
        assert_eq!(PRIMARY_HOTKEY, (MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, VK_F9));
        assert_eq!(FALLBACK_HOTKEY, (MOD_ALT | MOD_SHIFT | MOD_NOREPEAT, VK_F9));
        assert_eq!(THIRD_HOTKEY, (MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, VK_F10));
        assert_eq!(registration_candidates(), [PRIMARY_HOTKEY, FALLBACK_HOTKEY, THIRD_HOTKEY]);
        assert!(HOTKEY_TOOLTIP.contains("Ctrl+Shift+F10"));
    }

    #[test]
    fn combos_avoid_nvidia_overlay_shortcuts() {
        for (mods, vk) in registration_candidates() {
            assert!(vk == VK_F9 || vk == VK_F10);
            assert_ne!(mods, MOD_ALT | MOD_NOREPEAT);
            assert_ne!(mods, MOD_CONTROL | MOD_ALT | MOD_NOREPEAT);
        }
    }

    #[test]
    fn enabled_toggle_controls_status_label() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        HOTKEY_BUSY.store(false, Ordering::SeqCst);
        ACTIVE_CANDIDATE.store(0, Ordering::SeqCst);
        set_global_hotkey_enabled(false);
        assert!(!is_global_hotkey_enabled());
        assert_eq!(hotkey_status_label(), "Off");
        set_global_hotkey_enabled(true);
        assert!(is_global_hotkey_enabled());
        assert_eq!(hotkey_status_label(), "Ctrl+Shift+F9");
        ACTIVE_CANDIDATE.store(1, Ordering::SeqCst);
        assert_eq!(hotkey_status_label(), "Alt+Shift+F9");
        ACTIVE_CANDIDATE.store(2, Ordering::SeqCst);
        assert_eq!(hotkey_status_label(), "Ctrl+Shift+F10");
        HOTKEY_BUSY.store(true, Ordering::SeqCst);
        assert_eq!(hotkey_status_label(), "Busy");
        HOTKEY_BUSY.store(false, Ordering::SeqCst);
        ACTIVE_CANDIDATE.store(0, Ordering::SeqCst);
    }

    #[test]
    fn third_fallback_label_is_exact() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        HOTKEY_BUSY.store(false, Ordering::SeqCst);
        ACTIVE_CANDIDATE.store(2, Ordering::SeqCst);
        assert_eq!(hotkey_status_label(), "Ctrl+Shift+F10");
        ACTIVE_CANDIDATE.store(0, Ordering::SeqCst);
    }
}
