//! Process row context menu and sort button for PULSE CPU table.
//! Split from table_body.rs for line-limit compliance.
use std::time::Instant;

use crate::modules::cpu::model::ProcessCpuItem;
use crate::modules::cpu::qos_action::{
    enable_eco_qos, open_process_file_location, terminate_process, trim_process_working_set,
};
use crate::modules::cpu::table_state::CpuTableState;
use crate::theme::colors;
use eframe::egui;

pub fn show_row_menu(ui: &mut egui::Ui, item: &ProcessCpuItem, state: &mut CpuTableState) {
    ui.set_min_width(240.0);
    ui.label(
        egui::RichText::new(format!("{}\nPID {}", item.name, item.pid))
            .monospace()
            .size(11.0)
            .color(colors::TEXT_PRIMARY),
    );
    ui.separator();
    if ui.button("Trim Working Set (Free RAM)").clicked() {
        state.last_action_msg = Some(("Working...".to_owned(), Instant::now()));
        let (pid, name, ct, exe) =
            (item.pid, item.name.to_string(), item.creation_time, exe_opt(item));
        state.spawn_menu_action(move || {
            let msg = match trim_process_working_set(pid, &name, ct, exe) {
                Ok(()) => "Working set trimmed".to_owned(),
                Err(e) => format!("Trim blocked: {e}"),
            };
            (msg, None)
        });
        ui.close_menu();
    }
    if ui.button("Enable EcoQoS (Efficiency Mode)").clicked() {
        state.last_action_msg = Some(("Working...".to_owned(), Instant::now()));
        let (pid, name, start, ct, exe) =
            (item.pid, item.name.to_string(), item.start_time, item.creation_time, exe_opt(item));
        state.spawn_menu_action(move || {
            let msg = match enable_eco_qos(pid, &name, ct, exe) {
                Ok(()) => "EcoQoS enabled".to_owned(),
                Err(e) => format!("EcoQoS blocked: {e}"),
            };
            let eco = if msg == "EcoQoS enabled" { Some((pid, start, true)) } else { None };
            (msg, eco)
        });
        ui.close_menu();
    }
    if ui.button("Open File Location").clicked() {
        state.last_action_msg = Some(("Working...".to_owned(), Instant::now()));
        let pid = item.pid;
        state.spawn_menu_action(move || {
            let msg = match open_process_file_location(pid) {
                Ok(()) => "Opened file location".to_owned(),
                Err(error) => format!("Open blocked: {error}"),
            };
            (msg, None)
        });
        ui.close_menu();
    }
    ui.separator();
    if ui
        .add(egui::Button::new(egui::RichText::new("Terminate Process").color(colors::ACCENT_RED)))
        .clicked()
    {
        state.last_action_msg = Some(("Working...".to_owned(), Instant::now()));
        state.selected_pid = None;
        state.selected_name = None;
        state.confirm_kill = None;
        let (pid, name, ct, exe) =
            (item.pid, item.name.to_string(), item.creation_time, exe_opt(item));
        state.spawn_menu_action(move || {
            let msg = match terminate_process(pid, &name, ct, exe) {
                Ok(()) => "Task ended".to_owned(),
                Err(e) => format!("End blocked: {e}"),
            };
            (msg, None)
        });
        ui.close_menu();
    }
    ui.separator();
    if ui.button("Copy Name & PID").clicked() {
        ui.output_mut(|o| o.copied_text = format!("{} (PID {})", item.name, item.pid));
        ui.close_menu();
    }
}

/// Renders a sort button with an ASCII direction indicator.
pub fn render_sort_button(
    ui: &mut egui::Ui,
    label: &str,
    is_active: bool,
    sort_asc: bool,
) -> egui::Response {
    let arrow = if sort_asc { "\u{25B2}" } else { "\u{25BC}" };
    let text = if is_active { format!("{label} {arrow}") } else { label.to_owned() };
    let color = if is_active { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY };
    let fill = if is_active { colors::ACCENT_CYAN.gamma_multiply(0.1) } else { colors::BG_CARD };
    ui.add(
        egui::Button::new(egui::RichText::new(text).monospace().size(11.0).color(color))
            .frame(false)
            .fill(fill),
    )
    .on_hover_text("Click to sort")
}

pub(super) fn exe_opt(item: &ProcessCpuItem) -> Option<String> {
    if item.exe_path.is_empty() {
        None
    } else {
        Some(item.exe_path.to_string())
    }
}

