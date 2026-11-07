//! Network chart: traffic (Mbps), ping/jitter (ms), and split side by side.

use std::sync::Arc;

use eframe::egui;
use egui_plot::{GridMark, Line, Plot, PlotPoints};

use crate::modules::cpu::chart_axis::x_grid_spacer;
use crate::modules::memory::downsampling::{lttb_segments, nice_ceiling};
use crate::modules::network::view::ChartMode;
use crate::theme::{card_frame, colors};
use crate::ui::primitives::{cyber_accent, draw_cyber_corner_brackets};

fn draw_plot(
    ui: &mut egui::Ui,
    first: &Arc<Vec<[f64; 2]>>,
    second: &Arc<Vec<[f64; 2]>>,
    span_min: f64,
    latency: bool,
    legend: &str,
    id: (usize, bool),
) {
    let window_secs = (span_min * 60.0) as usize;
    let peak = first.iter().chain(second.iter()).map(|p| p[1]).fold(0.0f64, f64::max);
    let ceiling = nice_ceiling(peak, if latency { 10.0 } else { 1.0 });
    let step = ceiling / 4.0;
    let y_grid = [
        GridMark { value: f64::from(0), step_size: step },
        GridMark { value: step, step_size: step },
        GridMark { value: step * 2.0, step_size: step },
        GridMark { value: step * 3.0, step_size: step },
        GridMark { value: ceiling, step_size: step },
    ];
    let (n1, n2, unit) =
        if latency { ("Ping ms", "Jitter ms", "ms") } else { ("RX Mbps", "TX Mbps", "Mbps") };
    let first_pts = to_points(first);
    let second_pts = to_points(second);
    let resp = ui.horizontal(|ui| {
        ui.add_space(52.0);
        Plot::new(("network_io", window_secs, id))
            .width(ui.available_width().max(200.0))
            .height(220.0)
            .show_axes([false, false])
            .show_grid([true, true])
            .auto_bounds(egui::Vec2b::new(false, false))
            .y_grid_spacer(move |_| y_grid.to_vec())
            .x_grid_spacer(move |input| x_grid_spacer(span_min, &input))
            .include_x(-span_min)
            .include_x(0.0)
            .include_y(0.0)
            .include_y(ceiling)
            .allow_scroll(false)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_boxed_zoom(false)
            .label_formatter(move |name, value| {
                let ago = -value.x;
                let when =
                    if ago.abs() < 0.05 { "now".to_owned() } else { format!("{:.1}m ago", ago) };
                format!("{}: {:.1} {} ({})", name, value.y, unit, when)
            })
            .show(ui, |pu| {
                pu.line(
                    Line::new(first_pts)
                        .name(n1)
                        .color(colors::ACCENT_CYAN)
                        .width(2.2_f32)
                        .fill(0.0_f32),
                );
                pu.line(Line::new(second_pts).name(n2).color(colors::ACCENT_AMBER).width(2.2_f32));
            })
    });
    let rect = *resp.inner.transform.frame();
    draw_hud(ui, rect, span_min, ceiling, unit, legend);
}

fn header_modes(ui: &mut egui::Ui, current: ChartMode, window_secs: usize, scope: Option<&str>) {
    ui.horizontal_wrapped(|ui| {
        pick_mode(ui, "TRAFFIC (Mbps)", ChartMode::Traffic, current == ChartMode::Traffic);
        pick_mode(ui, "PING & JITTER (ms)", ChartMode::Latency, current == ChartMode::Latency);
        pick_mode(ui, "SPLIT", ChartMode::Split, current == ChartMode::Split);
        if current != ChartMode::Latency {
            if let Some(name) = scope {
                ui.label(
                    egui::RichText::new(format!("SCOPE: {name}"))
                        .monospace()
                        .size(11.0)
                        .color(colors::ACCENT_CYAN),
                );
                if ui.small_button(egui::RichText::new("Show All").monospace().size(11.0)).clicked()
                {
                    crate::modules::network::view::set_scope_all();
                }
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            for (label, secs) in [("15m", 900), ("5m", 300), ("1m", 60)] {
                pick_window(ui, label, secs, window_secs);
            }
        });
    });
}

/// Renders the single chart card; `latency` selects ping/jitter vs rx/tx.
pub fn render_network_chart(
    ui: &mut egui::Ui,
    first: &Arc<Vec<[f64; 2]>>,
    second: &Arc<Vec<[f64; 2]>>,
    window_secs: usize,
    latency: bool,
    scope: Option<&str>,
) {
    let mode = if latency { ChartMode::Latency } else { ChartMode::Traffic };
    let peak_now = second.last().map(|p| p[1]).unwrap_or(0.0);
    let resp = card_frame().show(ui, |ui| {
        header_modes(ui, mode, window_secs, scope);
        ui.add_space(4.0);
        draw_plot(
            ui,
            first,
            second,
            window_secs as f64 / 60.0,
            latency,
            "",
            (window_secs, latency),
        );
    });
    draw_cyber_corner_brackets(
        ui,
        resp.response.rect,
        cyber_accent(peak_now.clamp(0.0, 100.0) as f32),
    );
}

