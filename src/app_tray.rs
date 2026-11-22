use crate::app::PulseApp;
use crate::app_cmd::AppCommand;
use crate::core::worker_policy;
use crate::platform::hotkey;
use crate::platform::tray;
pub use crate::platform::tray::set_allow_real_close;
#[cfg(target_os = "windows")]
use crate::platform::tray::TrayAction;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// Set when the tray icon cannot be added even after one retry.
pub static TRAY_ICON_FAILED: AtomicBool = AtomicBool::new(false);
/// Set when an elevated restart cannot restore single-instance ownership.
static ADMIN_RESTART_FAILED: AtomicBool = AtomicBool::new(false);

pub fn tray_icon_failed() -> bool {
    TRAY_ICON_FAILED.load(Ordering::SeqCst)
}

pub fn admin_restart_failed() -> bool {
    ADMIN_RESTART_FAILED.load(Ordering::SeqCst)
}

#[cfg(target_os = "windows")]
pub fn mark_admin_restart_failed() {
    ADMIN_RESTART_FAILED.store(true, Ordering::SeqCst);
}

fn hide_window_to_tray(app: &mut PulseApp) {
    #[cfg(target_os = "windows")]
    let hidden = tray::hide_main_window(app.main_hwnd);
    #[cfg(not(target_os = "windows"))]
    let _ = app;
    #[cfg(not(target_os = "windows"))]
    let hidden = false;
    if hidden {
        worker_policy::set_tray_hidden(true);
    }
    tray::set_allow_real_close(false);
}

impl PulseApp {
    /// Applies a high-level UI command to app state.
    pub fn apply_command(&mut self, cmd: AppCommand) {
        match cmd {
            AppCommand::SetTab(tab) => self.active_tab = tab,
            AppCommand::TogglePause => {
                self.telemetry_paused = !self.telemetry_paused;
            }
            AppCommand::RequestProcessScan => {
                self.worker.request_process_scan();
            }
            AppCommand::RequestMemoryTrim => {
                let ok = crate::core::win32::self_trim_working_set();
                self.last_self_trim =
                    crate::app_trim::next_last_after_attempt(std::time::Instant::now(), ok);
            }
            AppCommand::HideToTray => hide_window_to_tray(self),
        }
    }
}

pub(crate) fn should_use_fast_process_cadence(tab: crate::app_tabs::ActiveTab) -> bool {
    matches!(tab, crate::app_tabs::ActiveTab::Cpu | crate::app_tabs::ActiveTab::Processes)
}

/// Bare F9 always works; Ctrl+Shift+F9 is a fallback only when no global hotkey
/// is actively registered, so WM_HOTKEY and the UI frame cannot both toggle.
pub(crate) fn local_toggle_pressed(
    focused: bool,
    f9: bool,
    ctrl: bool,
    shift: bool,
    global_registered: bool,
) -> bool {
    if !focused || !f9 {
        return false;
    }
    (!ctrl && !shift) || (ctrl && shift && !global_registered)
}

