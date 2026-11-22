//! Read-only Services (SCM) and Startup (Run) views for the System tab.
use std::sync::{Mutex, OnceLock};

use eframe::egui;

use crate::app_cmd::AppCommand;
use crate::modules::cpu::table_state::CpuTableState;
use crate::platform::services_startup::{
    format_service_state, is_pending_state, matches_service_filter, matches_startup_filter,
    query_startup_entries, query_windows_services, services_partial, startup_partial,
};
use crate::theme::{card_frame, colors};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SystemViewMode {
    #[default]
    GpuVram,
    Services,
    Startup,
}

struct ViewUi {
    mode: SystemViewMode,
    service_filter: String,
    startup_filter: String,
}

static UI: OnceLock<Mutex<ViewUi>> = OnceLock::new();

fn ui_state() -> &'static Mutex<ViewUi> {
    UI.get_or_init(|| {
        Mutex::new(ViewUi {
            mode: SystemViewMode::GpuVram,
            service_filter: String::new(),
            startup_filter: String::new(),
        })
    })
}

/// Final grid cell stretches so both read-only tables fill the panel width.
fn fill_rest(ui: &mut egui::Ui) {
    let w = ui.available_width().max(1.0);
    ui.allocate_exact_size(egui::vec2(w, 1.0), egui::Sense::hover());
}

fn head(ui: &mut egui::Ui, title: &str, shown: usize, total: usize) {
    ui.label(egui::RichText::new(title).monospace().size(12.0).color(colors::TEXT_PRIMARY));
    ui.label(
        egui::RichText::new(format!("{shown} / {total}"))
            .monospace()
            .size(10.0)
            .color(colors::ACCENT_CYAN),
    );
}

fn filter_box(ui: &mut egui::Ui, text: &mut String) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Filter:").monospace().size(10.0));
        ui.text_edit_singleline(text);
    });
}

pub fn render_mode_switcher(ui: &mut egui::Ui) -> SystemViewMode {
    let mut guard = ui_state().lock().unwrap_or_else(|err| err.into_inner());
    ui.horizontal(|ui| {
        for (label, mode) in [
            ("[ GPU & VRAM ]", SystemViewMode::GpuVram),
            ("[ Services (SCM) ]", SystemViewMode::Services),
            ("[ Startup (Run) ]", SystemViewMode::Startup),
        ] {
            if ui.selectable_label(guard.mode == mode, label).clicked() {
                guard.mode = mode;
            }
        }
    });
    guard.mode
}

fn state_color(state: u32) -> egui::Color32 {
    if is_pending_state(state) {
        colors::ACCENT_AMBER
    } else {
        match state {
            4 => colors::ACCENT_CYAN,
            1 => colors::TEXT_SECONDARY,
            _ => colors::TEXT_PRIMARY,
        }
    }
}

pub fn render_services_view(
    ui: &mut egui::Ui,
    cpu_state: &mut CpuTableState,
    cmds: &mut Vec<AppCommand>,
) {
    let refresh = ui.button("[ Refresh ]").clicked();
    let entries = query_windows_services(refresh);
    let filter = {
        let mut guard = ui_state().lock().unwrap_or_else(|err| err.into_inner());
        filter_box(ui, &mut guard.service_filter);
        guard.service_filter.clone()
    };
    let visible: Vec<_> = entries
        .iter()
        .filter(|e| {
            matches_service_filter(
                &filter,
                &e.name,
                &e.display_name,
                format_service_state(e.state),
                e.pid,
            )
        })
        .collect();
    card_frame().show(ui, |ui| {
        head(ui, "[05] WINDOWS SERVICES // SCM (READ ONLY)", visible.len(), entries.len());
        if services_partial() {
            ui.label(
                egui::RichText::new("[ PARTIAL DATA ]")
                    .monospace()
                    .size(10.0)
                    .color(colors::ACCENT_AMBER),
            );
        }
        egui::ScrollArea::vertical().max_height(430.0).auto_shrink([false, false]).show(ui, |ui| {
            egui::Grid::new("system_services_grid").striped(true).show(ui, |ui| {
                for column in ["SERVICE NAME", "DISPLAY NAME", "STATE", "PID"] {
                    ui.label(egui::RichText::new(column).monospace().size(10.0));
                }
                fill_rest(ui);
                ui.end_row();
                for entry in visible {
                    ui.label(egui::RichText::new(&entry.name).monospace().size(10.0));
                    ui.label(egui::RichText::new(&entry.display_name).monospace().size(10.0));
                    let label = format_service_state(entry.state);
                    ui.label(
                        egui::RichText::new(label)
                            .monospace()
                            .size(10.0)
                            .color(state_color(entry.state)),
                    );
                    if entry.pid > 0 {
                        if ui.button(format!("[ PID {} ]", entry.pid)).clicked() {
                            cpu_state.search_query = entry.pid.to_string();
                            cpu_state.selected_pid = Some(entry.pid);
                            cpu_state.mark_dirty();
                            ui.output_mut(|o| o.copied_text = entry.pid.to_string());
                            cmds.push(AppCommand::SetTab(crate::app_tabs::ActiveTab::Processes));
                            cmds.push(AppCommand::RequestProcessScan);
                        }
                    } else {
                        ui.label("--");
                    }
                    fill_rest(ui);
                    ui.end_row();
                }
            });
        });
    });
}

fn short_command(command: &str, limit: usize) -> String {
    if command.chars().count() <= limit {
        command.to_string()
    } else {
        let prefix: String = command.chars().take(limit).collect();
        format!("{prefix}...")
    }
}

pub fn render_startup_view(ui: &mut egui::Ui) {
    let refresh = ui.button("[ Refresh ]").clicked();
    let entries = query_startup_entries(refresh);
    let filter = {
        let mut guard = ui_state().lock().unwrap_or_else(|err| err.into_inner());
        filter_box(ui, &mut guard.startup_filter);
        guard.startup_filter.clone()
    };
    let visible: Vec<_> = entries
        .iter()
        .filter(|e| matches_startup_filter(&filter, e.scope, &e.name, &e.command))
        .collect();
    card_frame().show(ui, |ui| {
        head(ui, "[05] AUTOSTART // RUN KEYS (READ ONLY)", visible.len(), entries.len());
        if startup_partial() {
            ui.label(
                egui::RichText::new("[ PARTIAL DATA ]")
                    .monospace()
                    .size(10.0)
                    .color(colors::ACCENT_AMBER),
            );
        }
        egui::ScrollArea::vertical().max_height(430.0).auto_shrink([false, false]).show(ui, |ui| {
            egui::Grid::new("system_startup_grid").striped(true).show(ui, |ui| {
                for column in ["SCOPE", "NAME", "COMMAND / PATH"] {
                    ui.label(egui::RichText::new(column).monospace().size(10.0));
                }
                fill_rest(ui);
                ui.end_row();
                for entry in visible {
                    ui.label(egui::RichText::new(entry.scope).monospace().size(10.0));
                    ui.label(egui::RichText::new(&entry.name).monospace().size(10.0));
                    let response = ui
                        .label(
                            egui::RichText::new(short_command(&entry.command, 90))
                                .monospace()
                                .size(10.0),
                        )
                        .on_hover_text(&entry.command);
                    if response.clicked() {
                        ui.output_mut(|o| o.copied_text = entry.command.clone());
                    }
                    fill_rest(ui);
                    ui.end_row();
                }
            });
        });
    });
}
