//! Memory HUD card rendering (RAM card + Swap/Commit card).

use eframe::egui;

use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::model::MemorySnapshot;
use crate::modules::memory::ram_card::render_ram_card;
use crate::modules::memory::swap_commit_card::render_swap_commit_card;

/// Renders both memory cards (RAM + Swap/Commit) side by side.
pub fn render_memory_cards_row(ui: &mut egui::Ui, mem: &MemorySnapshot, history: &MemoryHistory) {
    ui.columns(2, |cols| {
        render_ram_card(&mut cols[0], mem, history);
        render_swap_commit_card(&mut cols[1], mem);
    });
}
