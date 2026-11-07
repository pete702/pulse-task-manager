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
        egui::RichText::new(item.name.as_ref()).monospace().size(12.0).color(colors::TEXT_PRIMARY),
    );
    ui.label(
        egui::RichText::new(format!("PID {}", item.pid))
            .monospace()
            .size(10.0)
            .color(colors::TEXT_SECONDARY),
    );
    ui.separator();
    if ui.button("Trim Working Set (Free RAM)").clicked() {
        state.last_action_msg = Some(("Working...".to_owned(), Instant::now()));
        let owned = item.clone();
        state.spawn_menu_action(move || {
            let msg = match trim_process_working_set(&owned) {
                Ok(()) => "Working set trimmed".to_owned(),
                Err(error) => format!("Trim blocked: {error}"),
            };
            (msg, None)
        });
        ui.close_menu();
    }
    if ui.button("Enable EcoQoS (Efficiency Mode)").clicked() {
        state.last_action_msg = Some(("Working...".to_owned(), Instant::now()));
        let pid = item.pid;
        let name = item.name.to_string();
        let start = item.start_time;
        state.spawn_menu_action(move || {
            let msg = match enable_eco_qos(pid, &name) {
                Ok(()) => "EcoQoS enabled".to_owned(),
                Err(error) => format!("EcoQoS blocked: {error}"),
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
        let owned = item.clone();
        state.spawn_menu_action(move || {
            let msg = match terminate_process(&owned) {
                Ok(()) => "Task ended".to_owned(),
                Err(error) => format!("End blocked: {error}"),
            };
            (msg, None)
        });
        ui.close_menu();
    }
    ui.separator();
    if ui.button("Copy Name & PID").clicked() {
        ui.output_mut(|output| {
            output.copied_text = format!("{} (PID {})", item.name, item.pid);
        });
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
    let text = if is_active {
        if sort_asc {
            format!("{label} \u{25B2}")
        } else {
            format!("{label} \u{25BC}")
        }
    } else {
        label.to_owned()
    };
    ui.add(
        egui::Button::new(egui::RichText::new(text).monospace().size(11.0).color(if is_active {
            colors::ACCENT_CYAN
        } else {
            colors::TEXT_SECONDARY
        }))
        .frame(false)
        .fill(if is_active {
            colors::ACCENT_CYAN.gamma_multiply(0.1)
        } else {
            colors::BG_CARD
        }),
    )
    .on_hover_text("Click to sort")
}
