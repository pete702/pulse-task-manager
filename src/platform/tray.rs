pub enum TrayAction {
    #[cfg(target_os = "windows")]
    Open,
    #[cfg(target_os = "windows")]
    TogglePause,
    #[cfg(target_os = "windows")]
    Exit,
}

#[cfg(target_os = "windows")]
#[path = "tray_win_core.rs"]
mod tray_win_core;
#[cfg(target_os = "windows")]
#[path = "tray_win_thread.rs"]
mod tray_win_thread;

#[cfg(target_os = "windows")]
pub use tray_win_core::{
    allow_real_close, hide_main_window, restore_main_window, set_allow_real_close, start_tray,
    update_tray_tooltip, TrayGuard,
};

#[cfg(not(target_os = "windows"))]
pub struct TrayGuard {
    _cleanup: crate::platform::suspend::SuspendCleanupGuard,
}
#[cfg(not(target_os = "windows"))]
impl Drop for TrayGuard {
    fn drop(&mut self) {}
}
#[cfg(not(target_os = "windows"))]
pub fn allow_real_close() -> bool {
    true
}
#[cfg(not(target_os = "windows"))]
pub fn set_allow_real_close(_v: bool) {}
#[cfg(not(target_os = "windows"))]
pub fn start_tray(
    _main_hwnd: isize,
    _ctx: egui::Context,
) -> (TrayGuard, std::sync::mpsc::Receiver<TrayAction>) {
    let (_tx, rx) = std::sync::mpsc::channel();
    (TrayGuard { _cleanup: crate::platform::suspend::SuspendCleanupGuard }, rx)
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::tray_win_core::encode_tray_tip;
    use super::tray_win_thread::restore_target;
    use crate::platform::sys_metrics::is_pulse_main_window_title;

    #[test]
    fn restore_target_prefers_cached_over_lookup() {
        assert_eq!(restore_target(7, 42, 99), 7);
        assert_eq!(restore_target(0, 42, 99), 42);
        assert_eq!(restore_target(0, 0, 99), 99);
        assert_eq!(restore_target(0, 0, 0), 0);
    }

    #[test]
    fn hidden_window_title_is_recognized() {
        assert!(is_pulse_main_window_title("\u{25B2} PULSE"));
        assert!(is_pulse_main_window_title("PULSE"));
        assert!(!is_pulse_main_window_title("PulseTrayHost"));
        assert!(!is_pulse_main_window_title(""));
    }

    #[test]
    fn tip_truncates_and_nul() {
        let long = "a".repeat(200);
        let t = encode_tray_tip(&long);
        assert_eq!(t[127], 0);
        assert!(t[..127].iter().all(|&c| c == 'a' as u16));
        let short = encode_tray_tip("hi");
        assert_eq!(short[0], 'h' as u16);
        assert_eq!(short[1], 'i' as u16);
        assert_eq!(short[2], 0);
    }
}
