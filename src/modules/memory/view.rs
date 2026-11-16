//! Memory HUD cards for PULSE -- 2-level cockpit matching the Processor tab.
use crate::modules::memory::chart_view;
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::model::MemorySnapshot;
use crate::modules::memory::top_cards::render_memory_top_cards;
use eframe::egui;

pub fn render_memory_cards(ui: &mut egui::Ui, mem: &MemorySnapshot, history: &mut MemoryHistory) {
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        render_memory_top_cards(ui, mem, history);
        ui.add_space(10.0);
        chart_view::render_memory_chart(ui, history, mem);
    });
}
