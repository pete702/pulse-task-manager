//! Memory hardware cockpit: RAM, commit charge, kernel tiles, and history chart.

use eframe::egui;

use crate::app_tabs::ActiveTab;
use crate::modules::cpu::table_state::{CpuSortCol, CpuTableState};
use crate::modules::memory::action_bar::MemoryActionBarState;
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::model::MemorySnapshot;
use crate::modules::memory::table_state::MemoryTableState;
use crate::modules::memory::view;
use crate::theme::{card_frame, colors, load_color};

/// Legacy sub-view selector kept for API/test compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MemorySubView {
    #[default]
    Overview,
    Processes,
}

impl MemorySubView {
    /// Switcher label with the live process count.
    pub fn label(self, process_count: usize) -> String {
        match self {
            Self::Overview => "Overview & Charts".to_owned(),
            Self::Processes => format!("RAM Processes ({process_count})"),
        }
    }
}

/// Renders the Memory hardware cockpit.
#[allow(clippy::too_many_arguments)]
pub fn render_memory_page(
    ui: &mut egui::Ui,
    mem: &MemorySnapshot,
    history: &mut MemoryHistory,
    _sub_view: &mut MemorySubView,
    _table_state: &mut MemoryTableState,
    _action_state: &mut MemoryActionBarState,
    _uptime_secs: u64,
    _telemetry_paused: &mut bool,
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
                    cpu_table_state.set_sort(CpuSortCol::RamBytes);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subview_default() {
        assert_eq!(MemorySubView::default(), MemorySubView::Overview);
    }

    #[test]
    fn test_subview_labels() {
        assert_eq!(MemorySubView::Overview.label(0), "Overview & Charts");
        assert_eq!(MemorySubView::Processes.label(123), "RAM Processes (123)");
    }
}
