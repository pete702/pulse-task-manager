//! Virtualized CPU process table with search, clone grouping, sorting, and actions.

use crate::modules::cpu::model::{CpuSnapshot, ProcessCpuItem};
use crate::modules::cpu::qos_action::{
    enable_eco_qos, open_process_file_location, terminate_process, trim_process_working_set,
};
use crate::modules::cpu::table_body::render_virtualized_table;
use crate::modules::cpu::table_state::CpuTableState;
use crate::modules::memory::safe_handle::is_app_elevated;
use crate::theme::{card_frame, colors, OPTIMAL_EMERALD};
use eframe::egui;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

static ELEVATED: OnceLock<bool> = OnceLock::new();
static PURGE_RUNNING: AtomicBool = AtomicBool::new(false);

fn cached_elevated() -> bool {
    *ELEVATED.get_or_init(is_app_elevated)
}

/// Renders the unified virtualized process table with its action toolbar.
pub fn render_cpu_table(
    ui: &mut egui::Ui,
    cpu_snapshot: &CpuSnapshot,
    state: &mut CpuTableState,
    scan_generation: u64,
) {
    state.rebuild_if_needed(scan_generation, cpu_snapshot.processes_raw());
    let rows = state.cached_rows().unwrap_or_default();
    // Keep the action toolbar enabled across rebuilds: when nothing is
    // selected but rows exist, auto-select the first row. A user's click
    // (set in table_body.rs) is preserved because selected_pid is already Some.
    if state.selected_pid.is_none() && !rows.is_empty() {
        state.selected_pid = Some(rows[0].pid);
        state.selected_name = Some(rows[0].name.to_string());
    }
    let total_processes = cpu_snapshot.process_count;
    let shown_rows = rows.len();
    let clone_groups = rows.iter().filter(|row| row.clone_count > 1).count();

    card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            render_toolbar(ui, total_processes, shown_rows, clone_groups, state, &rows);
            ui.add_space(6.0);

            if rows.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new("No processes match filter")
                            .monospace()
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY),
                    );
                });
            } else {
                let height = (ui.available_height() - 4.0).max(120.0);
                render_virtualized_table(ui, &rows, state, height);
            }
        });
    });
}

