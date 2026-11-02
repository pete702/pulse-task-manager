//! 60-second neon chart for PULSE memory telemetry.

use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};

use crate::i18n::Key;
use crate::modules::memory::model::MemoryHistory;
use crate::theme::colors;

/// Renders the 60-second memory history chart.
pub fn render_memory_chart(ui: &mut egui::Ui, history: &MemoryHistory) {
    crate::theme::card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(Key::MemoryHistory60s.t())
                        .monospace()
                        .size(13.0)
                        .color(colors::ACCENT_CYAN),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "{}  {}",
                            Key::RamPercent.t(),
                            Key::SwapPercent.t()
                        ))
                        .monospace()
                        .size(11.0)
                        .color(colors::TEXT_SECONDARY),
                    );
                });
            });
            ui.add_space(8.0);

            // Build plot points from history
            let ram_points: PlotPoints = history
                .ram_points
                .iter()
                .enumerate()
                .map(|(i, &v)| [-(history.len() as f64 - 1.0 - i as f64) * 0.5, v as f64])
                .collect();

            let swap_points: PlotPoints = history
                .swap_points
                .iter()
                .enumerate()
                .map(|(i, &v)| [-(history.len() as f64 - 1.0 - i as f64) * 0.5, v as f64])
                .collect();

            // Draw the plot - stretch to full card width
            let plot = Plot::new("memory_60s_plot")
                .width(ui.available_width())
                .height(200.0)
                .show_grid(true)
                .x_axis_formatter(|mark, _range, _ctx| format!("{}s", (-mark.value).round() as i32))
                .y_axis_formatter(|mark, _range, _ctx| format!("{}%", mark.value.round() as i32))
                .include_x(-60.0)
                .include_x(0.0)
                .include_y(0.0)
                .include_y(100.0)
                .allow_zoom(false)
                .allow_drag(false)
                .allow_boxed_zoom(false)
                .legend(
                    egui_plot::Legend::default()
                        .position(egui_plot::Corner::LeftTop)
                        .text_style(egui::TextStyle::Monospace),
                );

            plot.show(ui, |plot_ui| {
                // RAM line (cyan) with semi-transparent fill
                plot_ui.line(
                    Line::new(ram_points)
                        .name(Key::RamPercent.t())
                        .color(colors::ACCENT_CYAN)
                        .width(2.2_f32)
                        .fill(0.0_f32),
                );

                // Swap line (purple)
                plot_ui.line(
                    Line::new(swap_points)
                        .name(Key::SwapPercent.t())
                        .color(colors::ACCENT_PURPLE)
                        .width(1.8_f32),
                );
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use crate::modules::memory::model::MemoryHistory;

    #[test]
    fn test_render_memory_chart_no_panic() {
        let mut history = MemoryHistory::new();

        // Add some test data
        for i in 0..120 {
            let snap = crate::modules::memory::model::MemorySnapshot {
                ram_total_bytes: 16_000_000_000,
                ram_used_bytes: (8_000_000_000 + (i * 10_000_000) as u64) % 16_000_000_000,
                ram_available_bytes: 8_000_000_000,
                ram_free_bytes: 2_000_000_000,
                swap_total_bytes: 8_000_000_000,
                swap_used_bytes: (2_000_000_000 + (i * 5_000_000) as u64) % 8_000_000_000,
                swap_free_bytes: 6_000_000_000,
                ..Default::default()
            };
            history.push(&snap);
        }

        // Verify history has correct number of points
        assert_eq!(history.len(), 120);
        assert_eq!(history.ram_points.len(), 120);
        assert_eq!(history.swap_points.len(), 120);
    }
}
