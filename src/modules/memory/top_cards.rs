//! Memory top cards: Physical RAM + Commit Charge / Kernel Pools.

use crate::app_persist::MIN_WINDOW_WIDTH;
use crate::modules::memory::formatters::format_bytes;
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::model::MemorySnapshot;
use crate::theme::{card_frame, colors, load_color};
use crate::ui::primitives::{cyber_accent, draw_cyber_corner_brackets, draw_segmented_led_bar};
use eframe::egui;

const TOP_CARDS_SIDEBAR_OFFSET: f32 = 40.0;

pub fn render_memory_top_cards(ui: &mut egui::Ui, m: &MemorySnapshot, h: &MemoryHistory) {
    let avail = ui.available_width();
    if avail < MIN_WINDOW_WIDTH - TOP_CARDS_SIDEBAR_OFFSET {
        ui.vertical(|ui| {
            render_ram_top_card(ui, m, h);
            ui.add_space(10.0);
            render_commit_top_card(ui, m);
        });
    } else {
        ui.columns(2, |c| {
            render_ram_top_card(&mut c[0], m, h);
            render_commit_top_card(&mut c[1], m);
        });
    }
}

fn render_ram_top_card(ui: &mut egui::Ui, m: &MemorySnapshot, h: &MemoryHistory) {
    let p = m.ram_usage_percent();
    let c = load_color(p);
    let resp = card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new("Physical Memory (RAM)")
                            .font(egui::FontId::proportional(13.5))
                            .color(colors::TEXT_PRIMARY),
                    )
                    .wrap(false),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{:.1}%", p)).monospace().size(28.0).color(c),
                    );
                });
            });
            ui.add_space(8.0);
            let t = m.ram_total_bytes.max(1) as f64;
            let iu = (m.ram_used_bytes as f64).clamp(0.0, t);
            let av = (m.ram_available_bytes as f64).clamp(0.0, (t - iu).max(0.0));
            // Windows often reports free == available; derive cache from the
            // same value printed in "OS Cache / Peak" and split Available once.
            let ca = (m.ram_cached_bytes() as f64).min(av).max(0.0);
            let fr = (av - ca).max(0.0);
            let pf = |v: f64| (v / t * 100.0) as f32;
            let parts = [iu, ca, fr];
            let tints = [load_color(pf(iu)), colors::ACCENT_CYAN, colors::TEXT_MUTED];
            let names = ["In Use", "OS Cache", "Free"];
            let bw = ui.available_width().max(80.0);
            let (r, rs) = ui.allocate_exact_size(egui::vec2(bw, 12.0), egui::Sense::hover());
            rs.on_hover_text(cached_memory_tooltip());
            ui.painter().rect_filled(r, egui::Rounding::same(3.0), colors::ELEVATED);
            let mut x = r.left();
            for (i, &v) in parts.iter().enumerate() {
                let end = (x + r.width() * (v / t).clamp(0.0, 1.0) as f32).min(r.right());
                if end > x {
                    fill_seg(ui, r, x, end, tints[i]);
                }
                x = end;
            }
            ui.painter().rect_stroke(
                r,
                egui::Rounding::same(3.0),
                egui::Stroke::new(1.0_f32, colors::BORDER),
            );
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                for (i, &v) in parts.iter().enumerate() {
                    crate::ui::primitives::pill_badge(
                        ui,
                        &format!("{} {:.1} GiB", names[i], v / 1_073_741_824.0),
                        tints[i],
                    );
                }
            });
            ui.add_space(10.0);
            egui::Grid::new("ram_top_grid").num_columns(3).spacing([16.0, 6.0]).show(ui, |ui| {
                metric_cell(
                    ui,
                    "Used / Total",
                    &format!("{:.2} / {:.2} GiB", m.ram_used_gib(), m.ram_total_gib()),
                    colors::TEXT_PRIMARY,
                );
                metric_cell(
                    ui,
                    "Available RAM",
                    &format!(
                        "{:.2} GiB ({:.1}%)",
                        m.ram_available_gib(),
                        (m.ram_available_bytes as f64 / m.ram_total_bytes.max(1) as f64) * 100.0
                    ),
                    colors::TEXT_PRIMARY,
                );
                metric_cell(
                    ui,
                    "OS Cache / Peak",
                    &format!("{:.2} GiB | Peak {:.1}%", m.ram_cached_gib(), h.peak_ram_percent),
                    colors::ACCENT_CYAN,
                );
                ui.end_row();
            });
        });
    });
    draw_cyber_corner_brackets(ui, resp.response.rect, cyber_accent(p));
}

