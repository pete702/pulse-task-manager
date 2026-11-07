use crate::modules::memory::action_bar::{format_trim_status, MemoryActionBarState, BUSY_HINT};
use crate::modules::memory::action_bar_status::{hint, mode_label, selection_info};
use crate::modules::memory::{
    actions, model::MemorySnapshot, process_target::ProcessTarget,
    table_selection::SelectedGroupSummary, table_state::MemoryTableState,
};
use crate::theme::colors;
use eframe::egui;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

static TRIM_RUNNING: AtomicBool = AtomicBool::new(false);
struct TrimGuard;
impl Drop for TrimGuard {
    fn drop(&mut self) {
        TRIM_RUNNING.store(false, Ordering::Release);
    }
}
pub fn render_action_bar(
    ui: &mut egui::Ui,
    mem: &MemorySnapshot,
    ts: &mut MemoryTableState,
    st: &mut MemoryActionBarState,
    telemetry_paused: &mut bool,
) {
    let now = Instant::now();
    st.poll(now);
    let busy = st.is_busy();
    if ts.take_group_clones_toggled() {
        st.cancel_confirm();
    }
    let sel = ts.resolved_selection(mem.process_scan_id, &mem.processes);
    crate::theme::card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                mode_label(ui);
                ui.separator();
                trim_all_button(ui, mem, st, busy);
                ui.separator();
                selection_actions(ui, &sel, st, now, busy);
                ui.separator();
                pause_button(ui, telemetry_paused, busy);
                selection_info(ui, &sel, st, now);
            });
            ui.allocate_ui_with_layout(
                egui::vec2(280.0, 18.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    if let Some(v) = sel.as_ref() {
                        if st.confirm_kill_active(&v.name, now) {
                            ui.label(
                                egui::RichText::new(format!(
                                    "[ Confirm Kill armed for {} ]",
                                    v.name
                                ))
                                .monospace()
                                .size(10.0)
                                .color(colors::ACCENT_RED),
                            );
                        } else if let Some(t) = st.status_text(now) {
                            ui.label(
                                egui::RichText::new(t)
                                    .monospace()
                                    .size(10.0)
                                    .color(colors::TEXT_SECONDARY),
                            );
                        }
                    }
                },
            );
        });
    });
}
fn trim_all_button(
    ui: &mut egui::Ui,
    mem: &MemorySnapshot,
    st: &mut MemoryActionBarState,
    busy: bool,
) {
    let enabled = !busy && !mem.processes.is_empty();
    let btn = ui.add_enabled(enabled, egui::Button::new("Trim Working Sets"));
    let btn = if enabled {
        btn.on_hover_text("Trim every eligible raw process (ignores search and grouping)")
    } else if busy {
        btn.on_disabled_hover_text(BUSY_HINT)
    } else {
        btn.on_disabled_hover_text("No processes in the live snapshot yet")
    };
    if btn.clicked() {
        if TRIM_RUNNING.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_err()
        {
            return;
        }
        let raw = Arc::clone(&mem.processes);
        let spawned = st.spawn(move || {
            let _trim_guard = TrimGuard;
            let items = raw.iter().map(ProcessTarget::from_raw).collect::<Vec<_>>();
            let msg = format_trim_status(&actions::purge_system_working_sets(&items));
            crate::core::request_process_scan();
            msg
        });
        if !spawned {
            TRIM_RUNNING.store(false, Ordering::Release);
        }
    }
}
fn selection_actions(
    ui: &mut egui::Ui,
    sel: &Option<Arc<SelectedGroupSummary>>,
    st: &mut MemoryActionBarState,
    now: Instant,
    busy: bool,
) {
    let hint = hint(sel, busy);
    let can_modify = !busy && sel.as_ref().is_some_and(|v| !v.blocked && !v.targets.is_empty());
    let multi = sel.as_ref().is_some_and(|v| v.targets.len() > 1);
    let trim_text = if multi {
        "Trim every raw process in the selected group"
    } else {
        "Trim the selected process"
    };
    let trim = ui.add_enabled(can_modify, egui::Button::new("Trim Selected"));
    let trim =
        if can_modify { trim.on_hover_text(trim_text) } else { trim.on_disabled_hover_text(hint) };
    if trim.clicked() {
        if let Some(v) = sel {
            let items = v.targets.clone();
            crate::core::request_process_scan();
            st.spawn(move || format_trim_status(&actions::trim_process_group(&items)));
        }
    }
    open_location_button(ui, sel, st, now, busy, hint);
    end_task_button(ui, sel, st, now, can_modify, hint, multi);
}
fn open_location_button(
    ui: &mut egui::Ui,
    sel: &Option<Arc<SelectedGroupSummary>>,
    st: &mut MemoryActionBarState,
    _now: Instant,
    busy: bool,
    hint: &str,
) {
    let enabled = !busy && sel.as_ref().is_some_and(|v| !v.blocked && !v.targets.is_empty());
    let open = ui.add_enabled(enabled, egui::Button::new("Open Location"));
    let open = if enabled {
        open.on_hover_text("Open the executable folder in Explorer")
    } else if busy {
        open.on_disabled_hover_text(BUSY_HINT)
    } else {
        open.on_disabled_hover_text(hint)
    };
    if open.clicked() {
        if let Some(v) = sel {
            let first_exe_path = v.first_exe_path.clone();
            let first_pid = v.first_pid;
            let name = v.name.clone();
            st.spawn(move || {
                let resolved = first_exe_path.clone().or_else(|| {
                    first_pid.and_then(|pid| {
                        crate::modules::memory::win32_api::query_process_exe_path(pid)
                    })
                });
                match resolved {
                    Some(p) => {
                        match actions::open_process_file_location(std::path::Path::new(&p)) {
                            Ok(()) => format!("Opened location for {}", name),
                            Err(e) => format!("Open location failed: {e}"),
                        }
                    }
                    None => format!("Open location failed: no executable path for {}", name),
                }
            });
        }
    }
}
fn end_task_button(
    ui: &mut egui::Ui,
    sel: &Option<Arc<SelectedGroupSummary>>,
    st: &mut MemoryActionBarState,
    now: Instant,
    can_modify: bool,
    hint: &str,
    multi: bool,
) {
    if let Some(v) = sel {
        st.validate_confirm_targets(&v.targets);
    }
    let confirming = sel.as_ref().is_some_and(|v| st.confirm_kill_active(&v.name, now));
    let label = if confirming { "Confirm Kill?" } else { "End Task" };
    let end = ui.add_enabled(can_modify, egui::Button::new(label));
    let end = if can_modify {
        let text = if multi {
            "End every raw process in the group (two-step confirmation)"
        } else {
            "End this process (two-step confirmation)"
        };
        end.on_hover_text(text)
    } else {
        end.on_disabled_hover_text(hint)
    };
    if end.clicked() {
        if let Some(v) = sel {
            if confirming {
                st.validate_confirm_targets(&v.targets);
                if let Some(targets) = st.confirmed_targets(&v.name, now) {
                    if targets != v.targets {
                        st.cancel_confirm();
                        return;
                    }
                    let groups = targets.len();
                    st.cancel_confirm();
                    crate::core::request_process_scan();
                    st.spawn(move || match actions::terminate_process_group(&targets) {
                        Ok(n) => {
                            let suffix = if n == 0 && groups > 0 {
                                " (access denied or process exited)"
                            } else {
                                ""
                            };
                            format!("End task: {n}/{groups} processes terminated{suffix}")
                        }
                        Err(e) => format!("End task blocked: {e}"),
                    });
                }
            } else {
                st.begin_confirm_kill(&v.name, &v.targets, now);
            }
        }
    }
}
fn pause_button(ui: &mut egui::Ui, paused: &mut bool, busy: bool) {
    let label = if *paused { "> Resume (Paused)" } else { "|| Pause" };
    let color = if *paused { colors::ACCENT_AMBER } else { colors::ACCENT_CYAN };
    let btn = ui
        .add_enabled(!busy, egui::Button::new(egui::RichText::new(label).color(color).monospace()));
    let btn = if *paused {
        btn.on_hover_text("Resume live telemetry updates")
    } else {
        btn.on_hover_text("Pause telemetry updates to inspect current state")
    };
    if btn.clicked() {
        *paused = !*paused;
    }
}
