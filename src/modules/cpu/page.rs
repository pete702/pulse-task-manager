//! CPU hardware cockpit: charts, per-core grid, and the master Processes entry.

use crate::app_tabs::ActiveTab;
use crate::modules::cpu::chart_axis::{selected_window, set_window};
use crate::modules::cpu::chart_view::render_cpu_chart;
use crate::modules::cpu::core_grid::render_core_grid;
use crate::modules::cpu::history::{CpuChartWindow, CpuHistory};
use crate::modules::cpu::model::CpuSnapshot;
use crate::modules::cpu::table::render_cpu_table;
use crate::modules::cpu::table_state::{CpuSortCol, CpuTableState};
use crate::modules::cpu::top_cards::render_cpu_cards_row;
use crate::theme::{card_frame, colors, load_color};
use eframe::egui;

/// Legacy sub-view selector kept for API/test compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CpuSubView {
    #[default]
    Overview,
    Processes,
}

impl CpuSubView {
    pub fn label(self, process_count: usize) -> String {
        match self {
            Self::Overview => "Overview & Charts".to_owned(),
            Self::Processes => format!("CPU Processes ({process_count})"),
        }
    }
}

/// Renders the CPU hardware cockpit.
#[allow(clippy::too_many_arguments)]
pub fn render_cpu_page(
    ui: &mut egui::Ui,
    cpu: &CpuSnapshot,
    history: &mut CpuHistory,
    sub_view: &mut CpuSubView,
    table_state: &mut CpuTableState,
    _uptime_secs: u64,
    active_tab: &mut ActiveTab,
    scan_generation: u64,
) {
    match *sub_view {
        CpuSubView::Overview => {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                card_frame().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("[02] Processor (CPU) // HARDWARE COCKPIT")
                                .monospace()
                                .size(12.0)
                                .color(colors::TEXT_PRIMARY),
                        )
                        .on_hover_text(sub_view.label(cpu.process_count));
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!("{:.1}%", cpu.total_pct))
                                .monospace()
                                .size(13.0)
                                .color(load_color(cpu.total_pct)),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("[ View Processes -> ]").clicked() {
                                table_state.ensure_sort(CpuSortCol::CpuPct, false);
                                *active_tab = ActiveTab::Processes;
                                crate::core::request_process_scan();
                            }
                            let toggle_label = match *sub_view {
                                CpuSubView::Overview => "Processes",
                                CpuSubView::Processes => "Overview",
                            };
                            if ui
                                .button(toggle_label)
                                .on_hover_text(sub_view.label(cpu.process_count))
                                .clicked()
                            {
                                *sub_view = match *sub_view {
                                    CpuSubView::Overview => CpuSubView::Processes,
                                    CpuSubView::Processes => CpuSubView::Overview,
                                };
                            }
                        });
                    });
                });
                ui.add_space(10.0);
                render_overview(ui, cpu, history);
            });
        }
        CpuSubView::Processes => {
            ui.label(
                egui::RichText::new(sub_view.label(cpu.process_count))
                    .monospace()
                    .size(11.0)
                    .color(colors::TEXT_SECONDARY),
            );
            render_processes_page(ui, cpu, table_state, scan_generation);
        }
    }
}

/// Renders the full-screen unified process table.
pub fn render_processes_page(
    ui: &mut egui::Ui,
    cpu: &CpuSnapshot,
    state: &mut CpuTableState,
    scan_generation: u64,
) {
    let ctx = ui.ctx();
    let search_id = egui::Id::new("cpu_search_filter");
    let search_focused = ctx.memory(|m| m.has_focus(search_id));
    if !search_focused
        && !ctx.wants_keyboard_input()
        && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Space))
    {
        state.toggle_pause();
    }
    render_cpu_table(ui, cpu, state, scan_generation);
}

fn render_overview(ui: &mut egui::Ui, cpu: &CpuSnapshot, history: &mut CpuHistory) {
    render_cpu_cards_row(ui, cpu);
    ui.add_space(10.0);

    card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                for (is_grid, label) in [(false, "Live History Chart"), (true, "Per-Core Grid")] {
                    let active = history.show_core_grid() == is_grid;
                    if ui.selectable_label(active, label).clicked() {
                        history.set_show_core_grid(is_grid);
                    }
                }
                if !history.show_core_grid() {
                    ui.separator();
                    let mut window = selected_window(ui.ctx());
                    for w in [
                        CpuChartWindow::OneMin,
                        CpuChartWindow::FiveMin,
                        CpuChartWindow::FifteenMin,
                    ] {
                        let active = window == w;
                        if ui.selectable_label(active, w.label_en()).clicked() {
                            window = w;
                            set_window(ui.ctx(), w);
                        }
                    }
                }
            });
            ui.add_space(8.0);
            if history.show_core_grid() {
                render_core_grid(ui, cpu);
            } else {
                render_cpu_chart(ui, history);
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cpu_subview_default() {
        assert_eq!(CpuSubView::default(), CpuSubView::Overview);
    }
    #[test]
    fn test_cpu_subview_labels() {
        assert_eq!(CpuSubView::Overview.label(0), "Overview & Charts");
        assert_eq!(CpuSubView::Processes.label(123), "CPU Processes (123)");
    }
    #[test]
    fn test_cpu_history_grid_toggle_and_labels() {
        let mut h = CpuHistory::new();
        assert!(!h.show_core_grid());
        h.set_show_core_grid(true);
        assert!(h.show_core_grid());
        h.set_show_core_grid(false);
        assert!(!h.show_core_grid());
        assert!(!CpuSubView::Overview.label(42).is_empty());
        assert!(!CpuSubView::Processes.label(42).is_empty());
    }
}