pub fn update(app: &mut PulseApp, ctx: &egui::Context) {
    #[cfg(target_os = "windows")]
    {
        if app.main_hwnd == 0 {
            let hwnd = crate::platform::sys_metrics::get_hwnd(ctx);
            if hwnd != 0 {
                app.main_hwnd = hwnd;
                let _ = crate::platform::sys_metrics::set_dark_title_bar(hwnd);
                let (guard, rx) = tray::start_tray(hwnd, ctx.clone());
                app.tray = Some(guard);
                app.tray_rx = Some(rx);
                app.tray_tooltip_last = Instant::now();
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    if app.tray.is_none() {
        let (guard, _rx) = tray::start_tray(0, ctx.clone());
        app.tray = Some(guard);
    }
    #[cfg(target_os = "windows")]
    {
        let mut actions = Vec::new();
        if let Some(rx) = &app.tray_rx {
            while let Ok(a) = rx.try_recv() {
                actions.push(a);
            }
        }
        for action in actions {
            match action {
                TrayAction::Open => {
                    tray::restore_main_window(app.main_hwnd);
                    worker_policy::set_tray_hidden(false);
                    app.worker.request_process_scan();
                }
                TrayAction::TogglePause => app.apply_command(AppCommand::TogglePause),
                TrayAction::Exit => {
                    tray::set_allow_real_close(true);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }
    #[cfg(target_os = "windows")]
    let global_registered =
        hotkey::is_global_hotkey_enabled() && hotkey::hotkey_status_label() != "Busy";
    #[cfg(not(target_os = "windows"))]
    let global_registered = false;
    if hotkey::take_mini_hud_toggle_request()
        || ctx.input(|i| {
            local_toggle_pressed(
                i.focused,
                i.key_pressed(egui::Key::F9),
                i.modifiers.ctrl,
                i.modifiers.shift,
                global_registered,
            )
        })
    {
        hotkey::set_mini_hud_active(!hotkey::is_mini_hud_active());
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Q) && i.modifiers.ctrl) {
        tray::set_allow_real_close(true);
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
    let close_requested = ctx.input(|i| i.viewport().close_requested());
    if close_requested && should_hide_on_close(app.close_to_tray, tray::allow_real_close()) {
        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        hide_window_to_tray(app);
    }
    if app.tray.is_some() && app.tray_tooltip_last.elapsed() >= Duration::from_secs(2) {
        if let Some(snap) = &app.latest_snapshot {
            let cpu = snap.cpu.as_ref().map(|c| c.total_pct as u32).unwrap_or(0);
            let ram = snap.memory.as_ref().map(|m| m.ram_usage_percent() as u32).unwrap_or(0);
            let _tip = format!(
                "PULSE | CPU {}% | RAM {}% | HK {}",
                cpu,
                ram,
                hotkey::hotkey_status_label()
            );
            #[cfg(target_os = "windows")]
            tray::update_tray_tooltip(&_tip);
            app.tray_tooltip_last = Instant::now();
        }
    }
}

pub(crate) fn should_hide_on_close(close_to_tray: bool, allow_real_close: bool) -> bool {
    close_to_tray && !allow_real_close
}

#[cfg(test)]
mod tests {
    use super::{local_toggle_pressed, should_hide_on_close};

    #[test]
    fn local_f9_and_ctrl_shift_f9_toggle() {
        assert!(local_toggle_pressed(true, true, false, false, true));
        assert!(local_toggle_pressed(true, true, false, false, false));
        assert!(local_toggle_pressed(true, true, true, true, false));
        assert!(!local_toggle_pressed(false, true, true, true, false));
        assert!(!local_toggle_pressed(true, true, true, false, false));
        assert!(!local_toggle_pressed(true, false, false, false, false));
    }

    #[test]
    fn local_ctrl_shift_f9_yields_to_active_global_hotkey() {
        assert!(!local_toggle_pressed(true, true, true, true, true));
        assert!(local_toggle_pressed(true, true, true, true, false));
        assert!(local_toggle_pressed(true, true, false, false, true));
    }

    #[test]
    fn close_to_tray_true_hides_when_close_not_allowed() {
        assert!(should_hide_on_close(true, false));
    }

    #[test]
    fn close_to_tray_false_or_allowed_closes() {
        assert!(!should_hide_on_close(false, false));
        assert!(!should_hide_on_close(true, true));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn allow_real_close_toggles() {
        crate::platform::tray::set_allow_real_close(true);
        assert!(crate::platform::tray::allow_real_close());
        crate::platform::tray::set_allow_real_close(false);
        assert!(!crate::platform::tray::allow_real_close());
    }

    #[test]
    fn nvidia_hijacked_combos_are_not_registered() {
        use crate::platform::hotkey::{
            registration_candidates, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, VK_F9,
        };
        for (mods, vk) in registration_candidates() {
            assert_ne!((mods, vk), (MOD_ALT | MOD_NOREPEAT, VK_F9));
            assert_ne!((mods, vk), (MOD_CONTROL | MOD_ALT | MOD_NOREPEAT, VK_F9));
        }
    }

    #[test]
    fn toggle_request_and_active_state_transitions() {
        use crate::platform::hotkey::{
            is_mini_hud_active, request_mini_hud_toggle, set_mini_hud_active,
            take_mini_hud_toggle_request,
        };
        while take_mini_hud_toggle_request() {}
        set_mini_hud_active(false);
        assert!(!is_mini_hud_active());
        assert!(!take_mini_hud_toggle_request());
        request_mini_hud_toggle();
        assert!(take_mini_hud_toggle_request());
        assert!(!take_mini_hud_toggle_request());
        set_mini_hud_active(!is_mini_hud_active());
        assert!(is_mini_hud_active());
        set_mini_hud_active(!is_mini_hud_active());
        assert!(!is_mini_hud_active());
    }
}
