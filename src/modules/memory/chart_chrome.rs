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
/// Also renders the Memory Spike Blackbox indicator on the right.
pub(crate) fn render_header(
    ui: &mut egui::Ui,
    selected: &mut ChartWindow,
    ram: f32,
    commit: f32,
    swap: f32,
    has_commit: bool,
    latest_spike: Option<&str>,
) {
    let narrow = ui.available_width() < 960.0;
    ui.horizontal_wrapped(|ui| {
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
        if !narrow {
            ui.add_space(12.0);
            let (label, value) = if has_commit { ("Commit", commit) } else { ("Swap", swap) };
            ui.add(
                egui::Label::new(
                    egui::RichText::new(format!("RAM: {ram:.1}% | {label}: {value:.1}%"))
                        .monospace()
                        .size(11.0)
                        .color(colors::TEXT_SECONDARY),
                )
                .wrap(false),
            );
        }

        // Blackbox badge on the right - never clipped when wrapped.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(spike) = latest_spike {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(format!("[ ! SPIKE: {spike} ]"))
                            .monospace()
                            .size(10.5)
                            .color(colors::ACCENT_RED),
                    )
                    .wrap(false),
                );
            } else {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new("[ BLACKBOX: ARMED ]")
                            .monospace()
                            .size(10.5)
                            .color(egui::Color32::from_rgb(0x00, 0xE0, 0x80)),
                    )
                    .wrap(false),
                );
            }
        });
    });
}
