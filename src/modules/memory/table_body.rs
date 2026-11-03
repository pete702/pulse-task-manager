//! Virtualized table body for the memory process list.
use eframe::egui;
use egui_extras::{Column, TableBuilder};

use crate::modules::memory::process_model::{MemorySortCol, ProcessMemoryItem};
use crate::theme::colors;

use super::table_view::MemoryTableState;
use super::table_widgets::{format_bytes, header_labels};

/// Renders the virtualized rows and sortable header.
pub(super) fn render_table(
    ui: &mut egui::Ui,
    rows: &[ProcessMemoryItem],
    state: &mut MemoryTableState,
    height: f32,
) {
    let labels = header_labels();
    let table = TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .sense(egui::Sense::click())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .auto_shrink([false, false])
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .column(Column::remainder().at_least(180.0))
        .column(Column::initial(75.0))
        .column(Column::initial(120.0))
        .column(Column::initial(120.0))
        .column(Column::initial(95.0))
        .header(26.0, |mut header| {
            for (idx, col) in [
                MemorySortCol::Name,
                MemorySortCol::Pid,
                MemorySortCol::RamBytes,
                MemorySortCol::VirtualBytes,
                MemorySortCol::RamPct,
            ]
            .into_iter()
            .enumerate()
            {
                header.col(|ui| {
                    let active = state.sort_col == col;
                    if sort_header(ui, labels[idx], active, state.sort_asc) {
                        toggle_sort(state, col);
                    }
                });
            }
        });

    table.body(|body| {
        body.rows(24.0, rows.len(), |mut row| {
            let item = &rows[row.index()];
            row.set_selected(state.selected_pid == Some(item.pid));
            row.col(|ui| render_name_cell(ui, item));
            row.col(|ui| {
                ui.label(
                    egui::RichText::new(item.pid.to_string())
                        .monospace()
                        .size(11.0)
                        .color(colors::TEXT_SECONDARY),
                );
            });
            row.col(|ui| {
                ui.label(
                    egui::RichText::new(format_bytes(item.ram_bytes))
                        .monospace()
                        .size(11.0)
                        .color(crate::theme::load_color(item.ram_pct)),
                );
            });
            row.col(|ui| {
                ui.label(
                    egui::RichText::new(format_bytes(item.virtual_bytes))
                        .monospace()
                        .size(11.0)
                        .color(colors::TEXT_PRIMARY),
                );
            });
            row.col(|ui| {
                ui.label(
                    egui::RichText::new(format!("{:.1}%", item.ram_pct))
                        .monospace()
                        .size(11.0)
                        .color(crate::theme::load_color(item.ram_pct)),
                );
            });
            if row.response().clicked() {
                state.selected_pid = Some(item.pid);
            }
        });
    });
}

fn render_name_cell(ui: &mut egui::Ui, item: &ProcessMemoryItem) {
    ui.label(
        egui::RichText::new(&item.name)
            .monospace()
            .size(12.0)
            .strong()
            .color(colors::TEXT_PRIMARY),
    );
    if item.process_count > 1 {
        ui.label(
            egui::RichText::new(format!("[{}x]", item.process_count))
                .monospace()
                .size(10.0)
                .color(colors::ACCENT_CYAN)
                .background_color(colors::ACCENT_CYAN.gamma_multiply(0.12)),
        );
    }
    if item.is_hog {
        // Use geometric ▲ HOG badge in a Frame instead of ⚠️ emoji
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(0x3A, 0x10, 0x1D))
            .stroke(egui::Stroke::new(
                1.0_f32,
                egui::Color32::from_rgb(0xE0, 0x2A, 0x54),
            ))
            .rounding(egui::Rounding::same(3.0))
            .inner_margin(egui::Margin::symmetric(5.0, 1.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("▲ HOG")
                        .monospace()
                        .size(9.0)
                        .color(egui::Color32::from_rgb(0xE0, 0x2A, 0x54)),
                );
            });
    }
}

fn sort_header(ui: &mut egui::Ui, text: &str, active: bool, asc: bool) -> bool {
    let label = if active {
        format!("{text} {}", if asc { "▲" } else { "▼" })
    } else {
        text.to_owned()
    };
    ui.add(
        egui::Button::new(
            egui::RichText::new(label)
                .monospace()
                .size(10.0)
                .color(if active {
                    colors::ACCENT_CYAN
                } else {
                    colors::TEXT_SECONDARY
                }),
        )
        .frame(false),
    )
    .clicked()
}

fn toggle_sort(state: &mut MemoryTableState, col: MemorySortCol) {
    if state.sort_col == col {
        state.sort_asc = !state.sort_asc;
    } else {
        state.sort_col = col;
        state.sort_asc = matches!(col, MemorySortCol::Name | MemorySortCol::Pid);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_toggle_sort() {
        let mut s = MemoryTableState::default();
        toggle_sort(&mut s, MemorySortCol::RamBytes);
        assert!(s.sort_asc);
        toggle_sort(&mut s, MemorySortCol::Name);
        assert_eq!(s.sort_col, MemorySortCol::Name);
        assert!(s.sort_asc);
        toggle_sort(&mut s, MemorySortCol::Name);
        assert!(!s.sort_asc);
    }
}
