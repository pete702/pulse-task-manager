//! Interactive in-canvas HUD for the GPU chart: legend toggles and hover scanner.
use crate::modules::cpu::chart_hud::{draw_x_axis_hud_labels, draw_y_axis_hud_labels};
use crate::modules::memory::model::ChartWindow;
use crate::theme::colors;
use crate::ui::chart_kit::find_nearest_point;
use crate::ui::chart_scale::{lock_time_chart_bounds, x_grid_spacer, PERCENT_Y_GRID};
use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints, Points};

pub fn legend_ids() -> (egui::Id, egui::Id) {
    (egui::Id::new("gpu_chart_show_engine"), egui::Id::new("gpu_chart_show_vram"))
}

fn legend_item(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    id: egui::Id,
    label: &str,
    color: egui::Color32,
) -> bool {
    let mut shown: bool = ui.ctx().data_mut(|d| *d.get_temp_mut_or_insert_with(id, || true));
    let response = ui.interact(rect, id, egui::Sense::click());
    if response.clicked() {
        shown = !shown;
        ui.ctx().data_mut(|d| d.insert_temp(id, shown));
    }
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let painter = ui.painter();
    let alpha = if shown { 195 } else { 60 };
    let dot = egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
    painter.circle_filled(egui::pos2(rect.left() + 6.0, rect.center().y), 3.5, dot);
    painter.text(
        egui::pos2(rect.left() + 14.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::monospace(10.0),
        egui::Color32::from_rgba_unmultiplied(148, 163, 184, if shown { 185 } else { 70 }),
    );
    shown
}

fn legend_rects(plot_rect: egui::Rect) -> (egui::Rect, egui::Rect) {
    let y = plot_rect.top() + 3.0;
    let engine =
        egui::Rect::from_min_size(egui::pos2(plot_rect.left() + 8.0, y), egui::vec2(100.0, 15.0));
    let vram =
        egui::Rect::from_min_size(egui::pos2(plot_rect.left() + 114.0, y), egui::vec2(72.0, 15.0));
    (engine, vram)
}

/// Two clickable pills; returns (show_engine, show_vram).
pub fn draw_hud_legend(ui: &mut egui::Ui, plot_rect: egui::Rect) -> (bool, bool) {
    let (engine_id, vram_id) = legend_ids();
    let (engine_rect, vram_rect) = legend_rects(plot_rect);
    let engine = legend_item(ui, engine_rect, engine_id, "GPU Engine %", colors::ACCENT_PURPLE);
    let vram = legend_item(ui, vram_rect, vram_id, "VRAM %", colors::ACCENT_CYAN);
    (engine, vram)
}

fn fmt_time(ago_min: f64) -> String {
    if ago_min.abs() < 0.01 {
        "now".to_owned()
    } else {
        format!("{ago_min:.1}m ago")
    }
}

/// Vertical scanner with exact percentages for the two visible series.
pub fn draw_hud_scanner(
    ui: &egui::Ui,
    plot_rect: egui::Rect,
    span_min: f64,
    engine_pts: &[[f64; 2]],
    vram_pts: &[[f64; 2]],
    show_engine: bool,
    show_vram: bool,
) {
    let Some(pos) = ui.ctx().pointer_hover_pos() else { return };
    if !plot_rect.contains(pos) || span_min <= 0.0 {
        return;
    }
    let (engine_rect, vram_rect) = legend_rects(plot_rect);
    if engine_rect.contains(pos) || vram_rect.contains(pos) {
        return;
    }
    let frac = ((pos.x - plot_rect.left()) / plot_rect.width().max(1.0)).clamp(0.0, 1.0) as f64;
    let ago = span_min * (1.0 - frac);
    let painter = ui.painter();
    painter.vline(
        pos.x,
        plot_rect.y_range(),
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(0, 212, 255, 140)),
    );
    let mut tip = vec![(fmt_time(ago), colors::TEXT_PRIMARY)];
    for (enabled, pts, color, label) in [
        (show_engine, engine_pts, colors::ACCENT_PURPLE, "GPU Engine"),
        (show_vram, vram_pts, colors::ACCENT_CYAN, "VRAM"),
    ] {
        if !enabled {
            continue;
        }
        let Some(pt) = find_nearest_point(pts, -ago, f64::INFINITY) else { continue };
        let value = pt[1].clamp(0.0, 100.0);
        let y = plot_rect.bottom() - (value as f32 / 100.0) * plot_rect.height();
        painter.circle_filled(egui::pos2(pos.x, y), 4.0, color);
        tip.push((format!("{label}: {value:.1}%"), color));
    }
    if tip.len() <= 1 {
        return;
    }
    let (tw, th) = (150.0, tip.len() as f32 * 15.0 + 8.0);
    let x = (pos.x + 14.0)
        .clamp(plot_rect.left() + 4.0, (plot_rect.right() - tw - 4.0).max(plot_rect.left() + 4.0));
    let y = (pos.y + 14.0)
        .clamp(plot_rect.top() + 4.0, (plot_rect.bottom() - th - 4.0).max(plot_rect.top() + 4.0));
    let rect = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(tw, th));
    let fill = egui::Color32::from_rgba_unmultiplied(11, 12, 16, 235);
    painter.rect_filled(rect, egui::Rounding::same(4.0_f32), fill);
    painter.rect_stroke(
        rect,
        egui::Rounding::same(4.0_f32),
        egui::Stroke::new(1.0_f32, colors::BORDER),
    );
    for (i, (text, color)) in tip.iter().enumerate() {
        painter.text(
            egui::pos2(x + 7.0, y + 4.0 + i as f32 * 15.0),
            egui::Align2::LEFT_TOP,
            text,
            egui::FontId::monospace(10.0),
            *color,
        );
    }
}

