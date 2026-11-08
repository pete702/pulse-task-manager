//! Virtualized CPU process table with search, clone grouping, sorting, and actions.
use crate::modules::cpu::model::{CpuSnapshot, ProcessCpuItem};
use crate::modules::cpu::qos_action::trim_process_working_set;
use crate::modules::cpu::table_body::render_virtualized_table;
use crate::modules::cpu::table_menu::{action_button, exe_opt};
use crate::modules::cpu::table_state::CpuTableState;
use crate::modules::memory::safe_handle::is_app_elevated;
use crate::theme::{card_frame, colors};
use eframe::egui;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::Instant;
static ELEVATED: OnceLock<bool> = OnceLock::new();
static PURGE_RUNNING: AtomicBool = AtomicBool::new(false);
struct PurgeGuard;
impl Drop for PurgeGuard {
    fn drop(&mut self) {
        PURGE_RUNNING.store(false, Ordering::SeqCst);
    }
}
fn cached_elevated() -> bool {
    *ELEVATED.get_or_init(is_app_elevated)
}
pub fn render_cpu_table(
    ui: &mut egui::Ui,
    cpu_snapshot: &CpuSnapshot,
    state: &mut CpuTableState,
    scan_generation: u64,
) {
    state.poll_menu_action();
    state.rebuild_if_needed(scan_generation, cpu_snapshot.processes_raw());
    let rows = state.cached_rows().unwrap_or_default();
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
fn badge(ui: &mut egui::Ui) {
    let elevated = cached_elevated();
    let (text, color) = if elevated {
        ("[ ADMIN ]", colors::ACCENT_CYAN)
    } else {
        ("[ USER ]", colors::TEXT_SECONDARY)
    };
    ui.label(egui::RichText::new(text).monospace().size(11.0).color(color));
}
fn filter_field(ui: &mut egui::Ui, state: &mut CpuTableState, fw: f32) {
    ui.label(egui::RichText::new("Filter:").monospace().size(11.0).color(colors::TEXT_SECONDARY));
    let filter = ui.add(
        egui::TextEdit::singleline(&mut state.search_query)
            .id(egui::Id::new("cpu_search_filter"))
            .hint_text("Filter process...")
            .desired_width(fw),
    );
    if state.focus_search_next_frame {
        filter.request_focus();
        state.focus_search_next_frame = false;
    }
    if filter.changed() {
        state.mark_dirty();
    }
}
fn clear_btn(ui: &mut egui::Ui, state: &mut CpuTableState) {
    if ui
        .add_enabled(
            !state.search_query.is_empty(),
            egui::Button::new(egui::RichText::new("[X]").monospace().size(11.0)),
        )
        .clicked()
    {
        state.clear_search();
    }
}
fn group_density(ui: &mut egui::Ui, state: &mut CpuTableState) {
    let old = state.group_clones;
    if ui.checkbox(&mut state.group_clones, "Group Clones").changed() {
        let new_group = state.group_clones;
        state.group_clones = old;
        state.set_group_clones(new_group);
    }
    let label = if state.comfort_density { "[ Comfort ]" } else { "[ Compact ]" };
    if action_button(ui, true, label, colors::TEXT_SECONDARY).clicked() {
        state.comfort_density = !state.comfort_density;
    }
}
fn do_purge(rows: &[ProcessCpuItem], state: &mut CpuTableState) -> bool {
    if !PURGE_RUNNING.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_ok() {
        return false;
    }
    let mut heavy: Vec<ProcessCpuItem> =
        rows.iter().filter(|item| item.ram_bytes >= 10 * 1024 * 1024).cloned().collect();
    heavy.sort_by_key(|a| std::cmp::Reverse(a.ram_bytes));
    heavy.truncate(64);
    let owned: Vec<ProcessCpuItem> = heavy;
    state.last_action_msg = Some(("Purging RAM...".to_owned(), Instant::now()));
    std::thread::spawn(move || {
        let _guard = PurgeGuard;
        for item in &owned {
            let exe = exe_opt(item);
            let _ = trim_process_working_set(item.pid, &item.name, item.creation_time, exe);
        }
        crate::core::request_process_scan();
    });
    true
}
fn right_controls(
    ui: &mut egui::Ui,
    total: usize,
    shown: usize,
    rows: &[ProcessCpuItem],
    state: &mut CpuTableState,
) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if action_button(ui, !rows.is_empty(), "[ Purge All RAM ]", colors::ACCENT_CYAN).clicked() {
            do_purge(rows, state);
        }
        ui.add_space(8.0);
        let pause_label = if state.paused { "> Resume" } else { "|| Pause" };
        if ui.button(pause_label).clicked() {
            state.toggle_pause();
        }
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(format!("{} / {} rows", shown, total))
                .monospace()
                .size(11.0)
                .color(colors::TEXT_SECONDARY),
        );
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
    let avail_w = ui.available_width();
    if avail_w >= 1180.0 {
        ui.horizontal(|ui| {
            let right_w = 300.0;
            let left_w = (avail_w - right_w - 8.0).max(240.0);
            ui.allocate_ui_with_layout(
                egui::vec2(left_w, 0.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    let fw = (avail_w * 0.22).clamp(140.0, 180.0);
                    badge(ui);
                    ui.add_space(8.0);
                    filter_field(ui, state, fw);
                    crate::modules::cpu::quick_filter::render_quick_filters(ui, state);
                    clear_btn(ui, state);
                    group_density(ui, state);
                },
            );
            ui.allocate_ui_with_layout(
                egui::vec2(right_w, 0.0),
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    if action_button(ui, !rows.is_empty(), "[ Purge All RAM ]", colors::ACCENT_CYAN)
                        .clicked()
                    {
                        do_purge(rows, state);
                    }
                    ui.add_space(8.0);
                    let pause_label = if state.paused { "> Resume" } else { "|| Pause" };
                    if ui.button(pause_label).clicked() {
                        state.toggle_pause();
                    }
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(format!("{} / {} rows", shown, total))
                            .monospace()
                            .size(11.0)
                            .color(colors::TEXT_SECONDARY),
                    );
                },
            );
        });
    } else {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let fw = (avail_w * 0.30).clamp(140.0, 180.0);
                badge(ui);
                ui.add_space(8.0);
                filter_field(ui, state, fw);
                clear_btn(ui, state);
                crate::modules::cpu::quick_filter::render_quick_filters(ui, state);
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                group_density(ui, state);
                right_controls(ui, total, shown, rows, state);
            });
        });
    }
    crate::modules::cpu::table_menu::render_bottom_action_bar(ui, state, selected);
}
