//! Multi-horizon neon chart for PULSE memory telemetry (1m / 5m / 15m).
use crate::i18n::Key;
use crate::modules::memory::chart_axis::{window_minutes, window_seconds};
use crate::modules::memory::chart_chrome::render_header;
use crate::modules::memory::chart_hud::{
    draw_hud_legend, draw_hud_scanner, draw_x_axis_hud_labels, draw_y_axis_hud_labels,
};
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::model::{ChartWindow, MemorySnapshot};
use crate::theme::colors;
use crate::ui::chart_scale::{
    anchored, convert_to_minutes_ago, lock_time_chart_bounds, x_grid_spacer, ONE_SEC_MIN,
    PERCENT_Y_GRID,
};
use crate::ui::primitives::{cyber_accent, draw_cyber_corner_brackets};
use eframe::egui;
use egui_plot::{HLine, Line, LineStyle, Plot, PlotPoints};
use std::sync::Arc;

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

pub fn render_memory_chart(
    ui: &mut egui::Ui,
    h: &mut MemoryHistory,
    s: &MemorySnapshot,
    chart_height: f32,
) {
    let mut selected = selected_window(ui.ctx());
    let _ = chart_height; // Keep signature; we compute actual height after header
    let ram_pct = h.current_ram_percent();
    let resp = crate::ui::chart_kit::chart_frame().show(ui, |ui| {
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
            let ram_raw = anchored(convert_to_minutes_ago(&ram_series, now));
            let purple_raw = anchored(convert_to_minutes_ago(&purple_series, now));

            // Handle empty series (startup)
            // Auto-trim status badge: "ARMED (85% trigger)" while waiting for threshold.
            let ram_points = if ram_raw.is_empty() {
                let mut pts = Vec::with_capacity(2);
                pts.extend([[-ONE_SEC_MIN, ram as f64], [0.0, ram as f64]]);
                PlotPoints::new(pts)
            } else {
                let mut pts = Vec::with_capacity(ram_raw.len());
                pts.extend(ram_raw.clone());
                PlotPoints::new(pts)
            };
            let purple_points = if purple_raw.is_empty() {
                let flat = if has_commit { commit as f64 } else { swap as f64 };
                let mut pts = Vec::with_capacity(2);
                pts.extend([[-ONE_SEC_MIN, flat], [0.0, flat]]);
                PlotPoints::new(pts)
            } else {
                let mut pts = Vec::with_capacity(purple_raw.len());
                pts.extend(purple_raw.clone());
                PlotPoints::new(pts)
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
                .show_x(false)
                .show_y(false)
                .auto_bounds(egui::Vec2b::new(false, false))
                .y_grid_spacer(move |_| PERCENT_Y_GRID.to_vec())
                .x_grid_spacer(move |input| x_grid_spacer(span, &input))
                .include_x(-span)
                .include_x(0.0)
                .include_y(crate::ui::chart_kit::baseline_y())
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
                    lock_time_chart_bounds(pu, window_seconds(selected), 100.0);
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
            if plot_response.response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
            }
            draw_y_axis_hud_labels(ui, plot_rect);
            draw_x_axis_hud_labels(ui, plot_rect, selected);

            // Draw interactive horizontal legend on top of the plot (using actual plot rect)
            // Use previous frame's legend rect for interaction (stored in ctx.data), fallback to current plot_rect
            let legend_rect_id = egui::Id::new("mem_chart_legend_rect");
            let prev_legend_rect: Option<egui::Rect> =
                ui.ctx().data(|d| d.get_temp(legend_rect_id));
            let interaction_rect = prev_legend_rect.unwrap_or(plot_rect);

            draw_hud_scanner(
                ui,
                plot_rect,
                span,
                &ram_raw,
                &purple_raw,
                show_ram,
                show_commit,
                has_commit,
            );
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
