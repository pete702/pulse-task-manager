//! Adaptive per-core live telemetry grid with load spectrum and hottest-core pills.

use crate::modules::cpu::model::CpuSnapshot;
use crate::theme::{card_frame, colors};
use crate::ui::primitives::{mini_progress, pill_badge};
use eframe::egui;
use std::borrow::Cow;
use std::sync::OnceLock;

/// Adaptive column count: 2..10 columns for 4, 8, 16, 28 or 32+ logical cores.
pub(crate) fn compute_grid_cols(n: usize, avail_w: f32, spacing: f32) -> (usize, f32) {
    let w = if avail_w.is_finite() { avail_w.max(0.0) } else { 0.0 };
    let gap = if spacing.is_finite() { spacing.max(0.0) } else { 0.0 };
    let cols = (((w + 8.0) / 136.0).floor() as usize).clamp(2, 10).min(n.max(1)).max(1);
    let tile_w = ((w - gap * (cols.saturating_sub(1) as f32)) / cols as f32).max(40.0);
    (cols, tile_w)
}

fn core_label(idx: usize) -> Cow<'static, str> {
    static LABELS: OnceLock<Vec<String>> = OnceLock::new();
    let labels = LABELS.get_or_init(|| (0..256).map(|i| format!("C{i:02}")).collect());
    labels
        .get(idx)
        .map(|s| Cow::Borrowed(s.as_str()))
        .unwrap_or_else(|| Cow::Owned(format!("C{idx:02}")))
}

fn usage_label(usage: f32) -> Cow<'static, str> {
    static LABELS: OnceLock<Vec<String>> = OnceLock::new();
    let labels = LABELS.get_or_init(|| (0..=100).map(|i| format!("{i}%")).collect());
    let idx = if usage.is_finite() { usage.clamp(0.0, 100.0).round() as usize } else { 0 };
    labels
        .get(idx)
        .map(|s| Cow::Borrowed(s.as_str()))
        .unwrap_or_else(|| Cow::Owned(format!("{idx}%")))
}

/// P/E badge only on genuine hybrid CPUs; never a fake wattage/TDP reading.
fn core_type_badge(idx: usize) -> Option<&'static str> {
    if idx >= 64 {
        return None;
    }
    let (p_mask, e_mask) = crate::platform::hybrid_cores::query_hybrid_core_masks()?;
    let bit = 1usize << idx;
    if p_mask & bit != 0 {
        Some("P")
    } else if e_mask & bit != 0 {
        Some("E")
    } else {
        None
    }
}

/// Idle (<10%) / Active (10-50%) / Saturated (>50%) distribution.
fn render_load_spectrum(ui: &mut egui::Ui, cpu: &CpuSnapshot) {
    let mut idle = 0usize;
    let mut active = 0usize;
    let mut saturated = 0usize;
    for &v in cpu.per_core_pct.iter() {
        if v < 10.0 {
            idle += 1
        } else if v <= 50.0 {
            active += 1
        } else {
            saturated += 1
        }
    }
    ui.horizontal(|ui| {
        pill_badge(ui, "LOAD DISTRIBUTION", colors::TEXT_SECONDARY);
        pill_badge(ui, &format!("Idle <10%: {idle}"), colors::EMERALD);
        pill_badge(ui, &format!("Active 10-50%: {active}"), colors::ACCENT_CYAN);
        pill_badge(ui, &format!("Saturated >50%: {saturated}"), colors::ACCENT_AMBER);
    });
}

