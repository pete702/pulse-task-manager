//! Process row context menu and sort button for PULSE CPU table.
use crate::modules::cpu::kill_modal::is_protected_pid;
use crate::modules::cpu::model::{ProcessCpuItem, RawProcessCpuRow};
use crate::modules::cpu::table_menu_actions::{spawn_affinity, spawn_eco, spawn_open, spawn_trim};
use crate::modules::cpu::table_state::CpuTableState;
use crate::platform::hybrid_cores::AffinityMode;
use crate::platform::qos::is_system_process;
use crate::theme::colors;
use crate::ui::table_kit;
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
    if le.is_none() {
        return vec![leader(item, raw)];
    }
    let v: Vec<GroupTarget> = raw
        .iter()
        .filter(|r| r.pid != 0 && r.pid != 4 && r.name.eq_ignore_ascii_case(&item.name))
        .filter(|r| match (&le, r.exe_path.as_deref()) {
            (Some(a), Some(b)) => a.eq_ignore_ascii_case(b),
            (Some(_), None) => false,
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
pub fn arm_context_menu_kill(
    state: &mut CpuTableState,
    item: &ProcessCpuItem,
    group_clones: bool,
    raw: &[RawProcessCpuRow],
) {
    state.selected_pid = Some(item.pid);
    state.selected_start_time = Some(item.start_time);
    state.selected_name = Some(item.name.to_string());
    let targets = resolve_group_targets(item, group_clones, raw);
    state.confirm_kill = Some((item.pid, targets, Instant::now()));
    let m = format!("Confirm Kill for {} (PID {}) in bottom bar (3s)", item.name, item.pid);
    state.last_action_msg = Some((m, Instant::now()));
}
pub(crate) use crate::modules::cpu::table_menu_proc::{spawn_resume, spawn_suspend};

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
    let (lab, eco_hover) = match item.eco_qos {
        Some(true) => {
            ("Disable EcoQoS (Efficiency Mode)", "EcoQoS status: Active (Efficiency Mode)")
        }
        Some(false) => ("Enable EcoQoS (Efficiency Mode)", "EcoQoS status: Off (Normal Priority)"),
        None => (
            "Enable EcoQoS (Efficiency Mode) [unprobed]",
            "EcoQoS status: Unknown (awaiting background probe)",
        ),
    };
    if ui.button(lab).on_hover_text(eco_hover).clicked() {
        let enable = item.eco_qos != Some(true);
        spawn_eco(state, tg(state), item.name.to_string(), enable);
        ui.close_menu();
    }
    if ui.button("Open File Location").clicked() {
        spawn_open(state, tg(state));
        ui.close_menu();
    }
    let path_opt = exe_opt(item);
    let path = path_opt.as_deref().unwrap_or("");
    let self_pid = std::process::id();
    let blocked =
        crate::platform::suspend::is_suspend_blocklisted(&item.name, path, item.pid, self_pid);
    let suspended =
        crate::platform::suspend::is_process_suspended(item.pid, item.creation_time.unwrap_or(0));
    let label = if suspended { "Resume Process" } else { "Suspend Process" };
    let resp = ui.add_enabled(!blocked, egui::Button::new(label));
    if blocked {
        resp.on_hover_text("Suspend blocked: protected or critical process");
    } else if resp.clicked() {
        let targets = tg(state);
        if suspended {
            spawn_resume(state, targets, item.name.to_string());
        } else {
            spawn_suspend(state, targets, item.name.to_string());
        }
        ui.close_menu();
    }
    ui.separator();
    let protected = is_protected_pid(item.pid) || is_system_process(&item.name);
    let hybrid_masks = if crate::platform::hybrid_cores::is_hybrid_cpu() {
        crate::platform::hybrid_cores::query_hybrid_core_masks()
    } else {
        None
    };
    if let Some((p_mask, e_mask)) = hybrid_masks {
        let blocked = crate::platform::protected::is_process_protected(item.pid, path);
        for (label, tip, mode) in [
            ("[ P-Cores ]", "Pin process to Performance cores", AffinityMode::PCores),
            ("[ E-Cores ]", "Pin process to Efficiency cores", AffinityMode::ECores),
            ("[ All Cores ]", "Reset process affinity to all CPU cores", AffinityMode::AllCores),
        ] {
            let button = egui::Button::new(egui::RichText::new(label).monospace().size(11.0));
            let response = ui.add_enabled(!blocked, button).on_hover_text(tip);
            if response.clicked() {
                let targets = resolve_group_targets(item, state.group_clones, raw);
                spawn_affinity(state, targets, p_mask, e_mask, mode);
                ui.close_menu();
            }
        }
        ui.separator();
    }
    let k = egui::RichText::new("Terminate Process").color(if protected {
        colors::TEXT_SECONDARY
    } else {
        colors::ACCENT_RED
    });
    let resp = ui.add_enabled(!protected, egui::Button::new(k));
    if protected {
        resp.on_hover_text("Protected system or self process");
    } else if resp.clicked() {
        let gc = state.group_clones;
        arm_context_menu_kill(state, item, gc, raw);
        ui.close_menu();
    }
    ui.separator();
    if ui.button("Copy Name & PID").clicked() {
        ui.output_mut(|o| o.copied_text = format!("{} (PID {})", item.name, item.pid));
        ui.close_menu();
    }
    if ui.button("Copy Full Row").clicked() {
        let (path, cmd, user) = raw
            .iter()
            .find(|r| r.pid == item.pid && r.start_time == item.start_time)
            .map(|r| {
                (
                    r.exe_path.as_deref().unwrap_or(""),
                    r.command_line.as_deref().unwrap_or(""),
                    r.user.as_deref().unwrap_or(""),
                )
            })
            .unwrap_or(("", "", ""));
        let meta =
            crate::modules::cpu::proc_meta::get_or_cache(item.pid, item.start_time, cmd, user);
        let line = crate::modules::cpu::proc_meta::format_process_copy_line(
            &item.name,
            item.pid,
            &item.ram_text,
            &item.cpu_text,
            path,
            &meta.command_line,
        );
        ui.output_mut(|o| o.copied_text = line);
        ui.close_menu();
    }
}
pub fn render_sort_button(
    ui: &mut egui::Ui,
    label: &str,
    is_active: bool,
    sort_asc: bool,
) -> egui::Response {
    let text = table_kit::sort_header_text(label, is_active, sort_asc);
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