pub fn flatten(segments: &[Vec<[f64; 2]>]) -> Vec<[f64; 2]> {
    segments.iter().flat_map(|s| s.iter().copied()).collect()
}

/// Splits one Option series into ordered segments; missing samples stay honest gaps.
pub fn segments(
    rows: &[(f64, Option<f32>, Option<f32>)],
    span_min: f64,
    engine: bool,
) -> Vec<Vec<[f64; 2]>> {
    let max_age = span_min * 60.0;
    let mut out: Vec<Vec<[f64; 2]>> = Vec::new();
    let mut prev = false;
    for &(age, vram, gpu) in rows {
        if age > max_age {
            continue;
        }
        let x = -(age / 60.0).min(span_min);
        if let Some(v) = if engine { gpu } else { vram } {
            if !prev {
                out.push(Vec::new());
            }
            out.last_mut().unwrap().push([x, v.clamp(0.0, 100.0) as f64]);
            prev = true;
        } else {
            prev = false;
        }
    }
    out
}

/// Plot stays inside UPPER_H (265 px) after the 24 px header and 4 px gap.
pub fn plot_extent(w: f32, h: f32) -> (f32, f32) {
    (w.max(160.0), h.clamp(170.0, 200.0))
}

#[allow(clippy::too_many_arguments)]
pub fn draw_gpu_plot(
    ui: &mut egui::Ui,
    span_min: f64,
    window: ChartWindow,
    show_vram: bool,
    show_engine: bool,
    vram_segments: &[Vec<[f64; 2]>],
    engine_segments: &[Vec<[f64; 2]>],
) {
    let (plot_w, plot_h) = plot_extent(ui.available_width(), ui.available_height());
    let response = Plot::new("gpu_vram_history_plot")
        .width(plot_w)
        .height(plot_h)
        .show_axes([false, false])
        .show_grid([true, true])
        .show_background(false)
        .auto_bounds(egui::Vec2b::new(false, false))
        .y_grid_spacer(|_| PERCENT_Y_GRID.to_vec())
        .x_grid_spacer(move |input| x_grid_spacer(span_min, &input))
        .include_x(-span_min)
        .include_x(0.0)
        .include_y(0.0)
        .include_y(100.0)
        .allow_boxed_zoom(false)
        .label_formatter(|_, value| {
            let when = if value.x.abs() < 0.01 {
                "now".to_owned()
            } else {
                format!("{:.1}m ago", -value.x)
            };
            format!("{when}: {:.1}%", value.y)
        })
        .show(ui, |pu| {
            lock_time_chart_bounds(pu, window.seconds() as usize, 100.0);
            if show_vram {
                for seg in vram_segments {
                    if seg.len() > 1 {
                        pu.line(
                            Line::new(PlotPoints::new(seg.clone()))
                                .color(colors::ACCENT_CYAN)
                                .width(2.2_f32)
                                .fill(0.0_f32),
                        );
                    } else {
                        pu.points(
                            Points::new(PlotPoints::new(seg.clone()))
                                .radius(3.0_f32)
                                .color(colors::ACCENT_CYAN),
                        );
                    }
                }
            }
            if show_engine {
                for seg in engine_segments {
                    if seg.len() > 1 {
                        pu.line(
                            Line::new(PlotPoints::new(seg.clone()))
                                .color(colors::ACCENT_PURPLE)
                                .width(2.0_f32),
                        );
                    } else {
                        pu.points(
                            Points::new(PlotPoints::new(seg.clone()))
                                .radius(3.0_f32)
                                .color(colors::ACCENT_PURPLE),
                        );
                    }
                }
            }
        });
    let frame = *response.transform.frame();
    draw_y_axis_hud_labels(ui, frame);
    draw_x_axis_hud_labels(ui, frame, window);
    let engine = flatten(engine_segments);
    let vram = flatten(vram_segments);
    draw_hud_scanner(ui, frame, span_min, &engine, &vram, show_engine, show_vram);
    let _ = draw_hud_legend(ui, frame);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vram_is_kept_when_gpu_counters_are_missing() {
        let rows = [(0.0, Some(42.0), None)];
        assert!(segments(&rows, 1.0, true).is_empty());
        let vram = segments(&rows, 1.0, false);
        assert_eq!(vram.len(), 1);
        assert_eq!(vram[0], vec![[0.0, 42.0]]);
    }

    #[test]
    fn single_point_segments_keep_one_sample() {
        let rows = [(5.0, None, Some(75.0)), (4.0, None, None), (0.0, None, Some(60.0))];
        let segs = segments(&rows, 5.0, true);
        assert_eq!(segs.len(), 2);
        assert!(segs.iter().all(|s| s.len() == 1));
    }

    #[test]
    fn plot_extent_never_collapses_or_overflows_upper_row() {
        assert!(plot_extent(0.0, 0.0).0 >= 160.0);
        assert!(plot_extent(900.0, 2000.0).1 <= 200.0);
    }
}
