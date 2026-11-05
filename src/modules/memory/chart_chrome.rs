//! Chart header and time-scale row for PULSE memory history.

use crate::i18n::Key;
use crate::modules::memory::chart_view::set_window;
use crate::modules::memory::model::ChartWindow;
use crate::theme::colors;
use eframe::egui;

const WINDOWS: [ChartWindow; 3] =
    [ChartWindow::OneMin, ChartWindow::FiveMin, ChartWindow::FifteenMin];

fn window_button(ui: &mut egui::Ui, window: ChartWindow, active: bool) -> bool {
    let color = if active { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY };
    let text = egui::RichText::new(window.label_en()).monospace().size(11.0).strong().color(color);
    let mut button = egui::Button::new(text).frame(true).min_size(egui::vec2(40.0, 20.0));
    button = if active {
        button.fill(colors::ACCENT_CYAN.gamma_multiply(0.15))
    } else {
        button.fill(colors::BORDER)
    };
    ui.add(button).clicked()
}

/// Draws title, window buttons and the live readout left-to-right so the
/// readout can never be clipped by the card's right edge.
pub(crate) fn render_header(
    ui: &mut egui::Ui,
    selected: &mut ChartWindow,
    ram: f32,
    commit: f32,
    swap: f32,
    has_commit: bool,
) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(Key::MemoryHistoryTitle.t())
                .monospace()
                .size(13.0)
                .color(colors::ACCENT_CYAN),
        );
        ui.add_space(8.0);
        for window in WINDOWS {
            if window_button(ui, window, window == *selected) {
                *selected = window;
                set_window(ui.ctx(), window);
            }
        }
        ui.add_space(12.0);
        let (label, value) = if has_commit { ("Commit", commit) } else { ("Swap", swap) };
        ui.label(
            egui::RichText::new(format!("RAM: {:.1}% | {}: {:.1}%", ram, label, value))
                .monospace()
                .size(11.0)
                .color(colors::TEXT_SECONDARY),
        );
    });
}
