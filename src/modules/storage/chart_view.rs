use crate::modules::cpu::chart_axis::x_grid_spacer;
use crate::modules::memory::downsampling::{lttb_segments, nice_ceiling};
use crate::modules::storage::chart_tip::{draw_hover_tip, draw_toggles};
use crate::modules::storage::collector::IO_SPIKE_THRESHOLD_MB_S;
use crate::modules::storage::view_helpers::truncate;
use crate::theme::{card_frame, colors};
use crate::ui::primitives::{cyber_accent, draw_cyber_corner_brackets};
use eframe::egui;
use egui_plot::{GridMark, HLine, Line, LineStyle, Plot, PlotBounds, PlotPoints};
use std::sync::Arc;
fn san(v: f64) -> f64 {
    if v.is_finite() {
        v.max(0.0)
    } else {
        0.0
    }
}
pub(crate) fn stepped_ceiling(
    r: &Arc<Vec<[f64; 2]>>,
    w: &Arc<Vec<[f64; 2]>>,
    sr: bool,
    sw: bool,
) -> f64 {
    let peak = r
        .iter()
        .filter(|_| sr)
        .chain(w.iter().filter(|_| sw))
        .fold(0.0_f64, |a, p| a.max(san(p[1])));
    nice_ceiling(peak, 1.0)
}
pub(crate) fn next_visibility(sr: bool, sw: bool, is_read: bool) -> (bool, bool) {
    let nr = sr ^ is_read;
    let nw = sw ^ !is_read;
    if nr || nw {
        (nr, nw)
    } else {
        (sr, sw)
    }
}
fn lbl(s: impl Into<String>, sz: f32, c: egui::Color32) -> egui::RichText {
    egui::RichText::new(s).monospace().size(sz).color(c)
}
fn tx(
    p: &egui::Painter,
    x: f32,
    y: f32,
    a: egui::Align2,
    s: impl std::fmt::Display,
    sz: f32,
    c: egui::Color32,
) {
    p.text(egui::pos2(x, y), a, s, egui::FontId::monospace(sz), c);
}
pub fn render_storage_chart(
    ui: &mut egui::Ui,
    read: &Arc<Vec<[f64; 2]>>,
    write: &Arc<Vec<[f64; 2]>>,
    window_secs: usize,
    scope: Option<&str>,
    show_read: bool,
    show_write: bool,
) {
    let lr = read.last().map(|p| san(p[1]) as f32).unwrap_or(0.0);
    let lw = write.last().map(|p| san(p[1]) as f32).unwrap_or(0.0);
    let resp = card_frame().show(ui, |ui| {
        let header_w = ui.available_width();
        let (header_rect, _) =
            ui.allocate_exact_size(egui::vec2(header_w, 22.0), egui::Sense::hover());
        let scope_short = scope.map(|s| truncate(s, 14));
        let right_controls_width = if scope_short.is_some() { 215.0 } else { 120.0 };
        let left_w = (header_rect.width() - right_controls_width - 8.0).max(120.0);
        let raw_title = if header_rect.width() < 380.0 {
            format!("THROUGHPUT  R:{lr:.1} W:{lw:.1}")
        } else {
            format!("DISK THROUGHPUT  R: {lr:.1} | W: {lw:.1} MB/s")
        };
        let title = truncate(&raw_title, (left_w / 7.0) as usize);
        let mut left_ui = ui.child_ui(
            egui::Rect::from_min_size(header_rect.min, egui::vec2(left_w, 22.0)),
            egui::Layout::left_to_right(egui::Align::Center),
        );
        left_ui.set_min_width(left_w);
        left_ui.label(lbl(title, 11.0, colors::TEXT_PRIMARY));
        let mut right_ui = ui.child_ui(
            egui::Rect::from_min_size(
                egui::pos2(header_rect.right() - right_controls_width, header_rect.top()),
                egui::vec2(right_controls_width, 22.0),
            ),
            egui::Layout::right_to_left(egui::Align::Center),
        );
        for (label, secs) in [("15m", 900), ("5m", 300), ("1m", 60)] {
            pick_window(&mut right_ui, label, secs, window_secs);
        }
        if let Some(s) = scope_short {
            if right_ui
                .add(egui::Button::new(lbl(format!("[ {s} x ]"), 10.0, colors::TEXT_SECONDARY)))
                .clicked()
            {
                crate::modules::storage::view::clear_selected();
            }
        }
        ui.add_space(4.0);
        let span = window_secs as f64 / 60.0;
        let ceil = stepped_ceiling(read, write, show_read, show_write);
        let step = ceil / 4.0;
        let rpts = to_points(read);
        let wpts = to_points(write);
        let plot_resp = ui.horizontal(|ui| {
            ui.add_space(52.0);
            Plot::new("storage_io_plot")
                .width(ui.available_width().max(200.0))
                .height(220.0)
                .show_axes([false, false])
                .show_x(false)
                .show_y(false)
                .show_grid([true, true])
                .auto_bounds(egui::Vec2b::new(false, false))
                .y_grid_spacer(move |_| {
                    (0..=4).map(|i| GridMark { value: step * i as f64, step_size: step }).collect()
                })
                .x_grid_spacer(move |input| x_grid_spacer(span, &input))
                .include_x(-span)
                .include_x(0.0)
                .include_y(0.0)
                .include_y(ceil)
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
                    let label = if name.is_empty() { "Value" } else { name };
                    format!("{label}: {:.1} MB/s ({when})", san(value.y))
                })
                .show(ui, |pu| {
                    pu.set_plot_bounds(PlotBounds::from_min_max([-span, 0.0], [0.0, ceil]));
                    pu.hline(
                        HLine::new(IO_SPIKE_THRESHOLD_MB_S as f64)
                            .color(egui::Color32::from_rgba_unmultiplied(255, 42, 84, 110))
                            .style(LineStyle::Dashed { length: 6.0 }),
                    );
                    if show_read {
                        pu.line(
                            Line::new(rpts)
                                .name("Read")
                                .color(colors::ACCENT_CYAN)
                                .width(2.2_f32)
                                .fill(0.0_f32),
                        );
                    }
                    if show_write {
                        pu.line(
                            Line::new(wpts)
                                .name("Write")
                                .color(colors::ACCENT_AMBER)
                                .width(2.2_f32),
                        );
                    }
                })
        });
        let rect = *plot_resp.inner.transform.frame();
        let hover = plot_resp.inner.response.hover_pos();
        draw_hud(ui, rect, span, ceil);
        draw_toggles(ui, rect, show_read, show_write);
        draw_hover_tip(ui, rect, span, read, write, show_read, show_write, hover);
    });
    draw_cyber_corner_brackets(ui, resp.response.rect, cyber_accent((lr + lw).clamp(0.0, 100.0)));
}
fn pick_window(ui: &mut egui::Ui, label: &str, secs: usize, current: usize) {
    let c = if secs == current { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY };
    if ui.add(egui::Button::new(lbl(label, 11.0, c))).clicked() {
        crate::modules::storage::view::set_window_secs(secs);
    }
}
fn to_points(s: &Arc<Vec<[f64; 2]>>) -> PlotPoints {
    let clean: Vec<[f64; 2]> = s.iter().map(|p| [p[0], san(p[1])]).collect();
    if clean.is_empty() {
        PlotPoints::new(vec![[-1.0 / 60.0, 0.0], [0.0, 0.0]])
    } else if clean.len() <= 920 {
        PlotPoints::new(clean)
    } else {
        PlotPoints::new(lttb_segments(&Arc::new(clean), 920))
    }
}
fn draw_hud(ui: &egui::Ui, rect: egui::Rect, span: f64, ceil: f64) {
    let p = ui.painter();
    let col = egui::Color32::from_rgba_unmultiplied(148, 163, 184, 135);
    let gx = rect.left() - 6.0;
    let giga = ceil >= 1000.0;
    let top = if giga {
        format!("{:.2} GB/s", ceil / 1000.0)
    } else if ceil < 10.0 {
        format!("{ceil:.1} MB/s")
    } else {
        format!("{ceil:.0} MB/s")
    };
    tx(p, gx, rect.top() + 3.0, egui::Align2::RIGHT_TOP, top, 10.0, col);
    for v in [75.0, 50.0, 25.0] {
        let y = rect.bottom() - (v / 100.0) * rect.height();
        let val = v as f64 / 100.0 * ceil;
        let label = if giga {
            format!("{:.1}", val / 1000.0)
        } else if ceil < 10.0 {
            format!("{val:.1}")
        } else {
            format!("{val:.0}")
        };
        tx(p, gx, y - 2.0, egui::Align2::RIGHT_CENTER, label, 10.0, col);
    }
    let y = rect.bottom() - 5.0;
    let bg = egui::Color32::from_rgba_unmultiplied(11, 12, 16, 210);
    for (dx, w, txt, right) in [
        (3.0_f32, 76.0_f32, format!("{span:.0}m ago"), false),
        (rect.width() - 47.0, 40.0_f32, "now".to_owned(), true),
    ] {
        let r =
            egui::Rect::from_min_size(egui::pos2(rect.left() + dx, y - 15.0), egui::vec2(w, 16.0));
        p.rect_filled(r, egui::Rounding::same(2.0), bg);
        if right {
            tx(p, r.right() - 8.0, y, egui::Align2::RIGHT_BOTTOM, txt, 11.0, col);
        } else {
            tx(p, r.left() + 5.0, y, egui::Align2::LEFT_BOTTOM, txt, 11.0, col);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn toggle_peak_enabled_only() {
        let r = Arc::new(vec![[0.0, 10.0], [-1.0, 900.0]]);
        let w = Arc::new(vec![[0.0, 5.0], [-1.0, 20.0]]);
        let both = stepped_ceiling(&r, &w, true, true);
        let ro = stepped_ceiling(&r, &w, true, false);
        let wo = stepped_ceiling(&r, &w, false, true);
        assert!(both >= ro && both >= wo && ro >= wo);
        assert_eq!(next_visibility(true, true, true), (false, true));
        assert_eq!(stepped_ceiling(&Arc::new(vec![[0.0, f64::NAN]]), &w, true, false), 1.0);
    }
}
