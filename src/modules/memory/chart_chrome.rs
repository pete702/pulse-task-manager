//! Chart header and time-scale row for PULSE memory history.

use crate::i18n::Key;
use crate::modules::memory::chart_view::set_window;
use crate::modules::memory::model::ChartWindow;
use crate::theme::colors;
use crate::ui::primitives::pill_button_colored;
use eframe::egui;

const WINDOWS: [ChartWindow; 3] =
    [ChartWindow::OneMin, ChartWindow::FiveMin, ChartWindow::FifteenMin];

/// Soft chip with JetBrains Mono numbers for the live RAM/commit readout.
fn mono_badge(ui: &mut egui::Ui, text: &str, fg: egui::Color32) {
    egui::Frame::none()
        .fill(fg.gamma_multiply(0.13))
        .stroke(egui::Stroke::NONE)
        .rounding(egui::Rounding::same(5.0))
        .inner_margin(egui::Margin::symmetric(8.0, 3.0))
        .show(ui, |ui| {
            let rich = egui::RichText::new(text).font(egui::FontId::monospace(11.5)).color(fg);
            ui.add(egui::Label::new(rich).wrap(false));
        });
}

/// Single clean header row: title, window pills, right-side readout and spike note.
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
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Memory Utilization History")
                .font(egui::FontId::proportional(13.0))
                .color(colors::ACCENT_CYAN),
        )
        .on_hover_text(Key::MemoryHistoryTitle.t());
        ui.add_space(8.0);
        for window in WINDOWS {
            let col =
                if window == *selected { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY };
            if pill_button_colored(ui, window.label_en(), col).clicked() {
                *selected = window;
                set_window(ui.ctx(), window);
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(spike) = latest_spike {
                mono_badge(ui, spike, colors::ACCENT_RED);
            } else {
                mono_badge(ui, "BLACKBOX: ARMED (85% trigger)", colors::EMERALD);
            }
            if !narrow {
                let (label, value) = if has_commit { ("COMMIT", commit) } else { ("SWAP", swap) };
                mono_badge(
                    ui,
                    &format!("RAM {ram:.1}% | {label} {value:.1}%"),
                    colors::TEXT_SECONDARY,
                );
            }
        });
    });
}
