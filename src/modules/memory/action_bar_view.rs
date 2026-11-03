//! Full-width rendering for the memory action bar.

use crate::modules::memory::action_bar::{
    format_trim_status, MemoryActionBarState, BUSY_HINT, NO_SELECTION_HINT, PROTECTED_HINT,
};
use crate::modules::memory::action_bar_status::{mode_label, selection_info};
use crate::modules::memory::actions;
use crate::modules::memory::model::MemorySnapshot;
use crate::modules::memory::selection::{raw_targets, resolve_selection, SelectionView};
use crate::modules::memory::table_view::MemoryTableState;
use crate::theme::colors;
use eframe::egui;
use std::time::Instant;

/// Renders the full-width action bar; every action targets raw snapshot processes.
pub fn render_action_bar(
    ui: &mut egui::Ui,
    mem: &MemorySnapshot,
    ts: &MemoryTableState,
    st: &mut MemoryActionBarState,
) {
    let now = Instant::now();
    st.poll(now);
    let busy = st.is_busy();
    let sel = resolve_selection(mem, ts.selected_group.as_deref());
    crate::theme::card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            mode_label(ui);
            ui.separator();
            trim_all_button(ui, mem, st, busy);
            ui.separator();
            selection_actions(ui, &sel, st, now, busy);
            selection_info(ui, &sel, st, now);
        });
        if let Some(t) = st.status_text(now) {
            ui.label(egui::RichText::new(t).monospace().size(10.0).color(colors::TEXT_SECONDARY));
        }
    });
}

fn hint(sel: &Option<SelectionView>, busy: bool) -> &'static str {
    if busy {
        BUSY_HINT
    } else if sel.is_some() {
        PROTECTED_HINT
    } else {
        NO_SELECTION_HINT
    }
}

fn trim_all_button(
    ui: &mut egui::Ui,
    mem: &MemorySnapshot,
    st: &mut MemoryActionBarState,
    busy: bool,
) {
    let enabled = !busy && !mem.processes.is_empty();
    let btn = ui.add_enabled(enabled, egui::Button::new("⚡ Trim Working Sets"));
    let btn = if enabled {
        btn.on_hover_text("Trim every eligible raw process (ignores search and grouping)")
    } else if busy {
        btn.on_disabled_hover_text(BUSY_HINT)
    } else {
        btn.on_disabled_hover_text("No processes in the live snapshot yet")
    };
    if btn.clicked() {
        let items = raw_targets(mem);
        st.spawn(move || {
            let report = actions::purge_system_working_sets(&items);
            let _ = crate::core::win32::self_trim_working_set();
            format_trim_status(&report)
        });
    }
}

fn selection_actions(
    ui: &mut egui::Ui,
    sel: &Option<SelectionView>,
    st: &mut MemoryActionBarState,
    now: Instant,
    busy: bool,
) {
    let hint = hint(sel, busy);
    let can_modify = !busy && sel.as_ref().is_some_and(|v| v.can_modify());
    let trim = ui.add_enabled(can_modify, egui::Button::new("Trim Selected"));
    let trim = if can_modify {
        trim.on_hover_text("Trim every raw process in the selected group")
    } else {
        trim.on_disabled_hover_text(hint)
    };
    if trim.clicked() {
        if let Some(v) = sel {
            let items = v.targets.clone();
            st.spawn(move || format_trim_status(&actions::trim_process_group(&items)));
        }
    }
    open_location_button(ui, sel, st, now, busy, hint);
    end_task_button(ui, sel, st, now, can_modify, hint);
}

fn open_location_button(
    ui: &mut egui::Ui,
    sel: &Option<SelectionView>,
    st: &mut MemoryActionBarState,
    now: Instant,
    busy: bool,
    hint: &str,
) {
    let path = sel.as_ref().and_then(|v| v.path.clone());
    let enabled = !busy && path.is_some();
    let open = ui.add_enabled(enabled, egui::Button::new("Open Location"));
    let open = if enabled {
        open.on_hover_text("Open the executable folder in Explorer")
    } else if busy {
        open.on_disabled_hover_text(BUSY_HINT)
    } else if path.is_none() && sel.is_some() {
        open.on_disabled_hover_text("No executable path recorded for this process")
    } else {
        open.on_disabled_hover_text(hint)
    };
    if open.clicked() {
        if let (Some(p), Some(v)) = (path, sel) {
            let msg = match actions::open_process_file_location(std::path::Path::new(&p)) {
                Ok(()) => format!("Opened location for {}", v.name),
                Err(e) => format!("Open location failed: {e}"),
            };
            st.set_status(now, msg);
        }
    }
}

fn end_task_button(
    ui: &mut egui::Ui,
    sel: &Option<SelectionView>,
    st: &mut MemoryActionBarState,
    now: Instant,
    can_modify: bool,
    hint: &str,
) {
    let confirming = sel.as_ref().is_some_and(|v| st.confirm_kill_active(&v.name, now));
    let label = if confirming { "Confirm Kill?" } else { "End Task" };
    let end = ui.add_enabled(can_modify, egui::Button::new(label));
    let end = if can_modify {
        end.on_hover_text("End every raw process in the group (two-step confirmation)")
    } else {
        end.on_disabled_hover_text(hint)
    };
    if end.clicked() {
        if let Some(v) = sel {
            if confirming {
                let (pids, items) = (v.pids.clone(), v.targets.clone());
                st.cancel_confirm();
                st.spawn(move || match actions::terminate_process_group(&pids, &items) {
                    Ok(n) => format!("End task: {n}/{} processes terminated", pids.len()),
                    Err(e) => format!("End task blocked: {e}"),
                });
            } else {
                st.begin_confirm_kill(&v.name, now);
            }
        }
    }
}
