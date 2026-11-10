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
use std::time::Instant;
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::{
    GetPriorityClass, GetProcessInformation, ProcessPowerThrottling, SetPriorityClass,
    SetProcessInformation, IDLE_PRIORITY_CLASS, PROCESS_POWER_THROTTLING_CURRENT_VERSION,
    PROCESS_POWER_THROTTLING_EXECUTION_SPEED, PROCESS_POWER_THROTTLING_STATE,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION,
};
type EcoMap = HashMap<(u32, u64), (Option<bool>, Instant)>;
pub(crate) static PENDING_ECO: OnceLock<Mutex<EcoMap>> = OnceLock::new();
type PrioMap = HashMap<(u32, u64), (u32, Instant)>;
pub(crate) static ORIG_PRIORITY: OnceLock<Mutex<PrioMap>> = OnceLock::new();

pub fn is_system_process(name: &str) -> bool {
    verify_pid_safety_gate(u32::MAX, name).is_err()
}
pub(crate) fn evict_oldest<V>(map: &mut HashMap<(u32, u64), (V, Instant)>, key: (u32, u64)) {
    if map.len() >= 512 && !map.contains_key(&key) {
        if let Some(old) = map.iter().min_by_key(|(_, (_, at))| *at).map(|(k, _)| *k) {
            map.remove(&old);
        }
    }
}
fn pending_insert(key: (u32, u64), val: Option<bool>) {
    let slot = PENDING_ECO.get_or_init(|| Mutex::new(HashMap::new()));
    let mut g = slot.lock().unwrap_or_else(|e| e.into_inner());
    evict_oldest(&mut g, key);
    g.insert(key, (val, Instant::now()));
}
#[allow(dead_code)]
pub fn mark_pending_eco(pid: u32, start_time: u64, on: bool) {
    pending_insert((pid, start_time), Some(on));
}
#[allow(dead_code)]
pub fn probe_eco_qos_budgeted(pid: u32, budget: &mut u8) -> Option<bool> {
    if pid == 0 || pid == 4 {
        return Some(false);
    }
    if *budget == 0 {
        return None;
    }
    let v = is_process_eco_qos(pid);
    if v.is_some() {
        *budget -= 1;
    }
    v
}
pub(crate) fn resolve_restore_priority(key: (u32, u64)) -> Result<u32, String> {
    let slot = ORIG_PRIORITY.get_or_init(|| Mutex::new(HashMap::new()));
    let g = slot.lock().unwrap_or_else(|e| e.into_inner());
    g.get(&key)
        .map(|(p, _)| *p)
        .ok_or_else(|| "No saved priority class for this process".to_string())
}
pub(crate) fn target_for(
    pid: u32,
    name: &str,
    ct: Option<u64>,
    exe: Option<String>,
) -> Option<ProcessTarget> {
    #[cfg(target_os = "windows")]
    ct?;
    Some(ProcessTarget { pid, creation_time: ct, name: name.to_string(), exe_path: exe })
}
pub(crate) fn build_throttling_masks(enable: bool) -> (u32, u32) {
    #[cfg(target_os = "windows")]
    let c = PROCESS_POWER_THROTTLING_EXECUTION_SPEED;
    #[cfg(not(target_os = "windows"))]
    let c = 1u32;
    (c, if enable { c } else { 0 })
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
    let ct = creation_time.ok_or_else(|| "Cannot read process creation time".to_string())?;
    let handle = SafeHandle::open(PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_INFORMATION, pid)
        .ok_or_else(|| "Access denied while opening process".to_string())?;
    let ep = exe_path.as_deref().filter(|s| !s.trim().is_empty());
    if is_handle_critical(&handle) || !verify_handle_identity(&handle, name, Some(ct), ep) {
        return Err("Process identity verification failed".to_string());
    }
    let orig_prio = unsafe { GetPriorityClass(handle.as_raw()) };
    if orig_prio == 0 {
        return Err("Failed to read process priority class".to_string());
    }
    let priority = if enable { IDLE_PRIORITY_CLASS } else { resolve_restore_priority((pid, ct))? };
    if unsafe { SetPriorityClass(handle.as_raw(), priority) } == 0 {
        return Err("Failed to update process priority class for EcoQoS".to_string());
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
            &state as *const _ as *const _,
            std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        )
    };
    if info_ok == 0 {
        let _ = unsafe { SetPriorityClass(handle.as_raw(), orig_prio) };
        return Err("Windows rejected EcoQoS power throttling state".to_string());
    }
    if enable {
        if orig_prio != IDLE_PRIORITY_CLASS {
            let slot = ORIG_PRIORITY.get_or_init(|| Mutex::new(HashMap::new()));
            let mut g = slot.lock().unwrap_or_else(|e| e.into_inner());
            evict_oldest(&mut g, (pid, ct));
            g.entry((pid, ct)).or_insert((orig_prio, Instant::now()));
        }
    } else if let Some(slot) = ORIG_PRIORITY.get() {
        slot.lock().unwrap_or_else(|e| e.into_inner()).remove(&(pid, ct));
    }
    pending_insert((pid, ct), Some(enable));
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
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn is_process_eco_qos(pid: u32) -> Option<bool> {
    if pid == 0 || pid == 4 {
        return Some(false);
    }
    let handle = SafeHandle::open(PROCESS_QUERY_LIMITED_INFORMATION, pid)?;
    if unsafe { GetPriorityClass(handle.as_raw()) } == 0 {
        return None;
    }
    let mut state = PROCESS_POWER_THROTTLING_STATE {
        Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
        ControlMask: 0,
        StateMask: 0,
    };
    let ok = unsafe {
        GetProcessInformation(
            handle.as_raw(),
            ProcessPowerThrottling,
            &mut state as *mut _ as *mut _,
            std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        )
    };
    (ok != 0).then_some(
        (state.ControlMask & PROCESS_POWER_THROTTLING_EXECUTION_SPEED) != 0
            && (state.StateMask & PROCESS_POWER_THROTTLING_EXECUTION_SPEED) != 0,
    )
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
    trim_single_pid(&target).map(|_| ()).map_err(|r| match r {
        TrimSkipReason::Protected => "Protected process".to_string(),
        TrimSkipReason::AccessDenied => "Access denied".to_string(),
        TrimSkipReason::CriticalProcess => "Critical process".to_string(),
    })
}
pub fn open_process_file_location(
    pid: u32,
    name: &str,
    creation_time: Option<u64>,
    exe_path: Option<&str>,
) -> Result<(), String> {
    if is_system_process(name) {
        return Err("System process is protected".to_string());
    }
    if verify_pid_safety_gate(pid, name).is_err() {
        return Err("Process is protected by safety gate".to_string());
    }
    let handle = SafeHandle::open_process(pid)
        .ok_or_else(|| "Access denied while opening process".to_string())?;
    let ep = exe_path.filter(|s| !s.trim().is_empty());
    if is_handle_critical(&handle) || !verify_handle_identity(&handle, name, creation_time, ep) {
        return Err("Blocked by process identity or safety check".to_string());
    }
    let path = ep
        .map(str::to_owned)
        .or_else(|| query_process_image_path(&handle))
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
