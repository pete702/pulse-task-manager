//! Multi-horizon neon chart for PULSE memory telemetry (1m / 5m / 15m).
use crate::i18n::Key;
use crate::modules::memory::chart_axis::{minutes_ago, window_minutes};
use crate::modules::memory::chart_chrome::render_header;
use crate::modules::memory::chart_hud::{
    draw_hud_legend, draw_x_axis_hud_labels, draw_y_axis_hud_labels,
};
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::model::{ChartWindow, MemorySnapshot};
use crate::theme::colors;
use crate::ui::primitives::{cyber_accent, draw_cyber_corner_brackets};
use eframe::egui;
use egui_plot::{GridInput, GridMark, HLine, Line, LineStyle, Plot, PlotBounds, PlotPoints};
use std::sync::Arc;

const Y_GRID: &[GridMark] = &[
    GridMark { value: 0.0, step_size: 25.0 },
    GridMark { value: 25.0, step_size: 25.0 },
    GridMark { value: 50.0, step_size: 25.0 },
    GridMark { value: 75.0, step_size: 25.0 },
    GridMark { value: 100.0, step_size: 25.0 },
];

/// Custom X-axis grid spacer for consistent major/minor grid lines across all windows.
/// X coordinates are in minutes (negative = minutes ago, 0 = now).
/// Returns combined grid marks for the given window span.
fn x_grid_spacer(span_min: f64, _input: &GridInput) -> Vec<GridMark> {
    let (major_step, minor_step) = if span_min <= 1.0 {
        // 1m window: major every 15s (0.25 min), minor every 5s (1/12 min)
        (0.25, 1.0 / 12.0)
    } else if span_min <= 5.0 {
        // 5m window: major every 1m, minor every 15s (0.25 min)
        (1.0, 0.25)
    } else {
        // 15m window: major every 3m, minor every 1m
        (3.0, 1.0)
    };

    let mut marks = Vec::new();
    // Major marks
    for i in 0..=(span_min / major_step).ceil() as i32 {
        let val = -i as f64 * major_step;
        marks.push(GridMark { value: val, step_size: major_step });
    }
    // Minor marks
    for i in 0..=(span_min / minor_step).ceil() as i32 {
        let val = -i as f64 * minor_step;
        marks.push(GridMark { value: val, step_size: minor_step });
    }
    marks
}

fn window_id() -> egui::Id {
    egui::Id::new("pulse_memory_chart_window")
}

pub fn selected_window(ctx: &egui::Context) -> ChartWindow {
    let mut window = ChartWindow::FiveMin;
    ctx.data_mut(|d| window = *d.get_temp_mut_or_insert_with(window_id(), || ChartWindow::FiveMin));
    window
}

pub fn set_window(ctx: &egui::Context, window: ChartWindow) {
    ctx.data_mut(|d| d.insert_temp(window_id(), window));
}

/// Anchors a series to the right edge (now = 0.0).
/// For a single point (startup), draw a 1-second lead-in [-1/60, 0].
/// For 2+ points, only anchor the right edge to 0.0 (if last_x < 0).
/// The left edge grows naturally leftward as history accumulates.
/// X coordinates are in MINUTES (see chart_axis::minutes_ago).
const ONE_SEC_MIN: f64 = 1.0 / 60.0;

fn anchored(points: &[[f64; 2]], _span: f64) -> Vec<[f64; 2]> {
    if points.is_empty() {
        return vec![];
    }
    if points.len() == 1 {
        let y = points[0][1];
        return vec![[-ONE_SEC_MIN, y], [0.0, y]];
    }
    // Only anchor right edge to 0.0; left edge grows naturally.
    let mut out = points.to_vec();
    if let Some(last) = out.last().copied() {
        if last[0] < 0.0 {
            out.push([0.0, last[1]]);
        }
    }
    out
}

fn convert_to_minutes_ago(points: &[[f64; 2]], now: f64) -> Vec<[f64; 2]> {
    points.iter().map(|p| [minutes_ago(p[0], now), p[1]]).collect()
}

