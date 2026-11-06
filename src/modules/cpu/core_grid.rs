//! Adaptive per-core live telemetry grid (2–128+ logical threads).

use crate::modules::cpu::model::CpuSnapshot;
use crate::theme::{card_frame, colors};
use eframe::egui;

/// Renders the adaptive per-core grid.
pub fn render_core_grid(ui: &mut egui::Ui, cpu: &CpuSnapshot) {
    let per_core = &cpu.per_core_pct;
    let n = per_core.len();
    if n == 0 {
        return;
    }

    // Dynamic cols from available width so narrowing never clips C08/C09.
    let avail_w = ui.available_width().max(120.0);
    let cols = ((avail_w / 84.0).floor() as usize).clamp(4, 10);
    let cols = cols.min(n);

    card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            // Header
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

            // Grid layout — vertical ScrollArea when threads are high.
            let available_width = ui.available_width().max(120.0);
            let cell_width = (available_width - (cols - 1) as f32 * 8.0) / cols as f32;
            let cell_width = cell_width.clamp(48.0, 120.0);
            let cell_height = 48.0;

            let show_scroll = per_core.len() > cols * 4;
            let grid_fn = |ui: &mut egui::Ui| {
                egui::Grid::new("core_grid").num_columns(cols).spacing([8.0, 8.0]).show(ui, |ui| {
                    for (idx, &usage) in per_core.iter().enumerate() {
                        let is_hottest = idx == cpu.hottest_core_idx;
                        render_core_cell(ui, idx, usage, is_hottest, cell_width, cell_height);
                        if (idx + 1) % cols == 0 {
                            ui.end_row();
                        }
                    }
                    if per_core.len() % cols != 0 {
                        ui.end_row();
                    }
                });
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
    if is_hottest {
        resp.on_hover_text(format!(
            "Core C{:02}: {:.1}% (Hottest logical thread in current tick)",
            idx, usage
        ));
    } else {
        resp.on_hover_text(format!("Core C{:02}: {:.1}%", idx, usage));
    }

    // Cell background — idle cores dimmed.
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

    // Core label — idle dimmed.
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
        format!("C{:02}", idx),
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

    // Usage percentage text — idle dimmed.
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
        format!("{:.0}%", usage),
        egui::FontId::monospace(11.0),
        pct_color,
    );
}
