//! Dual-series LTTB chart for CPU telemetry (1m / 5m / 15m).
//! Mirrors memory/chart_view.rs architecture 1:1.

use crate::modules::cpu::chart_axis::*;
use crate::modules::cpu::chart_hud::*;
use crate::modules::cpu::history::CpuHistory;
use crate::theme::colors;
use crate::ui::primitives::{cyber_accent, draw_cyber_corner_brackets};
use eframe::egui;
use egui_plot::{HLine, Line, LineStyle, Plot, PlotPoints};
use std::sync::Arc;

const Y_GRID: &[egui_plot::GridMark] = &[
    egui_plot::GridMark { value: 0.0, step_size: 25.0 },
    egui_plot::GridMark { value: 25.0, step_size: 25.0 },
    egui_plot::GridMark { value: 50.0, step_size: 25.0 },
    egui_plot::GridMark { value: 75.0, step_size: 25.0 },
    egui_plot::GridMark { value: 100.0, step_size: 25.0 },
];

/// Renders the CPU dual-series telemetry chart.
/// Time window selector is rendered in page.rs header to avoid duplication.
pub fn render_cpu_chart(ui: &mut egui::Ui, h: &mut CpuHistory) {
    let selected = selected_window(ui.ctx());
    h.chart_window = selected;
    let peak_pct = h.current_hottest_percent();
    let resp = crate::theme::card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            ui.add_space(6.0);

            let now = h.latest_timestamp();
            let span = h.chart_window.seconds() as f64 / 60.0;

            // Get precomputed LTTB-downsampled series (zero allocation on render path)
            let total_series: Arc<Vec<[f64; 2]>> = h.total_series(selected);
            let peak_series: Arc<Vec<[f64; 2]>> = h.hottest_series(selected);

            // Convert timestamps to minutes-ago and anchor to right edge
            let total_raw = anchored(&convert_to_minutes_ago(&total_series, now), span);
            let peak_raw = anchored(&convert_to_minutes_ago(&peak_series, now), span);

            // Handle empty series (startup)
            const ONE_SEC_MIN: f64 = 1.0 / 60.0;
            let total_points = if total_raw.is_empty() {
                PlotPoints::new(vec![[-ONE_SEC_MIN, 0.0], [0.0, 0.0]])
            } else {
                PlotPoints::new(total_raw)
            };
            let peak_points = if peak_raw.is_empty() {
                PlotPoints::new(vec![[-ONE_SEC_MIN, 0.0], [0.0, 0.0]])
            } else {
                PlotPoints::new(peak_raw)
            };

            let plot_height = ui.available_height().max(220.0);
            let plot_width = ui.available_width().max(120.0);

            // Read toggle states BEFORE plot.show()
            let total_id = egui::Id::new("cpu_chart_show_total");
            let peak_id = egui::Id::new("cpu_chart_show_peak");
            let show_total: bool =
                ui.ctx().data_mut(|d| *d.get_temp_mut_or_insert_with(total_id, || true));
            let show_peak: bool =
                ui.ctx().data_mut(|d| *d.get_temp_mut_or_insert_with(peak_id, || true));

            let plot_response = Plot::new("cpu_load_plot")
                .width(plot_width)
                .height(plot_height)
                .show_axes([false, false])
                .show_grid([true, true])
                .auto_bounds(egui::Vec2b::new(false, false))
                .y_grid_spacer(move |_| Y_GRID.to_vec())
                .x_grid_spacer(move |input| x_grid_spacer(span, &input))
                .include_x(-span)
                .include_x(0.0)
                .include_y(0.0)
                .include_y(100.0)
                .allow_scroll(false)
                .allow_zoom(false)
                .allow_drag(false)
                .allow_boxed_zoom(false)
                .label_formatter(|name, value| {
                    let ago = -value.x;
                    let when = if ago.abs() < 0.05 {
                        "now".to_owned()
                    } else {
                        format!("{:.1}m ago", ago)
                    };
                    if name.is_empty() {
                        format!("{}: {:.1}%", when, value.y)
                    } else {
                        format!("{}\n{}: {:.1}%", name, when, value.y)
                    }
                })
                .show(ui, |pu| {
                    pu.set_plot_bounds(egui_plot::PlotBounds::from_min_max(
                        [-span, 0.0],
                        [0.0, 100.0],
                    ));
                    pu.hline(
                        HLine::new(85.0)
                            .color(egui::Color32::from_rgba_unmultiplied(255, 42, 84, 110))
                            .style(LineStyle::Dashed { length: 6.0 }),
                    );

                    if show_total {
                        pu.line(
                            Line::new(total_points)
                                .name("Total CPU %")
                                .color(colors::ACCENT_CYAN)
                                .width(2.2_f32)
                                .fill(0.0_f32),
                        );
                    }
                    if show_peak {
                        pu.line(
                            Line::new(peak_points)
                                .name("Peak Thread %")
                                .color(colors::ACCENT_AMBER)
                                .width(1.8_f32),
                        );
                    }
                });

            // Draw in-canvas HUD axis labels using the plot's frame rect
            let plot_rect = *plot_response.transform.frame();
            draw_y_axis_hud_labels(ui, plot_rect);
            draw_x_axis_hud_labels(ui, plot_rect, selected);

            // Draw interactive horizontal legend on top of the plot
            let legend_rect_id = egui::Id::new("cpu_chart_legend_rect");
            let prev_legend_rect: Option<egui::Rect> =
                ui.ctx().data(|d| d.get_temp(legend_rect_id));
            let interaction_rect = prev_legend_rect.unwrap_or(plot_rect);

            let (new_show_total, new_show_peak) = draw_hud_legend(ui, interaction_rect);

            // Store current legend rect for next frame's interaction
            ui.ctx().data_mut(|d| d.insert_temp(legend_rect_id, plot_rect));

            // Persist toggle states if changed
            if new_show_total != show_total {
                ui.ctx().data_mut(|d| d.insert_temp(total_id, new_show_total));
            }
            if new_show_peak != show_peak {
                ui.ctx().data_mut(|d| d.insert_temp(peak_id, new_show_peak));
            }
        });
    });
    draw_cyber_corner_brackets(ui, resp.response.rect, cyber_accent(peak_pct));
}
