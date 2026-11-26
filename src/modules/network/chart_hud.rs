//! Network chart helpers.

use crate::modules::network::view::ChartMode;
use crate::theme::colors;
use crate::ui::chart_scale::{
    clean_4_axis_max, format_y_tick, lock_time_chart_bounds, x_grid_spacer,
};
use eframe::egui;
use egui_plot::{GridMark, Line, Plot};
use std::sync::Arc;
pub(crate) fn draw_hud_pill(
    painter: &egui::Painter,
    pos: egui::Pos2,
    align: egui::Align2,
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
) {
    let galley = painter.layout_no_wrap(text.to_owned(), font, color);
    let rect = align.anchor_size(pos, galley.size()).expand2(egui::vec2(4.0, 1.5));
    painter.rect_filled(
        rect,
        egui::Rounding::same(3.0),
        egui::Color32::from_rgba_unmultiplied(11, 15, 22, 220),
    );
    painter.galley(rect.min + egui::vec2(4.0, 1.5), galley, color);
}
#[allow(clippy::type_complexity)]
pub(crate) fn draw_scanner(
    ui: &egui::Ui,
    rect: egui::Rect,
    span: f64,
    ceil: f64,
    series: [(&Arc<Vec<[f64; 2]>>, &str, egui::Color32); 2],
    unit: &str,
    hover: Option<egui::Pos2>,
) {
    let Some(pos) = hover else { return };
    if !rect.contains(pos) || span <= 0.0 || ceil <= 0.0 {
        return;
    }
    let frac = ((pos.x - rect.left()) / rect.width().max(1.0)).clamp(0.0, 1.0) as f64;
    let ago = span * (1.0 - frac);
    let target = -ago;
    let p = ui.painter();
    p.vline(
        pos.x,
        rect.y_range(),
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(0, 212, 255, 140)),
    );
    let when = if ago.abs() < 0.05 { "now".to_owned() } else { format!("-{ago:.1}m") };
    let mut lines: Vec<(String, egui::Color32)> = vec![(when, colors::TEXT_PRIMARY)];
    for (s, name, col) in series {
        let pt = crate::ui::chart_kit::find_nearest_point(s, target, f64::INFINITY);
        let Some(pt) = pt else { continue };
        let v = if pt[1].is_finite() { pt[1].max(0.0) } else { 0.0 };
        let y = rect.bottom() - ((v / ceil).clamp(0.0, 1.0) as f32) * rect.height();
        let d = egui::pos2(pos.x, y);
        p.circle_filled(
            d,
            5.0,
            egui::Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), 70),
        );
        p.circle_filled(d, 3.5, col);
        p.circle_stroke(d, 3.5, egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(240, 253, 250)));
        lines.push((format!("{name}: {v:.1} {unit}"), col));
    }
    if lines.len() <= 1 {
        return;
    }
    let (tw, th) = (180.0_f32, lines.len() as f32 * 15.0 + 8.0);
    let x =
        (pos.x + 14.0).clamp(rect.left() + 4.0, (rect.right() - tw - 4.0).max(rect.left() + 4.0));
    let y =
        (pos.y + 14.0).clamp(rect.top() + 4.0, (rect.bottom() - th - 4.0).max(rect.top() + 4.0));
    let tr = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(tw, th));
    p.rect_filled(
        tr,
        egui::Rounding::same(4.0),
        egui::Color32::from_rgba_unmultiplied(11, 12, 16, 235),
    );
    p.rect_stroke(tr, egui::Rounding::same(4.0), egui::Stroke::new(1.0_f32, colors::BORDER));
    for (i, (t, c)) in lines.iter().enumerate() {
        p.text(
            egui::pos2(x + 7.0, y + 4.0 + i as f32 * 15.0),
            egui::Align2::LEFT_TOP,
            t,
            egui::FontId::monospace(10.0),
            *c,
        );
    }
}
pub(crate) fn draw_plot(
    ui: &mut egui::Ui,
    first: &Arc<Vec<[f64; 2]>>,
    second: &Arc<Vec<[f64; 2]>>,
    window_secs: usize,
    latency: bool,
    legend: &str,
) {
    let span_min = window_secs as f64 / 60.0;
    let peak = first
        .iter()
        .chain(second.iter())
        .map(|p| if p[1].is_finite() { p[1].max(0.0) } else { 0.0 })
        .fold(0.0f64, f64::max);
    let ceiling = clean_4_axis_max(peak, if latency { 4.0 } else { 1.0 });
    let step = ceiling / 4.0;
    let y_grid =
        (0..=4).map(|i| GridMark { value: step * i as f64, step_size: step }).collect::<Vec<_>>();
    let (n1, n2, unit) =
        if latency { ("Ping ms", "Jitter ms", "ms") } else { ("RX Mbps", "TX Mbps", "Mbps") };
    let (first_pts, second_pts) =
        (super::chart_view::to_points(first), super::chart_view::to_points(second));
    let resp = ui.horizontal(|ui| {
        ui.add_space(52.0);
        Plot::new(("network_io_plot", latency))
            .width(ui.available_width().max(200.0))
            .height((ui.available_height() * 0.44).clamp(140.0, 220.0))
            .show_axes([false, false])
            .show_grid([true, true])
            .show_x(false)
            .show_y(false)
            .auto_bounds(egui::Vec2b::new(false, false))
            .y_grid_spacer(move |_| y_grid.clone())
            .x_grid_spacer(move |input| x_grid_spacer(span_min, &input))
            .include_x(-span_min)
            .include_x(0.0)
            .include_y(crate::ui::chart_kit::baseline_y())
            .include_y(ceiling)
            .allow_scroll(false)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_boxed_zoom(false)
            .show(ui, |pu| {
                lock_time_chart_bounds(pu, window_secs, ceiling);
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
    if resp.inner.response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
    }
    draw_hud(ui, rect, span_min, ceiling, unit, legend);
    draw_scanner(
        ui,
        rect,
        span_min,
        ceiling,
        [(first, n1, colors::ACCENT_CYAN), (second, n2, colors::ACCENT_AMBER)],
        unit,
        resp.inner.response.hover_pos(),
    );
}
pub(crate) fn header_modes(
    ui: &mut egui::Ui,
    current: ChartMode,
    window_secs: usize,
    scope: Option<&str>,
    state: &mut crate::modules::network::view::NetworkViewState,
) {
    ui.horizontal_wrapped(|ui| {
        for (label, mode) in [
            ("TRAFFIC (Mbps)", ChartMode::Traffic),
            ("PING & JITTER (ms)", ChartMode::Latency),
            ("SPLIT", ChartMode::Split),
        ] {
            let sel = current == mode;
            let (fg, fill, border) = if sel {
                (
                    colors::TEXT_PRIMARY,
                    colors::ACCENT_CYAN.gamma_multiply(0.20),
                    colors::ACCENT_CYAN,
                )
            } else {
                (colors::TEXT_SECONDARY, egui::Color32::TRANSPARENT, colors::BORDER)
            };
            if ui
                .add(
                    egui::Button::new(egui::RichText::new(label).monospace().size(11.0).color(fg))
                        .fill(fill)
                        .stroke(egui::Stroke::new(1.0_f32, border)),
                )
                .clicked()
            {
                state.set_mode(mode);
            }
        }
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
                    state.set_scope_all();
                }
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            for (label, secs) in [("15m", 900), ("5m", 300), ("1m", 60)] {
                let c =
                    if secs == window_secs { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY };
                if ui
                    .add(egui::Button::new(
                        egui::RichText::new(label).monospace().size(11.0).color(c),
                    ))
                    .clicked()
                {
                    state.set_window_secs(secs);
                }
            }
        });
    });
}
pub(crate) fn draw_hud(
    ui: &egui::Ui,
    rect: egui::Rect,
    span_min: f64,
    ceiling: f64,
    unit: &str,
    legend: &str,
) {
    let p = ui.painter();
    let c = egui::Color32::from_rgba_unmultiplied(148, 163, 184, 170);
    let f = egui::FontId::monospace(10.0);
    let gx = rect.left() - 6.0;
    let giga = ceiling >= 1000.0 && (unit == "Mbps" || unit == "MB/s");
    let top = if giga && unit == "Mbps" {
        format!("{:.2} Gbps", ceiling / 1000.0)
    } else if giga {
        format!("{:.2} GB/s", ceiling / 1000.0)
    } else {
        format_y_tick(ceiling, ceiling, unit)
    };
    draw_hud_pill(p, egui::pos2(gx, rect.top() + 3.0), egui::Align2::RIGHT_TOP, &top, f.clone(), c);
    for v in [75.0, 50.0, 25.0] {
        let y = rect.bottom() - (v / 100.0) * rect.height();
        let val = v as f64 / 100.0 * ceiling;
        let t = if giga { format!("{:.1}", val / 1000.0) } else { format_y_tick(val, ceiling, "") };
        draw_hud_pill(p, egui::pos2(gx, y - 2.0), egui::Align2::RIGHT_CENTER, &t, f.clone(), c);
    }
    if !legend.is_empty() {
        draw_hud_pill(
            p,
            egui::pos2(rect.left() + 70.0, rect.top() + 10.0),
            egui::Align2::LEFT_CENTER,
            legend,
            f.clone(),
            c,
        );
    }
    let y = rect.bottom() - 5.0;
    draw_hud_pill(
        p,
        egui::pos2(rect.left() + 8.0, y),
        egui::Align2::LEFT_BOTTOM,
        &format!("{span_min:.0}m ago"),
        f.clone(),
        c,
    );
    draw_hud_pill(p, egui::pos2(rect.right() - 8.0, y), egui::Align2::RIGHT_BOTTOM, "now", f, c);
}
