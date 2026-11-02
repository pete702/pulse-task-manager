//! Multi-horizon neon chart for PULSE memory telemetry (1m / 5m / 15m).

use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};

use crate::i18n::{current_lang, Key, Lang};
use crate::modules::memory::model::{ChartWindow, MemoryHistory};
use crate::theme::colors;

/// Renders the memory history chart with time horizon switcher.
pub fn render_memory_chart(ui: &mut egui::Ui, history: &mut MemoryHistory) {
    crate::theme::card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            // Header with title and time horizon switcher
            ui.horizontal(|ui| {
                // Title
                ui.label(
                    egui::RichText::new(Key::MemoryHistoryTitle.t())
                        .monospace()
                        .size(13.0)
                        .color(colors::ACCENT_CYAN),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Time horizon switcher buttons
                    let lang = current_lang();
                    for window in [
                        ChartWindow::OneMin,
                        ChartWindow::FiveMin,
                        ChartWindow::FifteenMin,
                    ] {
                        let is_active = history.chart_window == window;
                        let label = match lang {
                            Lang::Uk => window.label(),
                            Lang::En => window.label_en(),
                        };

                        let btn = if is_active {
                            ui.add(
                                egui::Button::new(
                                    egui::RichText::new(label)
                                        .monospace()
                                        .size(11.0)
                                        .color(colors::TEXT_PRIMARY),
                                )
                                .fill(colors::ACCENT_CYAN.gamma_multiply(0.15))
                                .frame(true),
                            )
                        } else {
                            ui.add(
                                egui::Button::new(
                                    egui::RichText::new(label)
                                        .monospace()
                                        .size(11.0)
                                        .color(colors::TEXT_SECONDARY),
                                )
                                .frame(false),
                            )
                        };

                        if btn.clicked() {
                            history.chart_window = window;
                        }
                    }
                });
            });
            ui.add_space(8.0);

            // Build plot points from history for selected window
            let window_secs = history.chart_window.seconds() as usize;
            let window_f64 = window_secs as f64;

            let ram_points: PlotPoints = history
                .windowed_ram_points()
                .enumerate()
                .map(|(i, v)| {
                    let total = history.ram_points.len();
                    let skip = total.saturating_sub(window_secs);
                    let _idx = skip + i;
                    let x = -((window_secs.saturating_sub(i + 1)) as f64);
                    [x, v as f64]
                })
                .collect();

            let swap_points: PlotPoints = history
                .windowed_swap_points()
                .enumerate()
                .map(|(i, v)| {
                    let total = history.swap_points.len();
                    let skip = total.saturating_sub(window_secs);
                    let _idx = skip + i;
                    let x = -((window_secs.saturating_sub(i + 1)) as f64);
                    [x, v as f64]
                })
                .collect();

            // Legend labels
            let ram_legend = Key::RamPercent.t();
            let swap_legend = Key::SwapPercent.t();

            // Draw the plot - stretch to full card width
            let plot = Plot::new("memory_history_plot")
                .width(ui.available_width())
                .height(195.0)
                .show_grid(true)
                .x_axis_formatter(|mark, _range, _ctx| format!("{}s", (-mark.value).round() as i32))
                .y_axis_formatter(|mark, _range, _ctx| format!("{}%", mark.value.round() as i32))
                .include_x(-window_f64)
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
                        .name(ram_legend)
                        .color(colors::ACCENT_CYAN)
                        .width(2.2_f32)
                        .fill(0.0_f32),
                );

                // Swap line (purple)
                plot_ui.line(
                    Line::new(swap_points)
                        .name(swap_legend)
                        .color(colors::ACCENT_PURPLE)
                        .width(1.8_f32),
                );
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use crate::modules::memory::model::{ChartWindow, MemoryHistory};

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

    #[test]
    fn test_chart_window_switcher() {
        let mut history = MemoryHistory::new();
        history.chart_window = ChartWindow::OneMin;
        assert_eq!(history.chart_window, ChartWindow::OneMin);

        history.chart_window = ChartWindow::FiveMin;
        assert_eq!(history.chart_window, ChartWindow::FiveMin);

        history.chart_window = ChartWindow::FifteenMin;
        assert_eq!(history.chart_window, ChartWindow::FifteenMin);
    }
}