fn render_commit_top_card(ui: &mut egui::Ui, m: &MemorySnapshot) {
    let v = m.kernel_metrics_valid;
    let cp = if v && m.commit_limit_bytes > 0 {
        ((m.commit_total_bytes as f64 / m.commit_limit_bytes as f64 * 100.0) as f32)
            .clamp(0.0, 100.0)
    } else {
        0.0
    };
    let c = load_color(cp);
    let resp2 = card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let hdr = egui::Label::new(
                    egui::RichText::new("Commit Charge & Kernel Pools").font(egui::FontId::proportional(13.5)).color(colors::TEXT_PRIMARY),
                )
                .wrap(false);
                ui.add(hdr).on_hover_text(
                    "Commit Charge = virtual memory committed by programs (RAM + Pagefile). Kernel Pools = memory used by Windows core (paged/non-paged).",
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!("{:.1}%", cp)).monospace().size(28.0).color(c));
                });
            });
            ui.add_space(8.0);
            let pv = if v { cp / 100.0 } else { 0.0 };
            let bw = ui.available_width().max(80.0);
            let (r, rs) = ui.allocate_exact_size(egui::vec2(bw, 16.0), egui::Sense::hover());
            rs.on_hover_text("Virtual Commit Charge vs system Commit Limit (RAM + Pagefile). >85% = OOM risk.");
            draw_segmented_led_bar(ui, r, pv, 28, colors::ACCENT_PURPLE);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Pagefile Swap:").font(egui::FontId::proportional(11.0)).color(colors::TEXT_SECONDARY));
                ui.add(egui::Label::new(egui::RichText::new(format!("{:.1}%", m.swap_usage_percent())).monospace().size(11.0).color(colors::TEXT_PRIMARY)).wrap(false));
                ui.label(egui::RichText::new("|").font(egui::FontId::proportional(11.0)).color(colors::TEXT_SECONDARY));
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(format!(
                            "{} Procs / {} Threads / {} Handles",
                            m.system_processes, m.system_threads, m.system_handles
                        ))
                            .font(egui::FontId::proportional(11.0))
                            .color(colors::TEXT_PRIMARY),
                    )
                    .wrap(false),
                );
            });
            ui.add_space(10.0);
            egui::Grid::new("commit_top_grid").num_columns(3).spacing([16.0, 6.0]).show(ui, |ui| {
                metric_cell(ui, "Commit / Limit", &format!("{:.2} / {:.2} GiB", m.commit_total_gib(), m.commit_limit_gib()), colors::ACCENT_PURPLE);
                metric_cell(ui, "Kernel Pools", &format!("{:.2} GiB ({:.0}/{:.0}M)", m.kernel_total_mib() / 1024.0, m.kernel_paged_mib(), m.kernel_nonpaged_mib()), colors::ACCENT_AMBER);
                metric_cell(ui, "Available Commit", &format_bytes(m.commit_limit_bytes.saturating_sub(m.commit_total_bytes)), colors::TEXT_PRIMARY);
                ui.end_row();
            });
        });
    });
    draw_cyber_corner_brackets(ui, resp2.response.rect, cyber_accent(cp));
}

fn metric_cell(ui: &mut egui::Ui, cap: &str, val: &str, vc: egui::Color32) {
    ui.vertical(|ui| {
        ui.add(
            egui::Label::new(
                egui::RichText::new(cap)
                    .font(egui::FontId::proportional(11.0))
                    .color(colors::TEXT_SECONDARY),
            )
            .wrap(false),
        );
        ui.add(
            egui::Label::new(egui::RichText::new(val).monospace().size(12.0).color(vc)).wrap(false),
        );
    });
}

fn fill_seg(ui: &egui::Ui, r: egui::Rect, x0: f32, x1: f32, c: egui::Color32) {
    ui.painter().rect_filled(
        egui::Rect::from_min_max(egui::pos2(x0, r.top()), egui::pos2(x1, r.bottom())),
        egui::Rounding::same(3.0),
        c,
    );
}

pub(crate) fn cached_memory_tooltip() -> &'static str {
    "OS file cache (SystemCache) -- may differ from Task Manager"
}

#[cfg(test)]
mod tests {
    use super::cached_memory_tooltip;

    #[test]
    fn cached_memory_tooltip_text() {
        assert_eq!(
            cached_memory_tooltip(),
            "OS file cache (SystemCache) -- may differ from Task Manager"
        );
    }
}
