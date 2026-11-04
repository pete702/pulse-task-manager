//! Memory tab page: sub-view switcher, overview, process table, and action bar.

use eframe::egui;

use crate::modules::memory::action_bar::MemoryActionBarState;
use crate::modules::memory::action_bar_view::render_action_bar;
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::model::MemorySnapshot;
use crate::modules::memory::table_state::MemoryTableState;
use crate::modules::memory::table_view::render_memory_table;
use crate::modules::memory::view;
use crate::theme::{card_frame, colors, load_color};

/// Sub-view selected inside the Memory tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MemorySubView {
    /// RAM/Swap cards, kernel tiles, and the history chart.
    #[default]
    Overview,
    /// Searchable, sortable process memory table.
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

/// Renders the Memory tab with its sub-view switcher.
pub fn render_memory_page(
    ui: &mut egui::Ui,
    mem: &MemorySnapshot,
    history: &mut MemoryHistory,
    sub_view: &mut MemorySubView,
    table_state: &mut MemoryTableState,
    action_state: &mut MemoryActionBarState,
    uptime_secs: u64,
    telemetry_paused: &mut bool,
) {
    card_frame().show(ui, |ui| {
        ui.horizontal(|ui| {
            for view in [MemorySubView::Overview, MemorySubView::Processes] {
                let active = *sub_view == view;
                let text = view.label(mem.processes.len());
                let button = egui::Button::new(
                    egui::RichText::new(text).monospace().size(12.0).color(if active {
                        colors::TEXT_PRIMARY
                    } else {
                        colors::TEXT_SECONDARY
                    }),
                )
                .fill(if active {
                    colors::ACCENT_CYAN.gamma_multiply(0.15)
                } else {
                    colors::BG_CARD
                })
                .frame(active);
                if ui.add(button).clicked() {
                    *sub_view = view;
                }
            }
        });
    });
    ui.add_space(10.0);

    match *sub_view {
        MemorySubView::Overview => {
            view::render_memory_cards(ui, mem, history);
        }
        MemorySubView::Processes => {
            render_compact_ram_bar(ui, mem);
            ui.add_space(8.0);
            render_action_bar(ui, mem, table_state, action_state, telemetry_paused);
            ui.add_space(6.0);
            render_memory_table(ui, mem, table_state, uptime_secs);
        }
    }
}

fn render_compact_ram_bar(ui: &mut egui::Ui, mem: &MemorySnapshot) {
    card_frame().show(ui, |ui| {
        ui.horizontal(|ui| {
            let pct = mem.ram_usage_percent();
            let color = load_color(pct);
            ui.label(
                egui::RichText::new("RAM").monospace().size(10.0).color(colors::TEXT_SECONDARY),
            );
            ui.add(
                egui::ProgressBar::new(pct / 100.0)
                    .fill(color)
                    .desired_width(220.0)
                    .rounding(egui::Rounding::same(3.0))
                    .animate(false),
            );
            ui.label(egui::RichText::new(format!("{pct:.1}%")).monospace().size(13.0).color(color));
            ui.label(
                egui::RichText::new(format!(
                    "{:.2} / {:.2} GiB",
                    mem.ram_used_gib(),
                    mem.ram_total_gib()
                ))
                .monospace()
                .size(11.0)
                .color(colors::TEXT_PRIMARY),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("PROCESSES: {}", mem.processes.len()))
                        .monospace()
                        .size(10.0)
                        .color(colors::ACCENT_CYAN),
                );
            });
        });
    });
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
