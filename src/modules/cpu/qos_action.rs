//! Process actions for the CPU table context menu.
//! Provides EcoQoS, working-set trim, file location, and verified termination.
//! All mutating actions reuse the hardened Memory module identity checks.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use windows_sys::Win32::System::Threading::{
    GetPriorityClass, GetProcessInformation, ProcessPowerThrottling, SetPriorityClass,
    SetProcessInformation, IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS,
    PROCESS_POWER_THROTTLING_CURRENT_VERSION, PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
    PROCESS_POWER_THROTTLING_STATE, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION,
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

/// Pending EcoQoS overrides from UI actions to worker cache.
#[allow(clippy::type_complexity)]
pub(crate) static PENDING_ECO: OnceLock<Mutex<HashMap<(u32, u64), Option<bool>>>> = OnceLock::new();
/// Watch cache for EcoQoS state (kept for API compatibility).
#[allow(dead_code, clippy::type_complexity)]
pub(crate) static ECO_WATCH: OnceLock<Mutex<HashMap<(u32, u64), Option<bool>>>> = OnceLock::new();

/// Checks if a process name is protected by the unified safety gate.
pub fn is_system_process(name: &str) -> bool {
    verify_pid_safety_gate(u32::MAX, name).is_err()
}

/// Primes worker EcoQoS cache after UI action.
#[allow(dead_code)]
pub fn mark_pending_eco(pid: u32, start_time: u64, on: bool) {
    let slot = PENDING_ECO.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = slot.lock().unwrap_or_else(|e| e.into_inner());
    if guard.len() > 512 {
        guard.clear();
    }
    guard.insert((pid, start_time), Some(on));
}

/// Alias for mark_pending_eco (watch cache prime).
#[allow(dead_code)]
pub fn set_eco_watch(pid: u32, start_time: u64, on: bool) {
    mark_pending_eco(pid, start_time, on);
}

/// Budgeted EcoQoS probe: decrements budget and queries Win32.
#[allow(dead_code)]
pub fn probe_eco_qos_budgeted(pid: u32, budget: &mut u8) -> Option<Option<bool>> {
    if *budget == 0 {
        return None;
    }
    *budget -= 1;
    Some(is_process_eco_qos(pid))
}

/// Back-compat prime entry (collector re-export).
#[allow(dead_code)]
pub fn set_eco_cached(pid: u32, start_time: u64, on: bool) {
    mark_pending_eco(pid, start_time, on);
}

/// Drops pending EcoQoS prime so next scan re-queries Win32.
#[allow(dead_code)]
pub fn invalidate_eco_cached(pid: u32, start_time: u64) {
    let slot = PENDING_ECO.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = slot.lock().unwrap_or_else(|e| e.into_inner());
    if guard.len() > 512 {
        guard.clear();
    }
    guard.insert((pid, start_time), None);
}

/// Builds a verified target from creation time at action time.
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

/// Throttling masks: ControlMask always set, StateMask only when enabling.
pub(crate) fn build_throttling_masks(enable: bool) -> (u32, u32) {
    let control_mask = PROCESS_POWER_THROTTLING_EXECUTION_SPEED;
    let state_mask = if enable { PROCESS_POWER_THROTTLING_EXECUTION_SPEED } else { 0 };
    (control_mask, state_mask)
}

/// Enables EcoQoS (Efficiency Mode) by setting IDLE_PRIORITY_CLASS.
#[allow(unsafe_code)]
pub fn set_process_eco_qos(pid: u32, name: &str, enable: bool) -> Result<(), String> {
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

    let priority = if enable { IDLE_PRIORITY_CLASS } else { NORMAL_PRIORITY_CLASS };
    let prio_ok = unsafe { SetPriorityClass(handle.as_raw(), priority) } != 0;
    if prio_ok {
    } else {
        return Err("Failed to set priority class".to_string());
    }
    // True EcoQoS: Power Throttling execution-speed control.
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
        ) != 0
    };
    if prio_ok && info_ok {
        Ok(())
    } else {
        Err("Failed to apply power throttling state".to_string())
    }
}

/// Enables EcoQoS (Efficiency Mode) by setting IDLE_PRIORITY_CLASS.
#[allow(unsafe_code)]
pub fn enable_eco_qos(pid: u32, name: &str) -> Result<(), String> {
    set_process_eco_qos(pid, name, true)
}

/// Some(true) when PID runs in EcoQoS (idle priority or power throttling).
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

    #[test]
    fn test_build_throttling_masks() {
        let (on_control, on_state) = build_throttling_masks(true);
        assert_ne!(on_control, 0);
        assert_ne!(on_state, 0);
        let (off_control, off_state) = build_throttling_masks(false);
        assert_ne!(off_control, 0);
        assert_eq!(off_state, 0);
    }
}
