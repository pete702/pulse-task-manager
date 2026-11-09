//! Process row context menu and sort button for PULSE CPU table.
use crate::modules::cpu::model::{ProcessCpuItem, RawProcessCpuRow};
use crate::modules::cpu::table_menu_actions::{spawn_eco, spawn_open, spawn_trim};
use crate::modules::cpu::table_state::CpuTableState;
use crate::theme::colors;
use eframe::egui;
use std::time::Instant;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GroupTarget {
    pub pid: u32,
    pub name: String,
    pub start_time: u64,
    pub creation_time: Option<u64>,
    pub exe_path: Option<String>,
}
pub(super) fn exe_opt(item: &ProcessCpuItem) -> Option<String> {
    if item.exe_path.is_empty() {
        None
    } else {
        Some(item.exe_path.to_string())
    }
}
fn leader(item: &ProcessCpuItem, raw: &[RawProcessCpuRow]) -> GroupTarget {
    if let Some(r) = raw.iter().find(|r| r.pid == item.pid && r.start_time == item.start_time) {
        let e = r.exe_path.as_deref().map(str::to_owned).or_else(|| exe_opt(item));
        return GroupTarget {
            pid: r.pid,
            name: r.name.to_string(),
            start_time: r.start_time,
            creation_time: r.creation_time,
            exe_path: e,
        };
    }
    GroupTarget {
        pid: item.pid,
        name: item.name.to_string(),
        start_time: item.start_time,
        creation_time: item.creation_time,
        exe_path: exe_opt(item),
    }
}
pub(crate) fn resolve_group_targets(
    item: &ProcessCpuItem,
    group_clones: bool,
    raw: &[RawProcessCpuRow],
) -> Vec<GroupTarget> {
    if !group_clones || item.clone_count <= 1 {
        return vec![leader(item, raw)];
    }
    let le: Option<String> = raw
        .iter()
        .find(|r| r.pid == item.pid && r.start_time == item.start_time)
        .and_then(|r| r.exe_path.as_deref().map(str::to_owned))
        .or_else(|| exe_opt(item));
    let v: Vec<GroupTarget> = raw
        .iter()
        .filter(|r| r.pid != 0 && r.pid != 4 && r.name.eq_ignore_ascii_case(&item.name))
        .filter(|r| match (&le, r.exe_path.as_deref()) {
            (Some(a), Some(b)) => a.eq_ignore_ascii_case(b),
            _ => true,
        })
        .map(|r| GroupTarget {
            pid: r.pid,
            name: r.name.to_string(),
            start_time: r.start_time,
            creation_time: r.creation_time,
            exe_path: r.exe_path.as_deref().map(str::to_owned),
        })
        .collect();
    if v.is_empty() {
        vec![leader(item, raw)]
    } else {
        v
    }
}
pub fn arm_context_menu_kill(state: &mut CpuTableState, item: &ProcessCpuItem) {
    state.selected_pid = Some(item.pid);
    state.selected_start_time = Some(item.start_time);
    state.selected_name = Some(item.name.to_string());
    state.confirm_kill = Some((item.pid, Instant::now()));
    let m = format!("Confirm Kill for {} (PID {}) in bottom bar (3s)", item.name, item.pid);
    state.last_action_msg = Some((m, Instant::now()));
}
pub fn show_row_menu(
    ui: &mut egui::Ui,
    item: &ProcessCpuItem,
    raw: &[RawProcessCpuRow],
    state: &mut CpuTableState,
) {
    ui.set_min_width(240.0);
    let h = format!("{}\nPID {}", item.name, item.pid);
    ui.label(egui::RichText::new(h).monospace().size(11.0).color(colors::TEXT_PRIMARY));
    ui.separator();
    let tg = |s: &CpuTableState| resolve_group_targets(item, s.group_clones, raw);
    if ui.button("Trim Working Set (Free RAM)").clicked() {
        spawn_trim(state, tg(state), item.name.to_string());
        ui.close_menu();
    }
    let on = item.eco_qos == Some(true);
    let lab =
        if on { "Disable EcoQoS (Efficiency Mode)" } else { "Enable EcoQoS (Efficiency Mode)" };
    if ui.button(lab).clicked() {
        spawn_eco(state, tg(state), item.name.to_string(), !on);
        ui.close_menu();
    }
    if ui.button("Open File Location").clicked() {
        spawn_open(state, tg(state));
        ui.close_menu();
    }
    ui.separator();
    let k = egui::RichText::new("Terminate Process").color(colors::ACCENT_RED);
    if ui.add(egui::Button::new(k)).clicked() {
        arm_context_menu_kill(state, item);
        ui.close_menu();
    }
    ui.separator();
    if ui.button("Copy Name & PID").clicked() {
        ui.output_mut(|o| o.copied_text = format!("{} (PID {})", item.name, item.pid));
        ui.close_menu();
    }
}
pub fn render_sort_button(
    ui: &mut egui::Ui,
    label: &str,
    is_active: bool,
    sort_asc: bool,
) -> egui::Response {
    let arrow = if sort_asc { "\u{25B2}" } else { "\u{25BC}" };
    let text = if is_active { format!("{label} {arrow}") } else { label.to_owned() };
    let color = if is_active { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY };
    let fill = if is_active { colors::ACCENT_CYAN.gamma_multiply(0.1) } else { colors::BG_CARD };
    let b = egui::Button::new(egui::RichText::new(text).monospace().size(11.0).color(color));
    ui.add(b.frame(false).fill(fill)).on_hover_text("Click to sort")
}
pub(crate) fn action_button(
    ui: &mut egui::Ui,
    enabled: bool,
    label: &str,
    color: egui::Color32,
) -> egui::Response {
    let b = egui::Button::new(egui::RichText::new(label).monospace().size(11.0).color(color));
    let b = b.fill(color.gamma_multiply(0.10));
    if enabled {
        ui.add(b)
    } else {
        ui.add_enabled(false, b)
    }
}
