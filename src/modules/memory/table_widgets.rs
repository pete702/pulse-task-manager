//! Shared widgets for the virtualized memory process table.

use eframe::egui;

use crate::i18n::Lang;
use crate::modules::memory::process_model::MemorySpikeEvent;
use crate::theme::colors;

use super::table_view::MemoryTableState;

/// Renders the amber/red "black box" spike banner.
pub(super) fn render_spike_banner(ui: &mut egui::Ui, spike: &MemorySpikeEvent, lang: Lang) {
    let (title, at) = match lang {
        Lang::Uk => ("⚠️ ПІК ОЗП", "час"),
        Lang::En => ("⚠️ RAM SPIKE", "at"),
    };

    egui::Frame::none()
        .fill(colors::ACCENT_RED.gamma_multiply(0.12))
        .stroke(egui::Stroke::new(1.0_f32, colors::ACCENT_RED))
        .rounding(egui::Rounding::same(colors::RADIUS))
        .inner_margin(egui::Margin::same(9.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(title)
                        .monospace()
                        .size(12.0)
                        .strong()
                        .color(colors::ACCENT_RED),
                );
                ui.label(
                    egui::RichText::new(format!("{:.1}%", spike.peak_ram_pct))
                        .monospace()
                        .size(14.0)
                        .color(colors::ACCENT_AMBER),
                );
                ui.label(
                    egui::RichText::new(format!("{at}: {}", spike.timestamp))
                        .monospace()
                        .size(10.0)
                        .color(colors::TEXT_SECONDARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "{} [PID {}] • {}",
                            spike.culprit_name,
                            spike.culprit_pid,
                            format_bytes(spike.culprit_bytes)
                        ))
                        .monospace()
                        .size(11.0)
                        .color(colors::TEXT_PRIMARY),
                    );
                });
            });
        });
}

/// Renders search, clone grouping, and the visible/total counter.
pub(super) fn render_toolbar(
    ui: &mut egui::Ui,
    total: usize,
    shown: usize,
    clone_groups: usize,
    state: &mut MemoryTableState,
    lang: Lang,
) {
    let (search_hint, group_label, counter) = match lang {
        Lang::Uk => ("Пошук: назва або PID", "Групувати клони", "Показано"),
        Lang::En => ("Search: name or PID", "Group clones", "Shown"),
    };

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("🔍").size(14.0));
        ui.add(
            egui::TextEdit::singleline(&mut state.search_query)
                .desired_width(220.0)
                .hint_text(search_hint),
        );
        if !state.search_query.is_empty() && ui.button("✕").clicked() {
            state.search_query.clear();
        }
        ui.separator();
        ui.checkbox(
            &mut state.group_clones,
            format!("{group_label} [{clone_groups}x]"),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("{counter}: {shown}/{total}"))
                    .monospace()
                    .size(11.0)
                    .color(colors::TEXT_SECONDARY),
            );
        });
    });
}

/// Localized empty-state text.
pub(super) fn empty_label(lang: Lang) -> &'static str {
    match lang {
        Lang::Uk => "Немає процесів за цим запитом",
        Lang::En => "No processes match this query",
    }
}

/// Column header labels: name, PID, physical RAM, virtual memory, RAM percent.
pub(super) fn header_labels(lang: Lang) -> [&'static str; 5] {
    match lang {
        Lang::Uk => ["Процес", "PID", "Фізична ОЗП", "Віртуальна", "ОЗП %"],
        Lang::En => ["Process", "PID", "Physical RAM", "Virtual", "RAM %"],
    }
}

/// Formats bytes into a compact binary unit string.
pub(super) fn format_bytes(bytes: u64) -> String {
    const GIB: u64 = 1_073_741_824;
    const MIB: u64 = 1_048_576;
    const KIB: u64 = 1024;

    if bytes >= GIB {
        format!("{:.2} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.2} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.2} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes_table() {
        assert_eq!(format_bytes(1_073_741_824), "1.00 GiB");
        assert_eq!(format_bytes(1_048_576), "1.00 MiB");
        assert_eq!(format_bytes(1024), "1.00 KiB");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(0), "0 B");
    }

    #[test]
    fn test_header_labels_localized() {
        assert_eq!(header_labels(Lang::Uk)[0], "Процес");
        assert_eq!(header_labels(Lang::Uk)[4], "ОЗП %");
        assert_eq!(header_labels(Lang::En)[0], "Process");
        assert_eq!(header_labels(Lang::En)[4], "RAM %");
        assert_eq!(empty_label(Lang::Uk), "Немає процесів за цим запитом");
        assert_eq!(empty_label(Lang::En), "No processes match this query");
    }
}