pub(super) fn action_button(
    ui: &mut egui::Ui,
    enabled: bool,
    label: &str,
    color: egui::Color32,
) -> egui::Response {
    let b = egui::Button::new(egui::RichText::new(label).monospace().size(11.0).color(color))
        .fill(color.gamma_multiply(0.10));
    if enabled {
        ui.add(b)
    } else {
        ui.add_enabled(false, b)
    }
}

pub(super) fn render_bottom_action_bar(
    ui: &mut egui::Ui,
    state: &mut CpuTableState,
    selected: Option<&ProcessCpuItem>,
) {
    ui.horizontal(|ui| {
        let can_act = selected.is_some();
        state.confirm_kill = state.confirm_kill.filter(|(pid, at)| {
            Instant::now().duration_since(*at).as_secs() < 3 && state.selected_pid == Some(*pid)
        });
        if let Some(item) = selected {
            if action_button(ui, can_act, "[ Trim RAM ]", colors::ACCENT_CYAN).clicked() {
                state.last_action_msg = Some(("Working...".to_owned(), Instant::now()));
                let (pid, name, ct, exe) =
                    (item.pid, item.name.to_string(), item.creation_time, exe_opt(item));
                state.spawn_menu_action(move || {
                    let msg = match trim_process_working_set(pid, &name, ct, exe) {
                        Ok(()) => "Working set trimmed".to_owned(),
                        Err(e) => format!("Trim blocked: {e}"),
                    };
                    (msg, None)
                });
            }
            if action_button(ui, can_act, "[ EcoQoS ]", crate::theme::OPTIMAL_EMERALD).clicked() {
                state.last_action_msg = Some(("Working...".to_owned(), Instant::now()));
                let (pid, name, start, ct, exe) = (
                    item.pid,
                    item.name.to_string(),
                    item.start_time,
                    item.creation_time,
                    exe_opt(item),
                );
                state.spawn_menu_action(move || {
                    let msg = match enable_eco_qos(pid, &name, ct, exe) {
                        Ok(()) => "EcoQoS enabled".to_owned(),
                        Err(e) => format!("EcoQoS blocked: {e}"),
                    };
                    let eco = if msg == "EcoQoS enabled" { Some((pid, start, true)) } else { None };
                    (msg, eco)
                });
            }
            if action_button(ui, can_act, "[ Open Location ]", colors::TEXT_PRIMARY).clicked() {
                state.last_action_msg = Some(("Working...".to_owned(), Instant::now()));
                let pid = item.pid;
                state.spawn_menu_action(move || {
                    let msg = match open_process_file_location(pid) {
                        Ok(()) => "Opened file location".to_owned(),
                        Err(e) => format!("Open blocked: {e}"),
                    };
                    (msg, None)
                });
            }
            if let Some((armed_pid, _)) = state.confirm_kill {
                if armed_pid == item.pid {
                    if action_button(ui, true, "[ Cancel ]", colors::TEXT_SECONDARY).clicked() {
                        state.confirm_kill = None;
                        state.last_action_msg = Some(("Kill cancelled".to_owned(), Instant::now()));
                    }
                    if action_button(ui, true, "[ Confirm Kill (3s) ]", colors::ACCENT_RED)
                        .clicked()
                    {
                        state.selected_name = None;
                        state.selected_pid = None;
                        state.confirm_kill = None;
                        state.last_action_msg = Some(("Working...".to_owned(), Instant::now()));
                        let (pid, name, ct, exe) =
                            (item.pid, item.name.to_string(), item.creation_time, exe_opt(item));
                        state.spawn_menu_action(move || {
                            let msg = match terminate_process(pid, &name, ct, exe) {
                                Ok(()) => "Task ended".to_owned(),
                                Err(e) => format!("End blocked: {e}"),
                            };
                            (msg, None)
                        });
                    }
                } else {
                    state.confirm_kill = None;
                }
            } else if action_button(ui, can_act, "[ End Task ]", colors::ACCENT_RED).clicked() {
                state.confirm_kill = Some((item.pid, Instant::now()));
                let msg =
                    format!("Click Confirm Kill within 3s to end {} (PID {})", item.name, item.pid);
                state.last_action_msg = Some((msg, Instant::now()));
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let msg = state.last_action_msg.as_ref().map(|(s, _)| s.as_str()).unwrap_or("");
            let color = if msg.contains("blocked") {
                colors::ACCENT_RED
            } else if msg.is_empty() {
                colors::TEXT_SECONDARY
            } else {
                colors::ACCENT_CYAN
            };
            let label = egui::Label::new(
                egui::RichText::new(if msg.is_empty() { " " } else { msg })
                    .monospace()
                    .size(10.0)
                    .color(color),
            )
            .truncate(true);
            ui.add_sized(egui::Vec2::new(280.0, 18.0), label);
        });
    });
}
