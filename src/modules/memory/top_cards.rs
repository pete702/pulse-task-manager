//! Memory top cards: Physical RAM + Commit Charge / Kernel Pools.

use crate::modules::memory::formatters::format_bytes;
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::model::MemorySnapshot;
use crate::theme::{card_frame, colors, load_color};
use crate::ui::primitives::{
    cyber_accent, draw_cyber_corner_brackets, draw_segmented_led_bar, draw_tactical_ticks,
};
use eframe::egui;

pub fn render_memory_top_cards(ui: &mut egui::Ui, m: &MemorySnapshot, h: &MemoryHistory) {
    let avail = ui.available_width();
    if avail < 880.0 {
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
                        egui::RichText::new("PHYSICAL MEMORY (RAM)")
                            .monospace()
                            .size(13.0)
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
            let ca = (m.ram_cached_bytes() as f64).clamp(0.0, av);
            let fr = (av - ca).max(0.0);
            let pf = |v: f64| (v / t * 100.0) as f32;
            let (ip, cp, fp) = (pf(iu), pf(ca), pf(fr));
            let bw = ui.available_width().max(80.0);
            let (r, rs) = ui.allocate_exact_size(egui::vec2(bw, 12.0), egui::Sense::hover());
            rs.on_hover_text("RAM composition: In Use | Windows File & System Cache | Free");
            ui.painter().rect_filled(
                r,
                egui::Rounding::same(3.0),
                egui::Color32::from_rgb(0x1E, 0x22, 0x30),
            );
            let (iw, cw, fw) = segment_widths(r.width(), iu, ca, fr, t);
            let (x0, x1) = (r.left(), r.left() + iw);
            let x2 = (x1 + cw).min(r.right());
            let x3 = (x2 + fw).min(r.right());
            if iw > 0.0 {
                fill_seg(ui, r, x0, x1, load_color(ip));
            }
            if cw > 0.0 {
                fill_seg(ui, r, x1, x2, egui::Color32::from_rgb(0x00, 0x88, 0xAA));
            }
            if fw > 0.0 {
                fill_seg(ui, r, x2, x3, egui::Color32::from_rgb(0x1E, 0x22, 0x30));
            }

            ui.painter().rect_stroke(
                r,
                egui::Rounding::same(3.0),
                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(35, 48, 68)),
            );
            draw_tactical_ticks(ui, r);
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                legend_chip(
                    ui,
                    load_color(ip),
                    &format!("In Use: {} ({:.0}%)", format_bytes(m.ram_used_bytes), ip),
                );
                legend_chip(
                    ui,
                    egui::Color32::from_rgb(0x00, 0x88, 0xAA),
                    &format!("Cached (OS): {} ({:.0}%)", format_bytes(m.ram_cached_bytes()), cp),
                );
                legend_chip(
                    ui,
                    colors::TEXT_SECONDARY,
                    &format!("Free: {} ({:.0}%)", format_bytes(fr as u64), fp),
                );
            });
            ui.add_space(10.0);
            egui::Grid::new("ram_top_grid").num_columns(3).spacing([16.0, 6.0]).show(ui, |ui| {
                metric_cell(
                    ui,
                    "USED / TOTAL",
                    &format!("{:.2} / {:.2} GiB", m.ram_used_gib(), m.ram_total_gib()),
                    colors::TEXT_PRIMARY,
                );
                metric_cell(
                    ui,
                    "AVAILABLE RAM",
                    &format!(
                        "{:.2} GiB ({:.1}%)",
                        m.ram_available_gib(),
                        (m.ram_available_bytes as f64 / m.ram_total_bytes.max(1) as f64) * 100.0
                    ),
                    colors::TEXT_PRIMARY,
                );
                metric_cell(
                    ui,
                    "OS CACHE / PEAK",
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
        (m.commit_total_bytes as f64 / m.commit_limit_bytes as f64 * 100.0) as f32
    } else {
        0.0
    };
    let c = load_color(cp);
    let resp2 = card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let hdr = egui::Label::new(
                    egui::RichText::new("COMMIT CHARGE & KERNEL POOLS")
                        .monospace()
                        .size(13.0)
                        .color(colors::TEXT_PRIMARY),
                )
                .wrap(false);
                ui.add(hdr).on_hover_text(
                    "Commit Charge = virtual memory committed by programs (RAM + Pagefile). Kernel Pools = memory used by Windows core (paged/non-paged).",
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{:.1}%", cp)).monospace().size(28.0).color(c),
                    );
                });
            });
            ui.add_space(8.0);
            let pv = if v { cp / 100.0 } else { 0.0 };
            let bw = ui.available_width().max(80.0);
            let (r, rs) = ui.allocate_exact_size(egui::vec2(bw, 16.0), egui::Sense::hover());
            rs.on_hover_text(
                "Virtual Commit Charge vs system Commit Limit (RAM + Pagefile). >85% = OOM risk.",
            );
            draw_segmented_led_bar(ui, r, pv, 28, colors::ACCENT_PURPLE);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Pagefile Swap:")
                        .monospace()
                        .size(10.0)
                        .color(colors::TEXT_SECONDARY),
                );
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(format!("{:.1}%", m.swap_usage_percent()))
                            .monospace()
                            .size(11.0)
                            .color(colors::TEXT_PRIMARY),
                    )
                    .wrap(false),
                );
                ui.label(
                    egui::RichText::new("|").monospace().size(10.0).color(colors::TEXT_SECONDARY),
                );
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(format!(
                            "{} Procs / {} Threads",
                            m.system_processes, m.system_threads
                        ))
                        .monospace()
                        .size(11.0)
                        .color(colors::TEXT_PRIMARY),
                    )
                    .wrap(false),
                );
            });
            ui.add_space(10.0);
            egui::Grid::new("commit_top_grid").num_columns(3).spacing([16.0, 6.0]).show(ui, |ui| {
                metric_cell(
                    ui,
                    "COMMIT / LIMIT",
                    &format!("{:.2} / {:.2} GiB", m.commit_total_gib(), m.commit_limit_gib()),
                    colors::ACCENT_PURPLE,
                );
                metric_cell(
                    ui,
                    "KERNEL POOLS",
                    &format!(
                        "{:.2} GiB ({:.0}/{:.0}M)",
                        m.kernel_total_mib() / 1024.0,
                        m.kernel_paged_mib(),
                        m.kernel_nonpaged_mib()
                    ),
                    colors::ACCENT_AMBER,
                );
                metric_cell(
                    ui,
                    "AVAILABLE COMMIT",
                    &format_bytes(m.commit_limit_bytes.saturating_sub(m.commit_total_bytes)),
                    colors::TEXT_PRIMARY,
                );
                ui.end_row();
            });
        });
    });
    draw_cyber_corner_brackets(ui, resp2.response.rect, cyber_accent(cp));
}

