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

/// Draws the custom monospace X-axis scale below the plot.
///
/// Labels are painted at exact fractions of the full width so the first and
/// last label (`0m` / `0s`) always stay inside the card instead of being
/// pushed past the right edge by accumulated widget spacing.
pub(crate) fn render_time_scale(ui: &mut egui::Ui, span_minutes: f64, plot_frame: egui::Rect) {
    ui.add_space(2.0);
    let labels: &[&str] = if span_minutes == 1.0 {
        &["-60s", "-45s", "-30s", "-15s", "0s"]
    } else if span_minutes == 5.0 {
        &["-5m", "-4m", "-3m", "-2m", "-1m", "0m"]
    } else {
        &["-15m", "-12m", "-9m", "-6m", "-3m", "0m"]
    };
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 14.0), egui::Sense::hover());
    let count = labels.len();
    for (index, label) in labels.iter().enumerate() {
        let fraction = index as f32 / (count - 1) as f32;
        let x = plot_frame.left() + fraction * plot_frame.width();
        let align = if index == 0 {
            egui::Align2::LEFT_CENTER
        } else if index + 1 == count {
            egui::Align2::RIGHT_CENTER
        } else {
            egui::Align2::CENTER_CENTER
        };
        ui.painter().text(
            egui::pos2(x, rect.center().y),
            align,
            *label,
            egui::FontId::monospace(10.0),
            colors::TEXT_SECONDARY,
        );
    }
}
