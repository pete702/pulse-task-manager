//! Mode badge and selection/status readout for the memory action bar.

use crate::modules::memory::action_bar::{MemoryActionBarState, NO_SELECTION_HINT, PROTECTED_HINT};
use crate::modules::memory::safe_handle::is_app_elevated;
use crate::modules::memory::table_state::SelectedGroupSummary;
use crate::theme::{colors, TEXT_MUTED};
use eframe::egui;
use std::sync::OnceLock;
use std::time::Instant;

fn elevated() -> bool {
    static CACHE: OnceLock<bool> = OnceLock::new();
    *CACHE.get_or_init(is_app_elevated)
}

/// Hover text for disabled selection actions.
pub(super) fn hint(sel: &Option<SelectedGroupSummary>, busy: bool) -> &'static str {
    if busy {
        crate::modules::memory::action_bar::BUSY_HINT
    } else if sel.is_some() {
        PROTECTED_HINT
    } else {
        NO_SELECTION_HINT
    }
}

pub(super) fn mode_label(ui: &mut egui::Ui) {
    let (text, color) = if elevated() {
        ("[ ADMIN MODE ]", colors::ACCENT_CYAN)
    } else {
        ("[ USER MODE ]", TEXT_MUTED)
    };
    ui.label(egui::RichText::new(text).monospace().size(10.0).color(color))
        .on_hover_text("Elevation decides which processes may be trimmed or ended");
}

pub(super) fn selection_info(
    ui: &mut egui::Ui,
    sel: &Option<SelectedGroupSummary>,
    st: &MemoryActionBarState,
    now: Instant,
) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| match sel {
        Some(v) => {
            ui.label(
                egui::RichText::new(format!("Selected: {}", v.name))
                    .monospace()
                    .size(11.0)
                    .color(colors::ACCENT_CYAN),
            );
            if v.blocked {
                ui.label(
                    egui::RichText::new("[ 🛡️ OS PROTECTED ]")
                        .monospace()
                        .size(10.0)
                        .color(colors::ACCENT_RED),
                )
                .on_hover_text(PROTECTED_HINT);
            } else {
                ui.label(
                    egui::RichText::new(format!("{} raw PID(s)", v.pids.len()))
                        .monospace()
                        .size(10.0)
                        .color(TEXT_MUTED),
                );
            }
        }
        None => {
            let (text, color) = match st.status_text(now) {
                Some(t) => (t.to_owned(), colors::TEXT_SECONDARY),
                None => (NO_SELECTION_HINT.to_owned(), TEXT_MUTED),
            };
            ui.label(egui::RichText::new(text).monospace().size(10.0).color(color));
        }
    });
}
