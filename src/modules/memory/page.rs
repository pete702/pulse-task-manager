//! Memory hardware cockpit: RAM, commit charge, kernel tiles, and history chart.

use eframe::egui;

use crate::app_tabs::ActiveTab;
use crate::modules::cpu::table_state::{CpuSortCol, CpuTableState};
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::model::MemorySnapshot;
use crate::modules::memory::view;
use crate::theme::{card_frame, colors, load_color};

/// Renders the Memory hardware cockpit.
#[allow(clippy::too_many_arguments)]
pub fn render_memory_page(
    ui: &mut egui::Ui,
    mem: &MemorySnapshot,
    history: &mut MemoryHistory,
    _uptime_secs: u64,
    active_tab: &mut ActiveTab,
    cpu_table_state: &mut CpuTableState,
) {
    let pct = mem.ram_usage_percent();
    card_frame().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("[01] Memory (RAM) // HARDWARE COCKPIT")
                    .monospace()
                    .size(12.0)
                    .color(colors::TEXT_PRIMARY),
            );
            ui.separator();
            ui.label(
                egui::RichText::new(format!("{:.1}%", pct))
                    .monospace()
                    .size(13.0)
                    .color(load_color(pct)),
            );
            ui.label(
                egui::RichText::new(format!(
                    "{:.2} / {:.2} GiB",
                    mem.ram_used_gib(),
                    mem.ram_total_gib()
                ))
                .monospace()
                .size(11.0)
                .color(colors::TEXT_SECONDARY),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("[ View Processes -> ]").clicked() {
                    cpu_table_state.ensure_sort(CpuSortCol::RamBytes, false);
                    *active_tab = ActiveTab::Processes;
                    crate::core::request_process_scan();
                }
            });
        });
    });
    ui.add_space(10.0);

    // Two-level cockpit (top cards + full-height chart), no vertical scrollbar.
    view::render_memory_cards(ui, mem, history);
}
