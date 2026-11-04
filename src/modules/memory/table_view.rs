//! Virtualized process memory table with search, clone grouping, sorting, and selection.

use eframe::egui;

use crate::modules::memory::model::MemorySnapshot;
use crate::theme::colors;

use super::table_body::render_table;
use super::table_state::MemoryTableState;
use super::table_widgets::{empty_label, render_spike_banner, render_toolbar};

/// Renders the spike banner, toolbar, and virtualized process table.
pub fn render_memory_table(
    ui: &mut egui::Ui,
    snap: &MemorySnapshot,
    state: &mut MemoryTableState,
    _uptime_secs: u64,
) {
    state.rebuild_for_scan(snap.process_scan_id, &snap.processes, snap.ram_total_bytes);
    let rows = state.cached_rows().unwrap_or_default();
    let clone_groups = rows.iter().filter(|r| r.process_count > 1).count();
    let total_processes = snap.processes.len();
    let shown_rows = rows.len();
    crate::theme::card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            if let Some(ref spike) = snap.latest_spike {
                render_spike_banner(ui, spike);
                ui.add_space(8.0);
            }
            let (old_sort_col, old_sort_asc, old_group_clones) =
                (state.sort_col, state.sort_asc, state.group_clones);
            render_toolbar(ui, total_processes, shown_rows, clone_groups, state);
            ui.add_space(6.0);
            if state.group_clones != old_group_clones {
                // The checkbox mutates the field directly, so restore and re-apply through
                // the setter to guarantee selection/confirmation invalidation.
                let new_mode = state.group_clones;
                state.group_clones = old_group_clones;
                state.set_group_clones(new_mode);
            } else if state.sort_col != old_sort_col || state.sort_asc != old_sort_asc {
                state.invalidate_cache();
            }
            if rows.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new(empty_label())
                            .monospace()
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY),
                    );
                });
            } else {
                let height = (ui.available_height() - 4.0).max(120.0);
                render_table(ui, &rows, state, height);
            }
        });
    });
}
