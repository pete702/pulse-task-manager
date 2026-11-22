//! UI helpers for window picking and dropped-file lock reports.
use crate::app_cmd::AppCommand;
use crate::app_tabs::ActiveTab;
use crate::modules::cpu::table_state::CpuTableState;
use crate::modules::cpu::table_tree::CpuViewMode;
use crate::theme::colors;
use eframe::egui;
use std::path::PathBuf;
use std::time::Instant;

/// Draws the pick-window toggle in the process table toolbar.
pub fn render_pick_button(ui: &mut egui::Ui) {
    let armed = crate::platform::pick_window::is_pick_window_armed();
    let label = if armed { "[ Picking... Esc ]" } else { "[ Pick Window ]" };
    let color = if armed { colors::ACCENT_AMBER } else { colors::ACCENT_CYAN };
    if crate::modules::cpu::table_menu::action_button(ui, true, label, color).clicked() {
        if armed {
            crate::platform::pick_window::cancel_pick_window();
        } else {
            crate::platform::pick_window::arm_pick_window();
        }
    }
}

/// Applies crosshair feedback and requests repaints while picking is armed.
/// Continuous polling is required because a click on another window removes focus.
pub fn render_pick_mode_feedback(ctx: &egui::Context) {
    if crate::platform::pick_window::is_pick_window_armed() {
        ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
        ctx.request_repaint_after(std::time::Duration::from_millis(16));
    }
}

fn select_pid(state: &mut CpuTableState, pid: u32, message: String) {
    state.set_view_mode(CpuViewMode::Flat);
    state.search_query.clear();
    state.selected_pid = Some(pid);
    state.selected_start_time = None;
    state.selected_name = None;
    state.mark_dirty();
    state.last_action_msg = Some((message, Instant::now()));
}

/// Resolves a picked window and selects its owning process in the table.
pub fn poll_pick_into_table(state: &mut CpuTableState, cmds: &mut Vec<AppCommand>) {
    if let Some(pid) = crate::platform::pick_window::poll_picked_window_pid(std::process::id()) {
        select_pid(state, pid, format!("Picked window -> PID {pid}"));
        cmds.push(AppCommand::SetTab(ActiveTab::Processes));
    }
}

/// Runs the read-only lock finder on dropped paths and selects the first holder.
pub fn handle_dropped_files(
    ctx: &egui::Context,
    state: &mut CpuTableState,
    cmds: &mut Vec<AppCommand>,
) {
    let paths: Vec<PathBuf> =
        ctx.input(|i| i.raw.dropped_files.iter().filter_map(|f| f.path.clone()).collect());
    if paths.is_empty() {
        return;
    }
    let report = crate::platform::file_lock::find_file_locks(&paths);
    if let Some(&(pid, _)) = report.holders.first() {
        select_pid(state, pid, format!("Picked lock holder -> PID {pid}"));
        cmds.push(AppCommand::SetTab(ActiveTab::Processes));
    } else {
        state.last_action_msg = Some((report.summary, Instant::now()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picked_pid_selects_row_and_clears_non_matching_filter() {
        let mut state = CpuTableState::default();
        state.search_query = "zzz_no_match".to_owned();
        select_pid(&mut state, 4242, "Picked window -> PID 4242".to_owned());
        assert_eq!(state.selected_pid, Some(4242));
        assert!(state.search_query.is_empty());
        assert_eq!(
            state.last_action_msg.as_ref().map(|m| m.0.as_str()),
            Some("Picked window -> PID 4242")
        );
    }
}
