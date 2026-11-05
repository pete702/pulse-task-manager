//! Swap & Commit card rendering for PULSE memory HUD.
use crate::i18n::Key;
use crate::modules::memory::formatters::format_bytes;
use crate::modules::memory::model::MemorySnapshot;
use crate::theme::{card_frame, colors};
use crate::ui::primitives::draw_segmented_led_bar;
use eframe::egui;
const NA: &str = "N/A";
pub fn render_swap_commit_card(ui: &mut egui::Ui, m: &MemorySnapshot) {
    card_frame().show(ui, |ui| {
        let w = ui.available_width().max(80.0);
        let valid = m.kernel_metrics_valid;
        let cp = if valid && m.commit_limit_bytes > 0 {
            (m.commit_total_bytes as f64 / m.commit_limit_bytes as f64 * 100.0) as f32
        } else {
            0.0
        };
        let sp = if valid && m.swap_total_bytes > 0 { m.swap_usage_percent() } else { 0.0 };
        let cu = if valid { format!("{:.2} GiB", m.commit_total_gib()) } else { NA.to_owned() };
        let cl = if valid { format!("{:.2} GiB", m.commit_limit_gib()) } else { NA.to_owned() };
        let su = if valid && m.swap_total_bytes > 0 {
            format!("{:.2} GiB", m.swap_used_gib())
        } else {
            NA.to_owned()
        };
        let st = if valid && m.swap_total_bytes > 0 {
            format!("{:.2} GiB", m.swap_total_gib())
        } else {
            NA.to_owned()
        };
        let ac = if valid {
            format_bytes(m.commit_limit_bytes.saturating_sub(m.commit_total_bytes))
        } else {
            NA.to_owned()
        };
        let pct = if valid { format!("{:.1}%", cp) } else { NA.to_owned() };
        let spt =
            if valid && m.swap_total_bytes > 0 { format!("{:.1}%", sp) } else { NA.to_owned() };
        let cc = if valid { crate::theme::load_color(cp) } else { colors::BORDER };
        let _fc = if valid { colors::ACCENT_PURPLE } else { colors::BORDER };
        let uc = if valid { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY };
        let sc = if valid { colors::TEXT_PRIMARY } else { colors::TEXT_SECONDARY };
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(Key::SwapCommitTitle.t())
                        .monospace()
                        .size(13.0)
                        .color(colors::TEXT_PRIMARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(pct).monospace().size(22.0).color(cc))
                        .on_hover_text("Commit charge as a share of the system commit limit");
                });
            });
            ui.add_space(8.0);
            let pv = if valid { cp / 100.0 } else { 0.0 };
            let (r, resp) = ui.allocate_exact_size(egui::vec2(w, 16.0), egui::Sense::hover());
            resp.on_hover_text("Virtual Commit Charge vs system Commit Limit. >85% = OOM risk, 70-85% = monitor, <70% = healthy. Commit Limit = RAM + Pagefile");
            draw_segmented_led_bar(ui, r, pv, 28, colors::ACCENT_PURPLE);
            ui.add_space(12.0);
            egui::Grid::new("scg").num_columns(2).spacing([16.0, 8.0]).show(ui, |ui| {
                ui.label(
                    egui::RichText::new(Key::CommitChargeLimit.t())
                        .monospace()
                        .size(12.0)
                        .color(colors::TEXT_SECONDARY),
                );
                ui.label(
                    egui::RichText::new(format!("{} / {}", cu, cl))
                        .monospace()
                        .size(13.0)
                        .color(if valid { colors::ACCENT_PURPLE } else { colors::TEXT_SECONDARY }),
                );
                ui.end_row();
                ui.label(
                    egui::RichText::new("Available Commit")
                        .monospace()
                        .size(12.0)
                        .color(colors::TEXT_SECONDARY),
                );
                ui.label(egui::RichText::new(ac).monospace().size(13.0).color(uc));
                ui.end_row();
                ui.label(
                    egui::RichText::new(Key::SwapUsedTotal.t())
                        .monospace()
                        .size(12.0)
                        .color(colors::TEXT_SECONDARY),
                );
                ui.label(
                    egui::RichText::new(format!("{} / {}", su, st))
                        .monospace()
                        .size(13.0)
                        .color(sc),
                );
                ui.end_row();
                ui.label(
                    egui::RichText::new("Swap / Pagefile Usage %")
                        .monospace()
                        .size(12.0)
                        .color(colors::TEXT_SECONDARY),
                );
                ui.label(egui::RichText::new(spt).monospace().size(13.0).color(sc));
                ui.end_row();
            });
        });
    });
}
