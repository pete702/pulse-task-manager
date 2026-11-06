//! Network chart: traffic (Mbps), ping/jitter (ms), and split side by side.

use std::sync::Arc;

use eframe::egui;
use egui_plot::{GridMark, Line, Plot, PlotPoints};

use crate::modules::cpu::chart_axis::x_grid_spacer;
use crate::modules::memory::downsampling::lttb_downsample;
use crate::modules::network::view::ChartMode;
use crate::theme::{card_frame, colors};
use crate::ui::primitives::{cyber_accent, draw_cyber_corner_brackets};

const CEILINGS: &[f64] = &[
    1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0, 25000.0,
];

/// Smallest stepped ceiling at or above the peak (at least 10).
pub(crate) fn stepped_ceiling(a: &Arc<Vec<[f64; 2]>>, b: &Arc<Vec<[f64; 2]>>) -> f64 {
    let peak = a.iter().chain(b.iter()).map(|p| p[1]).fold(0.0f64, f64::max);
    let need = peak.max(10.0);
    CEILINGS.iter().copied().find(|c| *c >= need).unwrap_or_else(|| (peak * 1.25).ceil())
}

fn draw_plot(
    ui: &mut egui::Ui,
    first: &Arc<Vec<[f64; 2]>>,
    second: &Arc<Vec<[f64; 2]>>,
    span_min: f64,
    latency: bool,
    id: (usize, bool),
) {
    let window_secs = (span_min * 60.0) as usize;
    let ceiling = stepped_ceiling(first, second);
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
    let rect = *Plot::new(("network_io", window_secs, id))
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
            let when = if ago.abs() < 0.05 { "now".to_owned() } else { format!("{:.1}m ago", ago) };
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
        .transform
        .frame();
    draw_hud(ui, rect, span_min, ceiling, unit);
}

fn header_modes(ui: &mut egui::Ui, current: ChartMode, window_secs: usize, scope: Option<&str>) {
    ui.horizontal_wrapped(|ui| {
        pick_mode(ui, "[ TRAFFIC (Mbps) ]", ChartMode::Traffic, current == ChartMode::Traffic);
        pick_mode(ui, "[ PING & JITTER (ms) ]", ChartMode::Latency, current == ChartMode::Latency);
        pick_mode(ui, "[ SPLIT ]", ChartMode::Split, current == ChartMode::Split);
        if current != ChartMode::Latency {
            if let Some(name) = scope {
                ui.label(
                    egui::RichText::new(format!("[ SCOPE: {name} ]"))
                        .monospace()
                        .size(11.0)
                        .color(colors::ACCENT_CYAN),
                );
                if ui
                    .small_button(egui::RichText::new("[ Show All ]").monospace().size(11.0))
                    .clicked()
                {
                    crate::modules::network::view::set_scope_all();
                }
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            pick_window(ui, "1m", 60, window_secs);
            pick_window(ui, "5m", 300, window_secs);
            pick_window(ui, "15m", 900, window_secs);
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
        draw_plot(ui, first, second, window_secs as f64 / 60.0, latency, (window_secs, latency));
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
                draw_plot(&mut cols[0], t_first, t_second, span, false, (window_secs, false));
                draw_plot(&mut cols[1], l_first, l_second, span, true, (window_secs, true));
            });
        } else {
            draw_plot(ui, t_first, t_second, span, false, (window_secs, false));
            ui.add_space(4.0);
            draw_plot(ui, l_first, l_second, span, true, (window_secs, true));
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

/// Renders one chart-mode toggle button. Active gets Cyan border + fill.
fn pick_mode(ui: &mut egui::Ui, label: &str, mode: ChartMode, selected: bool) {
    let (fg, fill, border) = if selected {
        (colors::ACCENT_CYAN, colors::ACCENT_CYAN, colors::ACCENT_CYAN)
    } else {
        (colors::TEXT_SECONDARY, colors::BORDER, colors::BORDER)
    };
    let fill = if selected { fill } else { egui::Color32::TRANSPARENT };
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
    } else {
        PlotPoints::new(lttb_segments(series, 300))
    }
}

/// LTTB that preserves NAN gaps: each contiguous finite segment is
/// downsampled independently so outages stay broken, never bridged.
pub fn lttb_segments(points: &[[f64; 2]], threshold: usize) -> Vec<[f64; 2]> {
    if points.iter().all(|p| p[1].is_finite()) {
        return lttb_downsample(points, threshold);
    }
    let total_finite = points.iter().filter(|p| p[1].is_finite()).count();
    if total_finite <= threshold {
        return points.to_vec();
    }
    let mut out = Vec::with_capacity(points.len().min(threshold + 32));
    let mut start = 0usize;
    while start < points.len() {
        if !points[start][1].is_finite() {
            out.push(points[start]);
            start += 1;
            continue;
        }
        let mut end = start;
        while end < points.len() && points[end][1].is_finite() {
            end += 1;
        }
        let seg = &points[start..end];
        let budget = (seg.len() * threshold / total_finite.max(1)).clamp(2, threshold);
        out.extend(lttb_downsample(seg, budget.max(2)));
        start = end;
    }
    out
}

fn hud_color() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(148, 163, 184, 135)
}

fn draw_hud(ui: &egui::Ui, rect: egui::Rect, span_min: f64, ceiling: f64, unit: &str) {
    let painter = ui.painter();
    let col = hud_color();
    let top = format!("{ceiling:.0} {unit}");
    painter.text(
        egui::pos2(rect.right() - 6.0, rect.top() + 3.0),
        egui::Align2::RIGHT_TOP,
        top,
        egui::FontId::monospace(10.0),
        col,
    );
    for v in [75.0, 50.0, 25.0] {
        let y = rect.bottom() - (v / 100.0) * rect.height();
        let label = format!("{:.0}", v as f64 / 100.0 * ceiling);
        painter.text(
            egui::pos2(rect.right() - 6.0, y - 2.0),
            egui::Align2::RIGHT_CENTER,
            label,
            egui::FontId::monospace(10.0),
            col,
        );
    }
    let left = format!("{:.0}m ago", span_min);
    let y = rect.bottom() - 5.0;
    painter.text(
        egui::pos2(rect.left() + 8.0, y),
        egui::Align2::LEFT_BOTTOM,
        left,
        egui::FontId::monospace(11.0),
        col,
    );
    painter.text(
        egui::pos2(rect.right() - 8.0, y),
        egui::Align2::RIGHT_BOTTOM,
        "now",
        egui::FontId::monospace(11.0),
        col,
    );
}

#[cfg(test)]
#[path = "chart_view_tests.rs"]
mod tests;