/// Four hottest logical threads shown as compact pills.
fn render_top_cores(ui: &mut egui::Ui, cpu: &CpuSnapshot) {
    let mut hottest: Vec<(usize, f32)> = cpu.per_core_pct.iter().copied().enumerate().collect();
    hottest.sort_by(|a, b| b.1.total_cmp(&a.1));
    hottest.truncate(4);
    ui.horizontal_wrapped(|ui| {
        pill_badge(ui, "TOP 4 HOTTEST", colors::TEXT_SECONDARY);
        for (idx, usage) in hottest {
            let col = crate::theme::load_color(usage);
            let kind = core_type_badge(idx).unwrap_or("");
            pill_badge(
                ui,
                &format!(
                    "{} {}{} {:.0}%",
                    core_label(idx),
                    kind,
                    if kind.is_empty() { "" } else { " " },
                    usage
                ),
                col,
            );
        }
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
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let hot = is_hottest;
    let col = crate::theme::load_color(usage);
    let bg = if hot { colors::ELEVATED } else { colors::BG_CARD };
    let rounded = egui::Rounding::same(colors::PILL_RADIUS);
    ui.painter().rect_filled(rect, rounded, bg);
    ui.painter().rect_stroke(
        rect,
        rounded,
        egui::Stroke::new(
            if hot { 2.0 } else { 1.0_f32 },
            if hot { colors::ACCENT_CYAN } else { colors::BORDER },
        ),
    );
    response.on_hover_text(format!("Core {}: {:.1}%", core_label(idx), usage));
    let kind = core_type_badge(idx).map(|k| format!(" {k}")).unwrap_or_default();
    ui.painter().text(
        egui::pos2(rect.left() + 6.0, rect.top() + 8.0),
        egui::Align2::LEFT_CENTER,
        format!("{}{}", core_label(idx), kind),
        egui::FontId::monospace(10.0),
        if hot { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY },
    );
    ui.painter().text(
        egui::pos2(rect.right() - 6.0, rect.top() + 8.0),
        egui::Align2::RIGHT_CENTER,
        usage_label(usage),
        egui::FontId::monospace(11.0),
        col,
    );
    let bar = egui::Rect::from_min_max(
        egui::pos2(rect.left() + 6.0, rect.top() + 17.0),
        egui::pos2(rect.right() - 6.0, rect.bottom() - 5.0),
    );
    mini_progress(ui, bar, usage / 100.0, col);
}

pub fn render_core_grid(ui: &mut egui::Ui, cpu: &CpuSnapshot) {
    let per_core = &cpu.per_core_pct;
    if per_core.is_empty() {
        return;
    }
    let avail_w = ui.available_width().max(120.0);
    let spacing = 6.0_f32;
    let (cols, tile_w) = compute_grid_cols(per_core.len(), avail_w, spacing);
    card_frame().show(ui, |ui| {
        ui.horizontal(|ui| {
            pill_badge(ui, "PER-CORE LIVE TELEMETRY GRID", colors::ACCENT_CYAN);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                pill_badge(
                    ui,
                    &format!("{} Logical Threads", per_core.len()),
                    colors::TEXT_SECONDARY,
                );
            });
        });
        ui.add_space(6.0);
        render_load_spectrum(ui, cpu);
        ui.add_space(4.0);
        render_top_cores(ui, cpu);
        ui.add_space(8.0);
        let grid_fn = |ui: &mut egui::Ui| {
            egui::Grid::new("core_grid").num_columns(cols).spacing([spacing, 8.0]).show(ui, |ui| {
                for (idx, &usage) in per_core.iter().enumerate() {
                    render_core_cell(ui, idx, usage, idx == cpu.hottest_core_idx, tile_w, 44.0);
                    if (idx + 1) % cols == 0 {
                        ui.end_row();
                    }
                }
                if !per_core.len().is_multiple_of(cols) {
                    ui.end_row();
                }
            });
        };
        if per_core.len() > cols * 4 {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(320.0)
                .show(ui, grid_fn);
        } else {
            grid_fn(ui);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{compute_grid_cols, core_label, usage_label};

    #[test]
    fn core_grid_helpers() {
        assert_eq!(core_label(0).as_ref(), "C00");
        assert_eq!(core_label(256).as_ref(), "C256");
        for &(v, s) in
            &[(0.0, "0%"), (100.0, "100%"), (f32::NAN, "0%"), (-5.0, "0%"), (150.0, "100%")]
        {
            assert_eq!(usage_label(v).as_ref(), s);
        }
    }

    #[test]
    fn adaptive_columns_scale_with_width() {
        for n in [4usize, 8, 16, 28, 32] {
            let (cols, tile_w) = compute_grid_cols(n, 960.0, 6.0);
            assert!((2..=10).contains(&cols));
            assert!(tile_w >= 40.0);
            assert!(cols <= n);
        }
        let (c1, _) = compute_grid_cols(32, 400.0, 6.0);
        let (c2, _) = compute_grid_cols(32, 1900.0, 6.0);
        assert!(c1 < c2);
    }
}
