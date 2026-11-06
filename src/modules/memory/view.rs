//! Memory HUD cards for PULSE — 2-level cockpit matching the Processor tab.
use crate::modules::memory::chart_view;
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::model::MemorySnapshot;
use crate::modules::memory::top_cards::render_memory_top_cards;
use eframe::egui;

/// Renders the memory cockpit in two levels:
///   Row 1: two symmetric top cards (Physical RAM + Commit Charge / Kernel Pools)
///   Row 2: full-height Memory Utilization History chart
/// No vertical scrollbar: the chart claims the remaining window height.
pub fn render_memory_cards(ui: &mut egui::Ui, mem: &MemorySnapshot, history: &mut MemoryHistory) {
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        render_memory_top_cards(ui, mem, history);
        ui.add_space(10.0);
        let chart_height = ui.available_height().max(220.0);
        chart_view::render_memory_chart(ui, history, mem, chart_height);
    });
}

/// True free physical memory: available memory minus the reclaimable system cache.
///
/// `ram_available_bytes` includes both truly free pages and system cache, so
/// subtracting the cached portion yields the genuinely unused RAM.
#[allow(dead_code)]
#[inline]
pub fn true_free_bytes(mem: &MemorySnapshot) -> u64 {
    mem.ram_available_bytes.saturating_sub(mem.ram_cached_bytes())
}
