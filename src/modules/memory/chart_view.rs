//! Multi-horizon neon chart for PULSE memory telemetry (1m / 5m / 15m).
use crate::i18n::Key;
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::model::ChartWindow;
use crate::theme::colors;
use eframe::egui;
use egui_plot::{HLine, Line, LineStyle, Plot, PlotPoints};

/// Renders the memory history chart with time horizon switcher.
pub fn render_memory_chart(ui: &mut egui::Ui, history: &mut MemoryHistory) {
    crate::theme::card_frame()
        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
        .outer_margin(egui::Margin {
            left: 0.0,
            right: 4.0,
            top: 0.0,
            bottom: 0.0,
        })
        .show(ui, |ui| {
            let inner_width = (ui.available_width() - 16.0).max(100.0);
            ui.set_max_width(inner_width);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(Key::MemoryHistoryTitle.t())
                            .monospace()
                            .size(13.0)
                            .color(colors::ACCENT_CYAN),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        for window in [
                            ChartWindow::OneMin,
                            ChartWindow::FiveMin,
                            ChartWindow::FifteenMin,
                        ] {
                            let is_active = history.chart_window == window;
                            let label = window.label_en();
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
                ui.add_space(6.0);
                let window_secs = history.chart_window.seconds() as usize;
                let window_f64 = window_secs as f64;
                let ram_points: PlotPoints = history
                    .windowed_ram_points()
                    .enumerate()
                    .map(|(i, v)| {
                        let _total = history.ram_points.len();
                        let _skip = _total.saturating_sub(window_secs);
                        let x = -((window_secs.saturating_sub(i + 1)) as f64);
                        [x, v as f64]
                    })
                    .collect();
                let swap_points: PlotPoints = history
                    .windowed_swap_points()
                    .enumerate()
                    .map(|(i, v)| {
                        let _total = history.swap_points.len();
                        let _skip = _total.saturating_sub(window_secs);
                        let x = -((window_secs.saturating_sub(i + 1)) as f64);
                        [x, v as f64]
                    })
                    .collect();
                let ram_legend = Key::RamPercent.t();
                let swap_legend = Key::SwapPercent.t();
                let plot_height = (ui.available_height() - 24.0).clamp(75.0, 165.0);
                let plot = Plot::new("memory_history_plot")
                    .width(inner_width)
                    .height(plot_height)
                    .show_grid(true)
                    .x_axis_formatter(|mark, _range, _ctx| {
                        format!("{}s", (-mark.value).round() as i32)
                    })
                    .y_axis_formatter(|mark, _range, _ctx| {
                        format!("{}%", mark.value.round() as i32)
                    })
                    .include_x(-window_f64)
                    .include_x(0.0)
                    .include_y(0.0)
                    .include_y(100.0)
                    .allow_scroll(false)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_boxed_zoom(false)
                    .label_formatter(|name, value| {
                        let secs_ago = value.x.abs().round() as u32;
                        if name.is_empty() {
                            format!("{secs_ago}s ago: {:.1}%", value.y)
                        } else {
                            format!("{name}\n{secs_ago}s ago: {:.1}%", value.y)
                        }
                    })
                    .legend(
                        egui_plot::Legend::default()
                            .position(egui_plot::Corner::LeftTop)
                            .text_style(egui::TextStyle::Monospace),
                    );
                plot.show(ui, |plot_ui| {
                    plot_ui.hline(
                        HLine::new(85.0)
                            .color(egui::Color32::from_rgba_unmultiplied(255, 42, 84, 110))
                            .style(LineStyle::Dashed { length: 6.0 }),
                    );
                    plot_ui.line(
                        Line::new(ram_points)
                            .name(ram_legend)
                            .color(colors::ACCENT_CYAN)
                            .width(2.2_f32)
                            .fill(0.0_f32),
                    );
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
    use crate::modules::memory::history::MemoryHistory;
    use crate::modules::memory::model::ChartWindow;
    #[test]
    fn test_render_memory_chart_no_panic() {
        let mut h = MemoryHistory::new();
        for i in 0..120 {
            let s = crate::modules::memory::model::MemorySnapshot {
                ram_total_bytes: 16_000_000_000,
                ram_used_bytes: (8_000_000_000 + (i * 10_000_000) as u64) % 16_000_000_000,
                ram_available_bytes: 8_000_000_000,
                ram_free_bytes: 2_000_000_000,
                swap_total_bytes: 8_000_000_000,
                swap_used_bytes: (2_000_000_000 + (i * 5_000_000) as u64) % 8_000_000_000,
                swap_free_bytes: 6_000_000_000,
                ..Default::default()
            };
            h.push(&s);
        }
        assert_eq!(h.len(), 120);
        assert_eq!(h.ram_points.len(), 120);
        assert_eq!(h.swap_points.len(), 120);
    }
    #[test]
    fn test_chart_window_switcher() {
        let mut h = MemoryHistory::new();
        h.chart_window = ChartWindow::OneMin;
        assert_eq!(h.chart_window, ChartWindow::OneMin);
        h.chart_window = ChartWindow::FiveMin;
        assert_eq!(h.chart_window, ChartWindow::FiveMin);
        h.chart_window = ChartWindow::FifteenMin;
        assert_eq!(h.chart_window, ChartWindow::FifteenMin);
    }
}
