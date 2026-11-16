//! Interactive resizable column engine for the CPU process table.
use crate::modules::cpu::model::ProcessCpuItem;
use eframe::egui;
use std::sync::{Mutex, OnceLock};

static PERSISTED_WIDTHS: OnceLock<Mutex<Option<[f32; 7]>>> = OnceLock::new();
pub fn get_persisted_col_widths() -> Option<[f32; 7]> {
    PERSISTED_WIDTHS.get().and_then(|m| *m.lock().unwrap_or_else(|e| e.into_inner()))
}
pub fn set_persisted_col_widths(w: Option<[f32; 7]>) {
    *PERSISTED_WIDTHS.get_or_init(|| Mutex::new(None)).lock().unwrap_or_else(|e| e.into_inner()) =
        w;
}
pub const MIN_COL_W: [f32; 7] = [52.0, 125.0, 60.0, 66.0, 126.0, 90.0, 82.0];
const MIN_SUM: f32 = 601.0;
pub fn default_widths(avail_w: f32) -> [f32; 7] {
    let name = (avail_w.max(MIN_SUM) - 526.0).max(MIN_COL_W[1]);
    let mut w = [58.0, name, 64.0, 74.0, 144.0, 98.0, 88.0];
    fit_column_widths(&mut w, avail_w);
    w
}
fn clamp_min(w: &mut [f32; 7]) {
    for (a, b) in w.iter_mut().zip(MIN_COL_W.iter()) {
        if !a.is_finite() || *a < *b {
            *a = *b;
        }
    }
}
pub fn fit_column_widths(widths: &mut [f32; 7], avail_w: f32) {
    clamp_min(widths);
    let target = avail_w.max(MIN_SUM);
    let diff = target - widths.iter().sum::<f32>();
    let slack_sum: f32 = widths.iter().zip(MIN_COL_W.iter()).map(|(w, m)| (w - m).max(0.0)).sum();
    if slack_sum > 0.0 {
        for i in 0..7 {
            widths[i] += diff * (widths[i] - MIN_COL_W[i]).max(0.0) / slack_sum;
        }
    } else if diff > 0.0 {
        widths[1] += diff;
    }
    clamp_min(widths);
    widths[1] += target - widths.iter().sum::<f32>();
    if widths[1] < MIN_COL_W[1] {
        widths[1] = MIN_COL_W[1];
    }
    widths[1] += target - widths.iter().sum::<f32>();
    clamp_min(widths);
}
pub fn compute_col_x(left: f32, widths: &[f32; 7]) -> [f32; 7] {
    let mut x = [0.0_f32; 7];
    x[0] = left;
    for i in 1..7 {
        x[i] = x[i - 1] + widths[i - 1];
    }
    x
}
pub fn apply_drag(widths: &mut [f32; 7], i: usize, dx: f32) {
    if i >= 6 || !dx.is_finite() || dx == 0.0 {
        return;
    }
    if dx > 0.0 {
        let mut need = dx;
        for k in (i + 1)..7 {
            if need <= 0.0 {
                break;
            }
            let t = (widths[k] - MIN_COL_W[k]).max(0.0).min(need);
            widths[k] -= t;
            need -= t;
        }
        widths[i] += dx - need;
    } else {
        let mut need = -dx;
        let mut freed = 0.0;
        for k in (0..=i).rev() {
            if need <= 0.0 {
                break;
            }
            let t = (widths[k] - MIN_COL_W[k]).max(0.0).min(need);
            widths[k] -= t;
            freed += t;
            need -= t;
        }
        widths[i + 1] += freed;
    }
    clamp_min(widths);
}
pub fn autofit_widths(widths: &mut [f32; 7], i: usize, rows: &[ProcessCpuItem]) {
    if i >= 6 {
        return;
    }
    let total: f32 = widths.iter().sum();
    if i == 1 {
        let longest = rows.iter().map(|r| r.name.chars().count()).max().unwrap_or(0) as f32;
        let want0 = (longest * 7.1 + 14.0).max(MIN_COL_W[1]);
        let others_min =
            MIN_COL_W.iter().enumerate().filter(|(k, _)| *k != 1).map(|(_, v)| *v).sum::<f32>();
        let want = want0.min((total - others_min).max(MIN_COL_W[1]));
        let delta = want - widths[1];
        widths[1] = want;
        if delta > 0.0 {
            let mut need = delta;
            for k in [6usize, 5, 4, 3, 2, 0] {
                if need <= 0.0 {
                    break;
                }
                let t = (widths[k] - MIN_COL_W[k]).max(0.0).min(need);
                widths[k] -= t;
                need -= t;
            }
        } else if delta < 0.0 {
            let share = -delta / 6.0;
            for k in [0usize, 2, 3, 4, 5, 6] {
                widths[k] += share;
            }
        }
    } else {
        let freed = (widths[i] - MIN_COL_W[i]).max(0.0);
        if freed > 0.0 {
            widths[i] = MIN_COL_W[i];
            widths[1] += freed;
        }
    }
    clamp_min(widths);
}
pub fn truncate_name(name: &str, w: f32) -> std::borrow::Cow<'_, str> {
    if w < 14.0 {
        return std::borrow::Cow::Borrowed("...");
    }
    let m = (((w - 14.0) / 7.1).floor() as usize).max(12);
    if name.chars().count() <= m {
        return std::borrow::Cow::Borrowed(name);
    }
    let mut s: String = name.chars().take(m.saturating_sub(1)).collect();
    s.push('~');
    std::borrow::Cow::Owned(s)
}
pub fn col_widths(ui: &mut egui::Ui, avail_w: f32) -> [f32; 7] {
    let id = egui::Id::new("pulse_cpu_table_col_widths");
    let mut w = ui.ctx().data_mut(|d| {
        *d.get_temp_mut_or_insert_with(id, || {
            get_persisted_col_widths().unwrap_or_else(|| default_widths(avail_w))
        })
    });
    fit_column_widths(&mut w, avail_w);
    ui.ctx().data_mut(|d| {
        let prev: Option<[f32; 7]> = d.get_temp(id);
        if prev != Some(w) {
            d.insert_temp(id, w);
        }
    });
    w
}
pub fn draw_separators(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    widths: &mut [f32; 7],
    rows: &[ProcessCpuItem],
) {
    let col_x = compute_col_x(rect.left(), widths);
    for i in 0..6 {
        let bx = col_x[i + 1];
        let resp = ui.interact(
            egui::Rect::from_min_max(
                egui::pos2(bx - 2.0, rect.top()),
                egui::pos2(bx + 2.0, rect.bottom()),
            ),
            egui::Id::new("pulse_cpu_col_div").with(i),
            egui::Sense::click_and_drag(),
        );
        let hot = resp.hovered() || resp.dragged();
        if hot {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }
        let dx = resp.drag_delta().x;
        let mut changed = false;
        if dx != 0.0 && resp.dragged() {
            apply_drag(widths, i, dx);
            changed = true;
        }
        if resp.double_clicked() {
            autofit_widths(widths, i, rows);
            changed = true;
        }
        if changed {
            set_persisted_col_widths(Some(*widths));
            ui.ctx().request_repaint();
        }
        let (col, sw) = if hot {
            (crate::theme::colors::ACCENT_CYAN, 1.75_f32)
        } else {
            (crate::theme::colors::BORDER, 1.0_f32)
        };
        ui.painter().line_segment(
            [egui::pos2(bx, rect.top()), egui::pos2(bx, rect.bottom())],
            egui::Stroke::new(sw, col),
        );
    }
    ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("pulse_cpu_table_col_widths"), *widths));
}
#[cfg(test)]
mod tests {
    use super::*;
    fn check(w: &[f32; 7], avail: f32) {
        for i in 0..7 {
            assert!(w[i] >= MIN_COL_W[i] - 0.001, "col {i} below min");
        }
        assert!((w.iter().sum::<f32>() - avail.max(MIN_SUM)).abs() < 0.05);
    }
    fn assert_drag_sep1_left() {
        let mut w = default_widths(1000.0);
        fit_column_widths(&mut w, 1000.0);
        let before = w;
        apply_drag(&mut w, 1, -40.0);
        let freed = (before[0] - w[0]) + (before[1] - w[1]);
        assert!((w[2] - before[2] - freed).abs() < 0.01);
        assert_eq!([w[3], w[4], w[5], w[6]], [before[3], before[4], before[5], before[6]]);
    }
    fn assert_width_clamp_and_narrow_truncate() {
        for avail in [601.0_f32, 860.0, 1200.0, 1920.0] {
            let mut w = default_widths(avail);
            fit_column_widths(&mut w, avail);
            let sum: f32 = w.iter().sum();
            assert!((sum - avail.max(MIN_SUM)).abs() < 0.1);
            for i in 0..7 {
                assert!(w[i] >= MIN_COL_W[i] - 0.001);
            }
        }
        assert_eq!(truncate_name("hello", 10.0).as_ref(), "...");
        assert_eq!(truncate_name("hello", 0.0).as_ref(), "...");
        assert_eq!(truncate_name("hello", 13.9).as_ref(), "...");
    }
    #[test]
    fn engine_invariants() {
        let mut w = default_widths(860.0);
        fit_column_widths(&mut w, 860.0);
        check(&w, 860.0);
        let mut w = default_widths(1920.0);
        fit_column_widths(&mut w, 1920.0);
        check(&w, 1920.0);
        let mut w = default_widths(1000.0);
        for (i, dx) in [(0, 20.0), (1, 30.0), (3, -25.0), (5, 40.0), (2, -50.0), (1, -25.0)] {
            apply_drag(&mut w, i, dx);
            check(&w, 1000.0);
        }
        let mut w = default_widths(860.0);
        fit_column_widths(&mut w, 1200.0);
        check(&w, 1200.0);
        let x = compute_col_x(100.0, &w);
        assert!(x[0] == 100.0);
        for i in 0..6 {
            assert!((x[i + 1] - (x[i] + w[i])).abs() < 0.001);
        }
        let (before0, before1, before2) = (w[0], w[1], w[2]);
        let before_rest = [w[3], w[4], w[5], w[6]];
        apply_drag(&mut w, 1, -40.0);
        let freed = (before0 - w[0]) + (before1 - w[1]);
        assert_eq!([w[3], w[4], w[5], w[6]], before_rest);
        assert!((w[2] - before2 - freed).abs() < 0.01);
        assert_eq!(truncate_name("abc", 260.0).as_ref(), "abc");
    }
    #[test]
    fn drag_separator_1_left_gives_to_immediate_neighbor() {
        assert_drag_sep1_left();
    }
    #[test]
    fn width_clamp_and_narrow_truncate() {
        assert_width_clamp_and_narrow_truncate();
    }
}
