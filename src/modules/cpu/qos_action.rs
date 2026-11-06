//! Process actions for the CPU table context menu.
//! Provides EcoQoS, working-set trim, file location, and verified termination.
//! All mutating actions reuse the hardened Memory module identity checks.

use std::path::Path;

use windows_sys::Win32::System::Threading::{
    GetPriorityClass, GetProcessInformation, ProcessPowerThrottling, SetPriorityClass,
    SetProcessInformation, IDLE_PRIORITY_CLASS, PROCESS_POWER_THROTTLING_CURRENT_VERSION,
    PROCESS_POWER_THROTTLING_EXECUTION_SPEED, PROCESS_POWER_THROTTLING_STATE,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION,
};

use crate::modules::cpu::model::ProcessCpuItem;
use crate::modules::memory::actions::{
    open_process_file_location as open_path, terminate_process_group, trim_single_pid,
    TrimSkipReason,
};
use crate::modules::memory::process_target::ProcessTarget;
use crate::modules::memory::safe_handle::{
    is_handle_critical, query_creation_time_by_pid, query_process_image_path,
    verify_handle_identity, verify_pid_safety_gate, SafeHandle,
};

/// System process names that should never be targeted by mutating actions.
const SYSTEM_PROCESS_BLOCKLIST: &[&str] = &[
    "system",
    "registry",
    "smss.exe",
    "csrss.exe",
    "wininit.exe",
    "services.exe",
    "lsass.exe",
    "svchost.exe",
    "fontdrvhost.exe",
    "dwm.exe",
    "taskhostw.exe",
    "explorer.exe",
    "winlogon.exe",
    "audiodg.exe",
    "sihost.exe",
    "startmenuexperiencehost.exe",
    "textinputhost.exe",
    "runtimebroker.exe",
    "shellexperiencehost.exe",
    "searchindexer.exe",
    "searchprotocolhost.exe",
    "searchfilterhost.exe",
    "spoolsv.exe",
];

/// Checks if a process name is in the system blocklist.
fn is_system_process(name: &str) -> bool {
    SYSTEM_PROCESS_BLOCKLIST.iter().any(|&s| name.eq_ignore_ascii_case(s))
}

/// Builds a verified target using creation time queried at action time.
/// CPU rows intentionally do not collect creation time during scans.
fn target_for(item: &ProcessCpuItem) -> Result<ProcessTarget, String> {
    let creation_time = query_creation_time_by_pid(item.pid)
        .ok_or_else(|| "Cannot read process creation time".to_string())?;
    Ok(ProcessTarget {
        pid: item.pid,
        creation_time: Some(creation_time),
        name: item.name.to_string(),
        exe_path: None,
    })
}

/// Enables EcoQoS (Efficiency Mode) by setting IDLE_PRIORITY_CLASS.
#[allow(unsafe_code)]
pub fn enable_eco_qos(pid: u32, name: &str) -> Result<(), String> {
    if is_system_process(name) {
        return Err("System process is protected".to_string());
    }
    verify_pid_safety_gate(pid, name).map_err(str::to_string)?;
    let creation_time = query_creation_time_by_pid(pid)
        .ok_or_else(|| "Cannot read process creation time".to_string())?;

    let handle = SafeHandle::open(PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_INFORMATION, pid)
        .ok_or_else(|| "Access denied while opening process".to_string())?;

    if is_handle_critical(&handle)
        || !verify_handle_identity(&handle, name, Some(creation_time), None)
    {
        return Err("Process identity verification failed".to_string());
    }

    let result = unsafe { SetPriorityClass(handle.as_raw(), IDLE_PRIORITY_CLASS) };
    if result == 0 {
        return Err("Failed to set IDLE_PRIORITY_CLASS".to_string());
    }
    // Best-effort true EcoQoS: Power Throttling execution-speed control.
    let state = PROCESS_POWER_THROTTLING_STATE {
        Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
        ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
        StateMask: 0,
    };
    unsafe {
        let _ = SetProcessInformation(
            handle.as_raw(),
            ProcessPowerThrottling,
            &state as *const _ as *const std::os::raw::c_void,
            std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        );
    }
    Ok(())
}

