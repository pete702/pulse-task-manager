//! Memory HUD cards for PULSE.

use eframe::egui;

use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::kernel_tiles::render_kernel_tiles;
use crate::modules::memory::model::MemorySnapshot;
use crate::modules::memory::view_cards::render_memory_cards_row;

/// Renders the memory HUD cards and mini-tiles.
pub fn render_memory_cards(ui: &mut egui::Ui, mem: &MemorySnapshot, history: &MemoryHistory) {
    // ----- Row 1: Two main cards (RAM + Swap/Commit) -----
    render_memory_cards_row(ui, mem, history);

    ui.add_space(12.0);

    // ----- Row 2: 4 Mini-tiles (Kernel Metrics) -----
    render_kernel_tiles(ui, mem);
}
