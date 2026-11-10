//! Adaptive per-core live telemetry grid (2-128+ logical threads).

use crate::modules::cpu::model::CpuSnapshot;
use crate::theme::{card_frame, colors};
use eframe::egui;
use std::borrow::Cow;
use std::sync::OnceLock;

pub(crate) fn compute_grid_cols(n: usize, avail_w: f32, spacing: f32) -> (usize, f32) {
    let avail_w = if avail_w.is_finite() { avail_w.max(0.0) } else { 0.0 };
    let spacing = if spacing.is_finite() { spacing.max(0.0) } else { 0.0 };
    let max_fit = ((avail_w + spacing) / (68.0 + spacing)).floor().max(1.0) as usize;
    let cols = ((avail_w / 88.0).floor() as usize).clamp(4, 14).min(max_fit).min(n.max(1)).max(1);
    let tile_w = ((avail_w - spacing * (cols.saturating_sub(1) as f32)) / (cols as f32)).max(68.0);
    (cols, tile_w)
}

/// Renders the adaptive per-core grid.
pub fn render_core_grid(ui: &mut egui::Ui, cpu: &CpuSnapshot) {
    let per_core = &cpu.per_core_pct;
    let n = per_core.len();
    if n == 0 {
        return;
    }

    let avail_w = ui.available_width().max(120.0);
    let spacing = 6.0_f32;
    let (cols, tile_w) = compute_grid_cols(n, avail_w, spacing);

    card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("PER-CORE LIVE TELEMETRY GRID")
                        .monospace()
                        .size(13.0)
                        .color(colors::TEXT_PRIMARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{} Logical Threads", n))
                            .monospace()
                            .size(10.0)
                            .color(colors::TEXT_SECONDARY),
                    );
                });
            });
            ui.add_space(8.0);
            let cell_width = tile_w;
            let cell_height = 48.0;

            let show_scroll = per_core.len() > cols * 4;
            let grid_fn = |ui: &mut egui::Ui| {
                egui::Grid::new("core_grid").num_columns(cols).spacing([spacing, 8.0]).show(
                    ui,
                    |ui| {
                        for (idx, &usage) in per_core.iter().enumerate() {
                            let is_hottest = idx == cpu.hottest_core_idx;
                            render_core_cell(ui, idx, usage, is_hottest, cell_width, cell_height);
                            if (idx + 1) % cols == 0 {
                                ui.end_row();
                            }
                        }
                        if !per_core.len().is_multiple_of(cols) {
                            ui.end_row();
                        }
                    },
                );
            };
            if show_scroll {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_height(320.0)
                    .show(ui, grid_fn);
            } else {
                grid_fn(ui);
            }
        });
    });
}

fn core_label(idx: usize) -> Cow<'static, str> {
    static LABELS: OnceLock<Vec<String>> = OnceLock::new();
    let labels = LABELS.get_or_init(|| (0..256).map(|i| format!("C{i:02}")).collect());
    if let Some(s) = labels.get(idx) {
        Cow::Borrowed(s.as_str())
    } else {
        Cow::Owned(format!("C{idx:02}"))
    }
}

fn usage_label(usage: f32) -> Cow<'static, str> {
    static LABELS: OnceLock<Vec<String>> = OnceLock::new();
    let labels = LABELS.get_or_init(|| (0..=100).map(|i| format!("{i}%")).collect());
    let idx = if usage.is_finite() { usage.clamp(0.0, 100.0).round() as usize } else { 0 };
    if let Some(s) = labels.get(idx) {
        Cow::Borrowed(s.as_str())
    } else {
        Cow::Owned(format!("{idx}%"))
    }
}

