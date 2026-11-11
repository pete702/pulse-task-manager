//! Bottom action bar and grouped process actions (split from table_menu.rs).
use crate::modules::cpu::model::{ProcessCpuItem, RawProcessCpuRow};
use crate::modules::cpu::qos_action as q;
use crate::modules::cpu::table_menu::{action_button, resolve_group_targets, GroupTarget};
use crate::modules::cpu::table_state::CpuTableState;
use crate::theme::colors;
use eframe::egui;
use std::time::Instant;

fn wk(state: &mut CpuTableState) {
    state.last_action_msg = Some(("Working...".to_owned(), Instant::now()));
}
fn ct_ok(t: &GroupTarget) -> bool {
    t.creation_time
        .is_none_or(|ct| crate::modules::memory::query_creation_time_by_pid(t.pid) == Some(ct))
}
pub(crate) fn spawn_trim(state: &mut CpuTableState, targets: Vec<GroupTarget>, name: String) {
    let total = targets.len();
    wk(state);
    state.spawn_menu_action(move || {
        let (mut ok, mut last) = (0, String::new());
        for t in targets {
            if !ct_ok(&t) {
                continue;
            }
            match q::trim_process_working_set(t.pid, &t.name, t.creation_time, t.exe_path) {
                Ok(()) => ok += 1,
                Err(e) => last = e,
            }
        }
        let m = if total == 1 {
            if ok == 1 {
                "Working set trimmed".to_owned()
            } else {
                format!("Trim blocked: {last}")
            }
        } else {
            format!("Trimmed RAM for {ok}/{total} processes in {name} group")
        };
        (m, None, Vec::new())
    });
}
pub(crate) fn spawn_eco(
    state: &mut CpuTableState,
    targets: Vec<GroupTarget>,
    name: String,
    enable: bool,
) {
    let prevs: Vec<((u32, u64), Option<bool>)> = targets
        .iter()
        .map(|t| ((t.pid, t.start_time), state.eco_overrides.get(&(t.pid, t.start_time)).copied()))
        .collect();
    for t in &targets {
        let key = (t.pid, t.start_time);
        if state.eco_overrides.len() >= 512 && !state.eco_overrides.contains_key(&key) {
            if let Some(&k) = state.eco_overrides.keys().next() {
                state.eco_overrides.remove(&k);
            }
        }
        state.eco_overrides.insert(key, enable);
    }
    let total = targets.len();
    let (lead, lst) = (targets[0].pid, targets[0].start_time);
    let pairs: Vec<(GroupTarget, Option<bool>)> =
        targets.into_iter().zip(prevs.into_iter().map(|(_, v)| v)).collect();
    wk(state);
    state.spawn_menu_action(move || {
        let (mut ok, mut last) = (0, String::new());
        let mut rollbacks: Vec<((u32, u64), Option<bool>)> = Vec::new();
        for (t, prev) in pairs {
            match q::set_process_eco_qos(t.pid, &t.name, t.creation_time, t.exe_path, enable) {
                Ok(()) => ok += 1,
                Err(e) => {
                    last = e;
                    rollbacks.push(((t.pid, t.start_time), prev));
                }
            }
        }
        let v = if enable { "enabled" } else { "disabled" };
        if total == 1 {
            let m = if ok == 1 { format!("EcoQoS {v}") } else { format!("EcoQoS blocked: {last}") };
            (m, (ok == 1).then_some((lead, lst, enable)), rollbacks)
        } else {
            (format!("EcoQoS {v} for {ok}/{total} processes in {name} group"), None, rollbacks)
        }
    });
}
pub(crate) fn spawn_open(state: &mut CpuTableState, targets: Vec<GroupTarget>) {
    let t = targets.into_iter().next().unwrap();
    wk(state);
    state.spawn_menu_action(move || {
        let m = match q::open_process_file_location(
            t.pid,
            &t.name,
            t.creation_time,
            t.exe_path.as_deref(),
        ) {
            Ok(()) => "Opened file location".to_owned(),
            Err(e) => format!("Open blocked: {e}"),
        };
        (m, None, Vec::new())
    });
}
pub(crate) fn spawn_kill(state: &mut CpuTableState, targets: Vec<GroupTarget>, name: String) {
    state.selected_pid = None;
    state.selected_start_time = None;
    state.selected_name = None;
    state.confirm_kill = None;
    let total = targets.len();
    wk(state);
    state.spawn_menu_action(move || {
        let (mut ok, mut last) = (0, String::new());
        for t in targets {
            if !ct_ok(&t) {
                continue;
            }
            match q::terminate_process(t.pid, &t.name, t.creation_time, t.exe_path) {
                Ok(()) => ok += 1,
                Err(e) => last = e,
            }
        }
        let m = if total == 1 {
            if ok == 1 {
                "Task ended".to_owned()
            } else {
                format!("End blocked: {last}")
            }
        } else {
            format!("Terminated {ok}/{total} processes in {name} group")
        };
        (m, None, Vec::new())
    });
}
pub(crate) fn render_bottom_action_bar(
    ui: &mut egui::Ui,
    selected: Option<&ProcessCpuItem>,
    raw: &[RawProcessCpuRow],
    state: &mut CpuTableState,
) {
    if state.action_rx.is_none()
        && state.last_action_msg.as_ref().is_some_and(|(_, at)| at.elapsed().as_secs() >= 8)
    {
        state.last_action_msg = None;
    }
    ui.horizontal(|ui| {
        let can_act = selected.is_some();
        state.confirm_kill = state.confirm_kill.filter(|(pid, at)| {
            Instant::now().duration_since(*at).as_secs() < 3 && state.selected_pid == Some(*pid)
        });
        if let Some(item) = selected {
            if action_button(ui, can_act, "[ Trim RAM ]", colors::ACCENT_CYAN).clicked() {
                let t = resolve_group_targets(item, state.group_clones, raw);
                spawn_trim(state, t, item.name.to_string());
            }
            if action_button(ui, can_act, "[ EcoQoS ]", crate::theme::OPTIMAL_EMERALD).clicked() {
                let t = resolve_group_targets(item, state.group_clones, raw);
                spawn_eco(state, t, item.name.to_string(), item.eco_qos != Some(true));
            }
            if action_button(ui, can_act, "[ Open Location ]", colors::TEXT_PRIMARY).clicked() {
                let t = resolve_group_targets(item, state.group_clones, raw);
                spawn_open(state, t);
            }
            if let Some((armed, _)) = state.confirm_kill {
                if armed == item.pid {
                    if action_button(ui, true, "[ Cancel ]", colors::TEXT_SECONDARY).clicked() {
                        state.confirm_kill = None;
                        state.last_action_msg = Some(("Kill cancelled".to_owned(), Instant::now()));
                    }
                    if action_button(ui, true, "[ Confirm Kill (3s) ]", colors::ACCENT_RED)
                        .clicked()
                    {
                        let t = resolve_group_targets(item, state.group_clones, raw);
                        spawn_kill(state, t, item.name.to_string());
                    }
                } else {
                    state.confirm_kill = None;
                }
            } else if action_button(ui, can_act, "[ End Task ]", colors::ACCENT_RED).clicked() {
                state.confirm_kill = Some((item.pid, Instant::now()));
                let m =
                    format!("Click Confirm Kill within 3s to end {} (PID {})", item.name, item.pid);
                state.last_action_msg = Some((m, Instant::now()));
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let msg = state.last_action_msg.as_ref().map(|(s, _)| s.as_str()).unwrap_or("");
            let c = if msg.contains("blocked") {
                colors::ACCENT_RED
            } else if msg.is_empty() {
                colors::TEXT_SECONDARY
            } else {
                colors::ACCENT_CYAN
            };
            let rt = egui::RichText::new(if msg.is_empty() { " " } else { msg });
            let rt = rt.monospace().size(10.0).color(c);
            let l = egui::Label::new(rt).truncate(true);
            ui.add_sized(egui::Vec2::new(280.0, 18.0), l);
        });
    });
}
