//! Process actions for the CPU table context menu.
//! Provides EcoQoS, working-set trim, file location, and verified termination.
use crate::modules::memory::actions::{
    open_process_file_location as open_path, terminate_process_group, trim_single_pid,
    TrimSkipReason,
};
use crate::modules::memory::process_target::ProcessTarget;
use crate::modules::memory::safe_handle::{
    is_handle_critical, query_process_image_path, verify_handle_identity, verify_pid_safety_gate,
    SafeHandle,
};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::{
    GetPriorityClass, GetProcessInformation, ProcessPowerThrottling, SetPriorityClass,
    SetProcessInformation, IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS,
    PROCESS_POWER_THROTTLING_CURRENT_VERSION, PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
    PROCESS_POWER_THROTTLING_STATE, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION,
};
type EcoMap = HashMap<(u32, u64), Option<bool>>;
pub(crate) static PENDING_ECO: OnceLock<Mutex<EcoMap>> = OnceLock::new();
type PrioMap = HashMap<(u32, u64), u32>;
pub(crate) static ORIG_PRIORITY: OnceLock<Mutex<PrioMap>> = OnceLock::new();
pub fn is_system_process(name: &str) -> bool {
    verify_pid_safety_gate(u32::MAX, name).is_err()
}
fn pending_insert(key: (u32, u64), val: Option<bool>) {
    let slot = PENDING_ECO.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = slot.lock().unwrap_or_else(|e| e.into_inner());
    if guard.len() > 512 {
        guard.clear();
    }
    guard.insert(key, val);
}
#[allow(dead_code)]
pub fn mark_pending_eco(pid: u32, start_time: u64, on: bool) {
    pending_insert((pid, start_time), Some(on));
}
#[allow(dead_code)]
pub fn set_eco_watch(pid: u32, start_time: u64, on: bool) {
    mark_pending_eco(pid, start_time, on);
}
#[allow(dead_code)]
pub fn probe_eco_qos_budgeted(pid: u32, budget: &mut u8) -> Option<Option<bool>> {
    if *budget == 0 {
        return None;
    }
    *budget -= 1;
    Some(is_process_eco_qos(pid))
}
pub(crate) fn target_for(
    pid: u32,
    name: &str,
    ct: Option<u64>,
    exe: Option<String>,
) -> Option<ProcessTarget> {
    #[cfg(target_os = "windows")]
    let creation_time = Some(ct?);
    #[cfg(not(target_os = "windows"))]
    let creation_time = ct;
    Some(ProcessTarget { pid, creation_time, name: name.to_string(), exe_path: exe })
}
#[cfg(target_os = "windows")]
pub(crate) fn build_throttling_masks(enable: bool) -> (u32, u32) {
    let c = PROCESS_POWER_THROTTLING_EXECUTION_SPEED;
    (c, if enable { c } else { 0 })
}
#[cfg(not(target_os = "windows"))]
pub(crate) fn build_throttling_masks(enable: bool) -> (u32, u32) {
    (1, if enable { 1 } else { 0 })
}
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn set_process_eco_qos(
    pid: u32,
    name: &str,
    creation_time: Option<u64>,
    exe_path: Option<String>,
    enable: bool,
) -> Result<(), String> {
    if is_system_process(name) {
        return Err("System process is protected".to_string());
    }
    verify_pid_safety_gate(pid, name).map_err(str::to_string)?;
    let creation_time =
        creation_time.ok_or_else(|| "Cannot read process creation time".to_string())?;
    let handle = SafeHandle::open(PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_INFORMATION, pid)
        .ok_or_else(|| "Access denied while opening process".to_string())?;
    let img = query_process_image_path(&handle);
    let expected_path = exe_path.as_deref().filter(|s| !s.trim().is_empty()).or(img.as_deref());
    if is_handle_critical(&handle)
        || !verify_handle_identity(&handle, name, Some(creation_time), expected_path)
    {
        return Err("Process identity verification failed".to_string());
    }
    if enable {
        let cur = unsafe { GetPriorityClass(handle.as_raw()) };
        if cur != IDLE_PRIORITY_CLASS {
            let slot = ORIG_PRIORITY.get_or_init(|| Mutex::new(HashMap::new()));
            let mut g = slot.lock().unwrap_or_else(|e| e.into_inner());
            if g.len() > 512 {
                g.clear();
            }
            g.entry((pid, creation_time)).or_insert(cur);
        }
    }
    let (control_mask, state_mask) = build_throttling_masks(enable);
    let state = PROCESS_POWER_THROTTLING_STATE {
        Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
        ControlMask: control_mask,
        StateMask: state_mask,
    };
    let info_ok = unsafe {
        SetProcessInformation(
            handle.as_raw(),
            ProcessPowerThrottling,
            &state as *const _ as *const std::os::raw::c_void,
            std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        )
    };
    if info_ok == 0 {
        return Err("Failed to apply power throttling state".to_string());
    }
    let priority = if enable {
        IDLE_PRIORITY_CLASS
    } else {
        ORIG_PRIORITY
            .get()
            .and_then(|s| {
                s.lock().unwrap_or_else(|e| e.into_inner()).get(&(pid, creation_time)).copied()
            })
            .unwrap_or(NORMAL_PRIORITY_CLASS)
    };
    if unsafe { SetPriorityClass(handle.as_raw(), priority) } == 0 {
        return Err("Failed to set priority class".to_string());
    }
    Ok(())
}
#[cfg(not(target_os = "windows"))]
pub fn set_process_eco_qos(
    _: u32,
    _: &str,
    _: Option<u64>,
    _: Option<String>,
    _: bool,
) -> Result<(), String> {
    Err("EcoQoS requires Windows".to_string())
}
pub fn enable_eco_qos(
    pid: u32,
    name: &str,
    ct: Option<u64>,
    exe: Option<String>,
) -> Result<(), String> {
    set_process_eco_qos(pid, name, ct, exe, true)
}
#[cfg(target_os = "windows")]
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
#[cfg(not(target_os = "windows"))]
pub fn is_process_eco_qos(_pid: u32) -> Option<bool> {
    None
}
pub fn trim_process_working_set(
    pid: u32,
    name: &str,
    ct: Option<u64>,
    exe: Option<String>,
) -> Result<(), String> {
    if is_system_process(name) {
        return Err("System process is protected".to_string());
    }
    let target = target_for(pid, name, ct, exe)
        .ok_or_else(|| "Cannot read process creation time".to_string())?;
    trim_single_pid(&target).map(|_| ()).map_err(|reason| match reason {
        TrimSkipReason::Protected => "Protected process".to_string(),
        TrimSkipReason::AccessDenied => "Access denied".to_string(),
        TrimSkipReason::CriticalProcess => "Critical process".to_string(),
    })
}
pub fn open_process_file_location(pid: u32) -> Result<(), String> {
    let handle = SafeHandle::open_process(pid)
        .ok_or_else(|| "Access denied while opening process".to_string())?;
    let path = query_process_image_path(&handle)
        .ok_or_else(|| "Cannot resolve executable path".to_string())?;
    open_path(Path::new(&path))
}
pub fn terminate_process(
    pid: u32,
    name: &str,
    ct: Option<u64>,
    exe: Option<String>,
) -> Result<(), String> {
    if is_system_process(name) {
        return Err("System process is protected".to_string());
    }
    let target = target_for(pid, name, ct, exe)
        .ok_or_else(|| "Cannot read process creation time".to_string())?;
    match terminate_process_group(&[target])? {
        0 => Err("Process was not terminated (identity or access check failed)".to_string()),
        _ => Ok(()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_is_system_process() {
        assert!(
            is_system_process("System")
                && is_system_process("svchost.exe")
                && !is_system_process("chrome.exe")
        );
    }
    #[test]
    fn test_build_throttling_masks() {
        let (on_c, on_s) = build_throttling_masks(true);
        assert!(on_c != 0 && on_s != 0);
        let (off_c, off_s) = build_throttling_masks(false);
        assert!(off_c != 0 && off_s == 0);
    }
    #[cfg(target_os = "windows")]
    #[test]
    fn test_target_for_rejects_none_on_windows() {
        assert!(target_for(123, "app.exe", None, None).is_none());
        assert!(target_for(123, "app.exe", Some(42), None).is_some());
    }
}