fn render_core_cell(
    ui: &mut egui::Ui,
    idx: usize,
    usage: f32,
    is_hottest: bool,
    width: f32,
    height: f32,
) {
    let is_idle = usage < 0.5;
    let (bar_color, border_color) = if usage >= 85.0 {
        (
            egui::Color32::from_rgb(239, 68, 68),
            egui::Color32::from_rgba_unmultiplied(239, 68, 68, 180),
        )
    } else if usage >= 60.0 {
        (
            egui::Color32::from_rgb(245, 158, 11),
            egui::Color32::from_rgba_unmultiplied(245, 158, 11, 180),
        )
    } else if is_idle {
        (
            egui::Color32::from_rgba_unmultiplied(92, 104, 122, 120),
            egui::Color32::from_rgba_unmultiplied(51, 65, 85, 70),
        )
    } else {
        (colors::ACCENT_CYAN, egui::Color32::from_rgba_unmultiplied(8, 145, 178, 180))
    };

    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    if resp.hovered() {
        if is_hottest {
            resp.on_hover_text(format!(
                "Core C{:02}: {:.1}% (Hottest logical thread in current tick)",
                idx, usage
            ));
        } else {
            resp.on_hover_text(format!("Core C{:02}: {:.1}%", idx, usage));
        }
    }

    // Cell background - idle cores dimmed.
    let bg_color = if is_idle && !is_hottest {
        egui::Color32::from_rgba_unmultiplied(10, 12, 18, 180)
    } else if is_hottest {
        egui::Color32::from_rgba_unmultiplied(30, 41, 59, 240)
    } else {
        egui::Color32::from_rgba_unmultiplied(15, 23, 42, 220)
    };
    let stroke_color = if is_hottest {
        border_color
    } else if is_idle {
        egui::Color32::from_rgba_unmultiplied(51, 65, 85, 60)
    } else {
        egui::Color32::from_rgba_unmultiplied(51, 65, 85, 140)
    };
    let stroke_width = if is_hottest { 2.0_f32 } else { 1.0_f32 };

    ui.painter().rect_filled(rect, egui::Rounding::same(4.0), bg_color);
    ui.painter().rect_stroke(
        rect,
        egui::Rounding::same(4.0),
        egui::Stroke::new(stroke_width, stroke_color),
    );

    // Core label - idle dimmed.
    let label_y = rect.top() + 4.0;
    let lbl_color = if is_hottest {
        colors::ACCENT_CYAN
    } else if is_idle {
        egui::Color32::from_rgba_unmultiplied(92, 104, 122, 160)
    } else {
        colors::TEXT_SECONDARY
    };
    ui.painter().text(
        egui::pos2(rect.center().x, label_y),
        egui::Align2::CENTER_TOP,
        core_label(idx),
        egui::FontId::monospace(10.0),
        lbl_color,
    );

    // Precision bar (small, inside cell)
    let bar_rect = egui::Rect::from_min_max(
        egui::pos2(rect.left() + 4.0, rect.top() + 20.0),
        egui::pos2(rect.right() - 4.0, rect.bottom() - 4.0),
    );
    let ratio = (usage / 100.0).clamp(0.0, 1.0);

    // Track
    let track_color = egui::Color32::from_rgba_unmultiplied(15, 23, 42, 220);
    let border_track = egui::Color32::from_rgba_unmultiplied(51, 65, 85, 140);
    ui.painter().rect_filled(bar_rect, egui::Rounding::same(2.0), track_color);
    ui.painter().rect_stroke(
        bar_rect,
        egui::Rounding::same(2.0),
        egui::Stroke::new(1.0_f32, border_track),
    );

    // Fill
    if ratio > 0.002 {
        let fill_w = (bar_rect.width() * ratio).max(2.0).min(bar_rect.width());
        let fill_rect = egui::Rect::from_min_max(
            bar_rect.min,
            egui::pos2(bar_rect.min.x + fill_w, bar_rect.max.y),
        );
        let fill_color =
            egui::Color32::from_rgba_unmultiplied(bar_color.r(), bar_color.g(), bar_color.b(), 215);
        ui.painter().rect_filled(fill_rect, egui::Rounding::same(2.0), fill_color);

        // Leading-edge cap
        let cap_w = 2.0_f32.min(fill_w);
        let cap_rect = egui::Rect::from_min_max(
            egui::pos2(fill_rect.max.x - cap_w, fill_rect.min.y),
            fill_rect.max,
        );
        let cap_color = egui::Color32::from_rgb(240, 253, 250);
        ui.painter().rect_filled(cap_rect, egui::Rounding::ZERO, cap_color);
    }

    // Usage percentage text - idle dimmed.
    let pct_y = rect.bottom() - 10.0;
    let pct_color = if is_hottest {
        bar_color
    } else if is_idle {
        egui::Color32::from_rgba_unmultiplied(92, 104, 122, 170)
    } else {
        colors::TEXT_PRIMARY
    };
    ui.painter().text(
        egui::pos2(rect.center().x, pct_y),
        egui::Align2::CENTER_BOTTOM,
        usage_label(usage),
        egui::FontId::monospace(11.0),
        pct_color,
    );
}

#[cfg(test)]
mod tests {
    use super::{compute_grid_cols, core_label, usage_label};

    #[test]
    fn core_grid_helpers() {
        assert_eq!(core_label(0).as_ref(), "C00");
        assert_eq!(core_label(255).as_ref(), "C255");
        assert_eq!(core_label(256).as_ref(), "C256");
        for &(v, s) in
            &[(0.0, "0%"), (100.0, "100%"), (f32::NAN, "0%"), (-5.0, "0%"), (150.0, "100%")]
        {
            assert_eq!(usage_label(v).as_ref(), s);
        }
        for &avail_w in &[100.0_f32, 250.0, 860.0, 1920.0] {
            let (cols, tile_w) = compute_grid_cols(16, avail_w, 6.0);
            let total = cols as f32 * tile_w + cols.saturating_sub(1) as f32 * 6.0;
            assert!(total <= avail_w + 0.01);
            if avail_w >= 68.0 {
                assert!(tile_w >= 68.0);
            }
        }
    }
}