fn legend_chip(ui: &mut egui::Ui, c: egui::Color32, t: &str) {
    ui.label(egui::RichText::new("\u{25A0}").monospace().size(12.0).color(c));
    ui.add(
        egui::Label::new(egui::RichText::new(t).monospace().size(10.0).color(colors::TEXT_PRIMARY))
            .wrap(false),
    );
    ui.add_space(14.0);
}

fn metric_cell(ui: &mut egui::Ui, cap: &str, val: &str, vc: egui::Color32) {
    ui.vertical(|ui| {
        ui.add(
            egui::Label::new(
                egui::RichText::new(cap).monospace().size(10.0).color(colors::TEXT_SECONDARY),
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

fn segment_widths(px: f32, iu: f64, ca: f64, fr: f64, t: f64) -> (f32, f32, f32) {
    let p = px.max(0.0);
    let tt = t.max(1.0);
    let ifr = (iu.max(0.0) / tt).clamp(0.0, 1.0);
    let cfr = (ca.max(0.0) / tt).clamp(0.0, 1.0 - ifr);
    let ffr = (fr.max(0.0) / tt).clamp(0.0, 1.0 - ifr - cfr);
    let i = (p as f64 * ifr).clamp(0.0, p as f64) as f32;
    let c_ = (p as f64 * cfr).clamp(0.0, (p - i).max(0.0) as f64) as f32;
    let f = (p as f64 * ffr).clamp(0.0, (p - i - c_).max(0.0) as f64) as f32;
    (i, c_, f)
}