fn render_toolbar(
    ui: &mut egui::Ui,
    total: usize,
    shown: usize,
    _clone_groups: usize,
    state: &mut CpuTableState,
    rows: &[ProcessCpuItem],
) {
    let selected = state.selected_pid.and_then(|pid| rows.iter().find(|row| row.pid == pid));

    // Row 1: Filter controls (left) + Pause button (right) + Privilege badge + Purge All RAM
    ui.horizontal(|ui| {
        // Privilege badge on the far left (cached: 0 Win32 syscalls per frame)
        let elevated = cached_elevated();
        let (badge_text, badge_color) = if elevated {
            ("[ ADMIN ]", colors::ACCENT_CYAN)
        } else {
            ("[ USER ]", colors::TEXT_SECONDARY)
        };
        ui.label(egui::RichText::new(badge_text).monospace().size(11.0).color(badge_color));
        ui.add_space(8.0);

        ui.label(
            egui::RichText::new("Filter:").monospace().size(11.0).color(colors::TEXT_SECONDARY),
        );
        let filter = ui.add(
            egui::TextEdit::singleline(&mut state.search_query)
                .id(egui::Id::new("cpu_search_filter"))
                .hint_text("Filter process...")
                .desired_width(145.0),
        );
        if state.focus_search_next_frame {
            filter.request_focus();
            state.focus_search_next_frame = false;
        }
        if filter.changed() {
            state.mark_dirty();
        }
        crate::modules::cpu::quick_filter::render_quick_filters(ui, state);
        if ui
            .add_enabled(
                !state.search_query.is_empty(),
                egui::Button::new(egui::RichText::new("[X]").monospace().size(11.0)),
            )
            .clicked()
        {
            state.clear_search();
        }
        let old_group = state.group_clones;
        if ui.checkbox(&mut state.group_clones, "Group Clones").changed() {
            let new_group = state.group_clones;
            state.group_clones = old_group;
            state.set_group_clones(new_group);
        }
        let density_label = if state.comfort_density { "[ Comfort ]" } else { "[ Compact ]" };
        if action_button(ui, true, density_label, colors::TEXT_SECONDARY).clicked() {
            state.comfort_density = !state.comfort_density;
        }
        ui.separator();
        ui.label(
            egui::RichText::new(format!("{} / {} rows", shown, total))
                .monospace()
                .size(11.0)
                .color(colors::TEXT_SECONDARY),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Purge All RAM button
            if action_button(ui, !rows.is_empty(), "[ Purge All RAM ]", colors::ACCENT_CYAN)
                .clicked()
                && PURGE_RUNNING
                    .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
            {
                let owned: Vec<_> = rows.to_vec();
                state.last_action_msg = "Purging All RAM...".to_owned();
                std::thread::spawn(move || {
                    for item in &owned {
                        let _ = trim_process_working_set(item);
                    }
                    PURGE_RUNNING.store(false, Ordering::Release);
                    crate::core::request_process_scan();
                });
            }
            ui.add_space(8.0);
            let pause_label = if state.paused { "> Resume" } else { "|| Pause" };
            if ui.button(pause_label).clicked() {
                state.toggle_pause();
            }
        });
    });

    // Row 2: Actions left-to-right on the left, status on the right.
    ui.horizontal(|ui| {
        let can_act = selected.is_some();
        // Auto-expire confirm_kill after 3 seconds or if selection changed
        if let Some((armed_pid, armed_at)) = state.confirm_kill {
            if Instant::now().duration_since(armed_at).as_secs() >= 3
                || state.selected_pid != Some(armed_pid)
            {
                state.confirm_kill = None;
            }
        }

        if let Some(item) = selected {
            if action_button(ui, can_act, "[ Trim RAM ]", colors::ACCENT_CYAN).clicked() {
                let msg = match trim_process_working_set(item) {
                    Ok(()) => "Working set trimmed".to_owned(),
                    Err(error) => format!("Trim blocked: {error}"),
                };
                state.last_action_msg = msg;
                crate::core::request_process_scan();
            }
            if action_button(ui, can_act, "[ EcoQoS ]", OPTIMAL_EMERALD).clicked() {
                let msg = match enable_eco_qos(item.pid, &item.name) {
                    Ok(()) => "EcoQoS enabled".to_owned(),
                    Err(error) => format!("EcoQoS blocked: {error}"),
                };
                state.last_action_msg = msg;
            }
            if action_button(ui, can_act, "[ Open Location ]", colors::TEXT_PRIMARY).clicked() {
                let msg = match open_process_file_location(item.pid) {
                    Ok(()) => "Opened file location".to_owned(),
                    Err(error) => format!("Open blocked: {error}"),
                };
                state.last_action_msg = msg;
            }
            if let Some((armed_pid, _)) = state.confirm_kill {
                if armed_pid == item.pid {
                    if action_button(ui, true, "[ Cancel ]", colors::TEXT_SECONDARY).clicked() {
                        state.confirm_kill = None;
                        state.last_action_msg = "Kill cancelled".to_owned();
                    }
                    if action_button(ui, true, "[ Confirm Kill (3s) ]", colors::ACCENT_RED)
                        .clicked()
                    {
                        let result = terminate_process(item);
                        state.selected_name = None;
                        state.selected_pid = None;
                        state.confirm_kill = None;
                        let msg = match result {
                            Ok(()) => "Task ended".to_owned(),
                            Err(error) => format!("End blocked: {error}"),
                        };
                        state.last_action_msg = msg;
                        crate::core::request_process_scan();
                    }
                } else {
                    state.confirm_kill = None;
                }
            } else if action_button(ui, can_act, "[ End Task ]", colors::ACCENT_RED).clicked() {
                state.confirm_kill = Some((item.pid, Instant::now()));
                let msg =
                    format!("Click Confirm Kill within 3s to end {} (PID {})", item.name, item.pid);
                state.last_action_msg = msg;
            }
        }

        if !state.last_action_msg.is_empty() {
            let failed = state.last_action_msg.contains("blocked");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = egui::Label::new(
                    egui::RichText::new(state.last_action_msg.as_str())
                        .monospace()
                        .size(10.0)
                        .color(if failed { colors::ACCENT_RED } else { colors::ACCENT_CYAN }),
                )
                .truncate(true);
                ui.add(label);
            });
        }
    });
}

/// Styled action toolbar button: crisp accent text + subtle semantic border/fill
/// when enabled, dimmed when no row is selected.
fn action_button(
    ui: &mut egui::Ui,
    enabled: bool,
    label: &str,
    color: egui::Color32,
) -> egui::Response {
    let button = egui::Button::new(egui::RichText::new(label).monospace().size(11.0).color(color))
        .fill(color.gamma_multiply(0.10));
    if enabled {
        ui.add(button)
    } else {
        ui.add_enabled(false, button)
    }
}