/// Renders traffic (left) and latency (right) side by side; stacked when narrow.
pub fn render_network_split(
    ui: &mut egui::Ui,
    t_first: &Arc<Vec<[f64; 2]>>,
    t_second: &Arc<Vec<[f64; 2]>>,
    l_first: &Arc<Vec<[f64; 2]>>,
    l_second: &Arc<Vec<[f64; 2]>>,
    window_secs: usize,
    scope: Option<&str>,
) {
    let span = window_secs as f64 / 60.0;
    let resp = card_frame().show(ui, |ui| {
        header_modes(ui, ChartMode::Split, window_secs, scope);
        ui.add_space(4.0);
        if ui.available_width() >= 720.0 {
            ui.columns(2, |cols| {
                draw_plot(
                    &mut cols[0],
                    t_first,
                    t_second,
                    span,
                    false,
                    "RX / TX",
                    (window_secs, false),
                );
                draw_plot(
                    &mut cols[1],
                    l_first,
                    l_second,
                    span,
                    true,
                    "Ping / Jitter",
                    (window_secs, true),
                );
            });
        } else {
            draw_plot(ui, t_first, t_second, span, false, "RX / TX", (window_secs, false));
            ui.add_space(4.0);
            draw_plot(ui, l_first, l_second, span, true, "Ping / Jitter", (window_secs, true));
        }
    });
    draw_cyber_corner_brackets(ui, resp.response.rect, cyber_accent(100.0));
}

fn pick_window(ui: &mut egui::Ui, label: &str, secs: usize, current: usize) {
    let selected = secs == current;
    let btn =
        egui::Button::new(egui::RichText::new(label).monospace().size(11.0).color(if selected {
            colors::ACCENT_CYAN
        } else {
            colors::TEXT_SECONDARY
        }));
    if ui.add(btn).clicked() {
        crate::modules::network::view::set_window_secs(secs);
    }
}

/// Renders one chart-mode toggle button. Active gets dim Cyan fill + stroke.
fn pick_mode(ui: &mut egui::Ui, label: &str, mode: ChartMode, selected: bool) {
    let (fg, fill, border) = if selected {
        (colors::TEXT_PRIMARY, colors::ACCENT_CYAN.gamma_multiply(0.20), colors::ACCENT_CYAN)
    } else {
        (colors::TEXT_SECONDARY, egui::Color32::TRANSPARENT, colors::BORDER)
    };
    let btn = egui::Button::new(egui::RichText::new(label).monospace().size(11.0).color(fg))
        .fill(fill)
        .stroke(egui::Stroke::new(1.0_f32, border));
    if ui.add(btn).clicked() {
        crate::modules::network::view::set_mode(mode);
    }
}

fn to_points(series: &Arc<Vec<[f64; 2]>>) -> PlotPoints {
    if series.is_empty() {
        PlotPoints::new(vec![[-1.0 / 60.0, 0.0], [0.0, 0.0]])
    } else if series.len() <= 300 {
        PlotPoints::new(series.to_vec())
    } else {
        PlotPoints::new(lttb_segments(series, 300))
    }
}

fn hud_color() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(148, 163, 184, 135)
}

fn draw_hud(
    ui: &egui::Ui,
    rect: egui::Rect,
    span_min: f64,
    ceiling: f64,
    unit: &str,
    legend: &str,
) {
    let p = ui.painter();
    let c = hud_color();
    let g = egui::pos2;
    let f = egui::FontId::monospace;
    let gx = rect.left() - 6.0;
    let is_giga = ceiling >= 1000.0;
    let top = if unit == "Mbps" && is_giga {
        format!("{:.2} Gbps", ceiling / 1000.0)
    } else if unit == "MB/s" && is_giga {
        format!("{:.2} GB/s", ceiling / 1000.0)
    } else {
        format!("{ceiling:.0} {unit}")
    };
    p.text(g(gx, rect.top() + 3.0), egui::Align2::RIGHT_TOP, top, f(10.0), c);
    for v in [75.0, 50.0, 25.0] {
        let y = rect.bottom() - (v / 100.0) * rect.height();
        let val = v as f64 / 100.0 * ceiling;
        let t = if is_giga && (unit == "Mbps" || unit == "MB/s") {
            format!("{:.1}", val / 1000.0)
        } else {
            format!("{val:.0}")
        };
        p.text(g(gx, y - 2.0), egui::Align2::RIGHT_CENTER, t, f(10.0), c);
    }
    if !legend.is_empty() {
        p.text(
            g(rect.left() + 70.0, rect.top() + 10.0),
            egui::Align2::LEFT_CENTER,
            legend,
            f(10.0),
            c,
        );
    }
    let t = format!("{span_min:.0}m ago");
    let y = rect.bottom() - 5.0;
    p.text(g(rect.left() + 8.0, y), egui::Align2::LEFT_BOTTOM, t, f(11.0), c);
    p.text(g(rect.right() - 8.0, y), egui::Align2::RIGHT_BOTTOM, "now", f(11.0), c);
}

#[cfg(test)]
#[path = "chart_view_tests.rs"]
mod tests;
