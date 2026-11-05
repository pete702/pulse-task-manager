//! Multi-horizon neon chart for PULSE memory telemetry (1m / 5m / 15m).
use crate::i18n::Key;
use crate::modules::memory::chart_axis::{minutes_ago, window_minutes};
use crate::modules::memory::chart_chrome::{render_header, render_time_scale};
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::model::{ChartWindow, MemorySnapshot};
use crate::theme::colors;
use eframe::egui;
use egui_plot::{GridMark, HLine, Line, LineStyle, Plot, PlotPoints};

const Y_GRID: &[GridMark] = &[
    GridMark { value: 0.0, step_size: 25.0 },
    GridMark { value: 25.0, step_size: 25.0 },
    GridMark { value: 50.0, step_size: 25.0 },
    GridMark { value: 75.0, step_size: 25.0 },
    GridMark { value: 100.0, step_size: 25.0 },
];

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

pub(crate) fn cached_points(
    ui: &egui::Ui,
    key: &str,
    revision: u64,
    window_secs: usize,
    build: impl FnOnce() -> Vec<[f64; 2]>,
) -> Vec<[f64; 2]> {
    ui.ctx().data_mut(|d| {
        let cache: &mut (u64, usize, Vec<[f64; 2]>) =
            d.get_temp_mut_or_insert_with(egui::Id::new(key), || (u64::MAX, 0, Vec::new()));
        if cache.0 != revision || cache.1 != window_secs {
            cache.2 = build();
            cache.0 = revision;
            cache.1 = window_secs;
        }
        cache.2.clone()
    })
}

/// Extends a series so it spans the full window from `-span` to `0`.
/// For a single point (startup), only draw a 1-second lead-in [-1/60, 0]
/// instead of stretching across the entire window.
/// X coordinates are in MINUTES (see chart_axis::minutes_ago).
const ONE_SEC_MIN: f64 = 1.0 / 60.0;

fn anchored(mut points: Vec<[f64; 2]>, span: f64) -> Vec<[f64; 2]> {
    if points.len() == 1 {
        let y = points[0][1];
        points.insert(0, [-ONE_SEC_MIN, y]);
        points.push([0.0, y]);
        return points;
    }
    if let Some(first) = points.first() {
        if first[0] > -span {
            points.insert(0, [-span, first[1]]);
        }
    }
    if let Some(last) = points.last().copied() {
        if last[0] < 0.0 {
            points.push([0.0, last[1]]);
        }
    }
    points
}

pub fn render_memory_chart(ui: &mut egui::Ui, h: &mut MemoryHistory, s: &MemorySnapshot) {
    let mut selected = selected_window(ui.ctx());
    crate::theme::card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            let ram = h.current_ram_percent();
            let commit = s.commit_usage_percent();
            let swap = s.swap_usage_percent();
            let has_commit = s.kernel_metrics_valid && s.commit_limit_bytes > 0;
            render_header(ui, &mut selected, ram, commit, swap, has_commit);
            ui.add_space(6.0);
            h.chart_window = selected;

            let window_secs = selected.seconds() as usize;
            let now = h.latest_timestamp();
            let revision = h.revision();
            let span = window_minutes(selected);

            let ram_raw = cached_points(ui, "pulse_chart_ram", revision, window_secs, || {
                let points =
                    h.windowed_ram_timed().map(|(t, v)| [minutes_ago(t, now), v as f64]).collect();
                anchored(points, span)
            });

            let purple_key = if has_commit { "pulse_chart_commit" } else { "pulse_chart_swap" };
            let flat_value: f64 = if has_commit { commit as f64 } else { swap as f64 };
            let purple_raw = cached_points(ui, purple_key, revision, window_secs, || {
                let mut points: Vec<[f64; 2]> = if has_commit {
                    h.windowed_commit_timed()
                        .map(|(t, v)| [minutes_ago(t, now), v as f64])
                        .collect()
                } else {
                    h.windowed_swap_timed().map(|(t, v)| [minutes_ago(t, now), v as f64]).collect()
                };
                if points.is_empty() {
                    points.push([-ONE_SEC_MIN, flat_value]);
                }
                anchored(points, span)
            });

            let (ram_points, purple_points) =
                (PlotPoints::new(ram_raw), PlotPoints::new(purple_raw));
            let plot_height = (ui.available_height() - 24.0).clamp(180.0, 560.0);
            let legend = if has_commit { "Commit %" } else { "Swap %" };
            let plot = Plot::new(("mpp", selected.seconds()))
                .height(plot_height)
                .show_grid([false, true])
                .auto_bounds(egui::Vec2b::new(false, false))
                .x_axis_formatter(|_, _, _| String::new())
                .y_axis_formatter(|m, _, _| format!("{}%", m.value.round() as i32))
                .y_grid_spacer(move |_| Y_GRID.to_vec())
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
                .legend(
                    egui_plot::Legend::default()
                        .position(egui_plot::Corner::LeftTop)
                        .text_style(egui::TextStyle::Monospace),
                )
                .show(ui, |pu| {
                    pu.hline(
                        HLine::new(85.0)
                            .color(egui::Color32::from_rgba_unmultiplied(255, 42, 84, 110))
                            .style(LineStyle::Dashed { length: 6.0 }),
                    );
                    pu.line(
                        Line::new(ram_points)
                            .name(Key::RamPercent.t())
                            .color(colors::ACCENT_CYAN)
                            .width(2.2_f32)
                            .fill(0.0_f32),
                    );
                    pu.line(
                        Line::new(purple_points)
                            .name(legend)
                            .color(colors::ACCENT_PURPLE)
                            .width(1.8_f32),
                    );
                });
            render_time_scale(ui, span, *plot.transform.frame());
        });
    });
}
