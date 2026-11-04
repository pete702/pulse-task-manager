//! Memory HUD cards for PULSE.
use crate::modules::memory::chart_view;
use crate::modules::memory::composition_bar::render_composition_bar;
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::kernel_tiles::render_kernel_tiles;
use crate::modules::memory::model::MemorySnapshot;
use crate::modules::memory::view_cards::render_memory_cards_row;
use eframe::egui;

/// Renders the memory HUD cards, mini-tiles, composition bar, and chart.
pub fn render_memory_cards(ui: &mut egui::Ui, mem: &MemorySnapshot, history: &mut MemoryHistory) {
    // ----- Row 1: Two main cards (RAM + Swap/Commit) -----
    render_memory_cards_row(ui, mem, history);

    ui.add_space(6.0);

    // ----- Row 2: 4 Mini-tiles (Kernel Metrics) -----
    render_kernel_tiles(ui, mem);

    ui.add_space(6.0);

    // ----- Row 3: RAMMap 3-Segment Composition Bar -----
    render_composition_bar(ui, mem);

    ui.add_space(6.0);

    // ----- Row 4: History Chart -----
    chart_view::render_memory_chart(ui, history, mem);
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