/// Returns Some(true) when PID runs in EcoQoS (idle priority or power throttling).
/// Returns None when the process cannot be opened or queried.
#[allow(unsafe_code)]
pub fn is_process_eco_qos(pid: u32) -> Option<bool> {
    if pid == 0 || pid == 4 {
        return Some(false);
    }
    let handle = SafeHandle::open(PROCESS_QUERY_LIMITED_INFORMATION, pid)?;
    let priority = unsafe { GetPriorityClass(handle.as_raw()) };
    if priority == IDLE_PRIORITY_CLASS {
        return Some(true);
    }
    let mut state: PROCESS_POWER_THROTTLING_STATE = unsafe { std::mem::zeroed() };
    let ok = unsafe {
        GetProcessInformation(
            handle.as_raw(),
            ProcessPowerThrottling,
            &mut state as *mut _ as *mut _,
            std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        )
    };
    if ok != 0 {
        return Some(state.StateMask & PROCESS_POWER_THROTTLING_EXECUTION_SPEED != 0);
    }
    Some(false)
}

/// Trims the working set of a process to release physical RAM.
pub fn trim_process_working_set(item: &ProcessCpuItem) -> Result<(), String> {
    if is_system_process(item.name.as_ref()) {
        return Err("System process is protected".to_string());
    }
    let target = target_for(item)?;
    trim_single_pid(&target).map(|_| ()).map_err(|reason| match reason {
        TrimSkipReason::Protected => "Protected process".to_string(),
        TrimSkipReason::AccessDenied => "Access denied".to_string(),
        TrimSkipReason::CriticalProcess => "Critical process".to_string(),
    })
}

/// Opens the executable folder in Explorer.
pub fn open_process_file_location(pid: u32) -> Result<(), String> {
    let handle = SafeHandle::open_process(pid)
        .ok_or_else(|| "Access denied while opening process".to_string())?;
    let path = query_process_image_path(&handle)
        .ok_or_else(|| "Cannot resolve executable path".to_string())?;
    open_path(Path::new(&path))
}

/// Terminates a process after safety-gate and identity verification.
pub fn terminate_process(item: &ProcessCpuItem) -> Result<(), String> {
    if is_system_process(item.name.as_ref()) {
        return Err("System process is protected".to_string());
    }
    let target = target_for(item)?;
    match terminate_process_group(&[target])? {
        0 => Err("Process was not terminated (identity or access check failed)".to_string()),
        _ => Ok(()),
    }
}

/// Renders the right-click action menu for one CPU process row.
#[allow(dead_code)]
pub fn show_cpu_process_context_menu(ui: &mut eframe::egui::Ui, item: &ProcessCpuItem) {
    use eframe::egui;

    let blocked = is_system_process(item.name.as_ref());
    let pid = item.pid;
    let name = item.name.to_string();

    ui.label(
        egui::RichText::new(&name).monospace().size(12.0).color(crate::theme::colors::TEXT_PRIMARY),
    );
    ui.label(
        egui::RichText::new(format!("PID {pid}"))
            .monospace()
            .size(10.0)
            .color(crate::theme::colors::TEXT_SECONDARY),
    );
    ui.separator();

    if ui.add_enabled(!blocked, egui::Button::new("Enable EcoQoS (Efficiency Mode)")).clicked() {
        let _ = enable_eco_qos(pid, &name);
        ui.close_menu();
    }

    if ui.add_enabled(!blocked, egui::Button::new("Trim Working Set (Free RAM)")).clicked() {
        let _ = trim_process_working_set(item);
        ui.close_menu();
    }

    if ui.button("Open File Location").clicked() {
        let _ = open_process_file_location(pid);
        ui.close_menu();
    }

    ui.separator();

    if ui
        .add_enabled(
            !blocked,
            egui::Button::new(
                egui::RichText::new("Terminate Process").color(crate::theme::colors::ACCENT_RED),
            ),
        )
        .clicked()
    {
        let _ = terminate_process(item);
        ui.close_menu();
    }

    if blocked {
        ui.label(
            egui::RichText::new("Mutating actions blocked for system process")
                .italics()
                .size(10.0)
                .color(crate::theme::colors::TEXT_SECONDARY),
        );
    }

    ui.separator();
    if ui.button("Copy PID").clicked() {
        ui.output_mut(|o| o.copied_text = pid.to_string());
        ui.close_menu();
    }
    if ui.button("Copy Name").clicked() {
        ui.output_mut(|o| o.copied_text = name);
        ui.close_menu();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_system_process() {
        assert!(is_system_process("System"));
        assert!(is_system_process("svchost.exe"));
        assert!(is_system_process("lsass.exe"));
        assert!(!is_system_process("chrome.exe"));
        assert!(!is_system_process("notepad.exe"));
        assert!(!is_system_process("myapp"));
    }
}