pub fn render_memory_chart(
    ui: &mut egui::Ui,
    h: &mut MemoryHistory,
    s: &MemorySnapshot,
    chart_height: f32,
) {
    let mut selected = selected_window(ui.ctx());
    let _ = chart_height; // Keep signature; we compute actual height after header
    let ram_pct = h.current_ram_percent();
    let resp = crate::theme::card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            let ram = h.current_ram_percent();
            let commit = s.commit_usage_percent();
            let swap = s.swap_usage_percent();
            let has_commit = s.kernel_metrics_valid && s.commit_limit_bytes > 0;
            // Format spike for blackbox indicator in header
            let spike_text = s.latest_spike.as_ref().map(|e| {
                format!(
                    "{} @ {:.0}% ({} MB)",
                    e.culprit_name,
                    e.peak_ram_pct,
                    e.culprit_bytes / 1024 / 1024
                )
            });
            // Pass latest_spike to header for blackbox indicator
            render_header(ui, &mut selected, ram, commit, swap, has_commit, spike_text.as_deref());
            ui.add_space(6.0);
            h.chart_window = selected;

            let now = h.latest_timestamp();
            let span = window_minutes(selected);

            // Get precomputed LTTB-downsampled series (zero allocation on render path)
            let ram_series: Arc<Vec<[f64; 2]>> = h.ram_series(selected);
            let purple_series: Arc<Vec<[f64; 2]>> =
                if has_commit { h.commit_series(selected) } else { h.swap_series(selected) };

            // Convert timestamps to minutes-ago and anchor to right edge
            let ram_raw = anchored(&convert_to_minutes_ago(&ram_series, now), span);
            let purple_raw = anchored(&convert_to_minutes_ago(&purple_series, now), span);

            // Handle empty series (startup)
            let ram_points = if ram_raw.is_empty() {
                PlotPoints::new(vec![[-ONE_SEC_MIN, ram as f64], [0.0, ram as f64]])
            } else {
                PlotPoints::new(ram_raw)
            };
            let purple_points = if purple_raw.is_empty() {
                let flat = if has_commit { commit as f64 } else { swap as f64 };
                PlotPoints::new(vec![[-ONE_SEC_MIN, flat], [0.0, flat]])
            } else {
                PlotPoints::new(purple_raw)
            };

            // Compute plot height AFTER header so bottom labels fit.
            let plot_height = ui.available_height().max(220.0);
            let plot_width = ui.available_width().max(120.0);

            // Read toggle states BEFORE plot.show() for conditional series rendering
            let ram_id = egui::Id::new("mem_chart_show_ram");
            let commit_id = egui::Id::new("mem_chart_show_commit");
            let show_ram: bool =
                ui.ctx().data_mut(|d| *d.get_temp_mut_or_insert_with(ram_id, || true));
            let show_commit: bool =
                ui.ctx().data_mut(|d| *d.get_temp_mut_or_insert_with(commit_id, || true));

            // Full-width, full-height plot inside the card - flush with MEMORY COMPOSITION bar
            let plot_response = Plot::new("memory_load_plot")
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
                    pu.set_plot_bounds(PlotBounds::from_min_max([-span, 0.0], [0.0, 100.0]));
                    pu.hline(
                        HLine::new(85.0)
                            .color(egui::Color32::from_rgba_unmultiplied(255, 42, 84, 110))
                            .style(LineStyle::Dashed { length: 6.0 }),
                    );

                    if show_ram {
                        pu.line(
                            Line::new(ram_points)
                                .name(Key::RamPercent.t())
                                .color(colors::ACCENT_CYAN)
                                .width(2.2_f32)
                                .fill(0.0_f32),
                        );
                    }
                    if show_commit {
                        pu.line(
                            Line::new(purple_points)
                                .name(if has_commit { "Commit %" } else { "Swap %" })
                                .color(colors::ACCENT_PURPLE)
                                .width(1.8_f32),
                        );
                    }
                });

            // Draw in-canvas HUD axis labels using the plot's frame rect
            let plot_rect = *plot_response.transform.frame();
            draw_y_axis_hud_labels(ui, plot_rect);
            draw_x_axis_hud_labels(ui, plot_rect, selected);

            // Draw interactive horizontal legend on top of the plot (using actual plot rect)
            // Use previous frame's legend rect for interaction (stored in ctx.data), fallback to current plot_rect
            let legend_rect_id = egui::Id::new("mem_chart_legend_rect");
            let prev_legend_rect: Option<egui::Rect> =
                ui.ctx().data(|d| d.get_temp(legend_rect_id));
            let interaction_rect = prev_legend_rect.unwrap_or(plot_rect);

            let (new_show_ram, new_show_commit) = draw_hud_legend(ui, interaction_rect, has_commit);

            // Store current legend rect for next frame's interaction
            ui.ctx().data_mut(|d| d.insert_temp(legend_rect_id, plot_rect));

            // Persist toggle states if changed
            if new_show_ram != show_ram {
                ui.ctx().data_mut(|d| d.insert_temp(ram_id, new_show_ram));
            }
            if new_show_commit != show_commit {
                ui.ctx().data_mut(|d| d.insert_temp(commit_id, new_show_commit));
            }
        });
    });
    draw_cyber_corner_brackets(ui, resp.response.rect, cyber_accent(ram_pct));
}
