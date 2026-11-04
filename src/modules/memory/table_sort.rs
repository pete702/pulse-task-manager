use super::table_state::MemoryTableState;
use crate::modules::memory::process_model::MemorySortCol;
use crate::theme::colors;
use eframe::egui;

pub(super) fn sort_header(ui: &mut egui::Ui, text: &str, active: bool, asc: bool) -> bool {
    let label = if active {
        format!("{text} {}", if asc { "▲" } else { "▼" })
    } else {
        text.to_owned()
    };
    ui.add(
        egui::Button::new(egui::RichText::new(label).monospace().size(10.0).color(if active {
            colors::ACCENT_CYAN
        } else {
            colors::TEXT_SECONDARY
        }))
        .frame(false),
    )
    .clicked()
}

pub fn toggle_sort(state: &mut MemoryTableState, col: MemorySortCol) {
    if state.sort_col == col {
        state.sort_asc = !state.sort_asc;
    } else {
        state.sort_col = col;
        state.sort_asc = matches!(col, MemorySortCol::Name | MemorySortCol::Pid);
    }
}
