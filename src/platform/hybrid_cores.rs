//! Hybrid CPU detection and verified process affinity pinning (Windows only).
use crate::platform::suspend::ActionResult;
use std::sync::OnceLock;
const REL: u32 = 0;
const AFFINITY_ACCESS: u32 = 0x0200 | 0x1000;

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
extern "system" {
    fn GetLogicalProcessorInformationEx(
        relationship: u32,
        buffer: *mut u8,
        length: *mut u32,
    ) -> i32;
    fn SetProcessAffinityMask(handle: isize, mask: usize) -> i32;
}

#[repr(C)]
#[derive(Clone, Copy)]
struct GroupAffinity {
    mask: usize,
    group: u16,
    reserved: [u16; 3],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ProcessorRelationship {
    flags: u8,
    efficiency_class: u8,
    reserved: [u8; 20],
    group_count: u16,
    group_mask: [GroupAffinity; 1],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct LogicalProcessorInfoEx {
    relationship: u32,
    size: u32,
    processor: ProcessorRelationship,
}

/// Reads one record from an 8-byte aligned `Vec<u64>` at any byte offset.
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
fn read_record(words: &[u64], offset: usize) -> LogicalProcessorInfoEx {
    let ptr = unsafe { (words.as_ptr() as *const u8).add(offset) } as *const LogicalProcessorInfoEx;
    unsafe { std::ptr::read_unaligned(ptr) }
}
/// Strict bounded parse of `GetLogicalProcessorInformationEx` bytes.
#[cfg(target_os = "windows")]
fn parse_core_records(words: &[u64], used: usize) -> (Vec<(u8, usize)>, bool) {
    let need = std::mem::size_of::<LogicalProcessorInfoEx>();
    let len = used.min(words.len() * 8);
    let mut cores = Vec::new();
    let (mut offset, mut bad) = (0usize, false);
    while offset + need <= len {
        let record = read_record(words, offset);
        let size = record.size as usize;
        if size < need || offset + size > len {
            bad = true;
            break;
        }
        if record.relationship == REL && record.processor.group_count == 1 {
            cores.push((record.processor.efficiency_class, record.processor.group_mask[0].mask));
        }
        offset += size;
    }
    (cores, bad)
}

/// Pure helper: split physical-core masks into P-core and E-core masks.
pub fn compute_hybrid_masks(cores: &[(u8, usize)], total: usize) -> Option<(usize, usize)> {
    if total == 0 || total > 64 || cores.is_empty() {
        return None;
    }
    let min_class = cores.iter().map(|c| c.0).min()?;
    let max_class = cores.iter().map(|c| c.0).max()?;
    if min_class == max_class {
        return None;
    }
    let mut p_mask = 0usize;
    let mut e_mask = 0usize;
    for (class, mask) in cores {
        if *class == max_class {
            p_mask |= *mask;
        } else {
            e_mask |= *mask;
        }
    }
    (p_mask != 0 && e_mask != 0).then_some((p_mask, e_mask))
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
fn windows_detect() -> Option<(usize, usize)> {
    let mut length = 0u32;
    unsafe { GetLogicalProcessorInformationEx(REL, std::ptr::null_mut(), &mut length) };
    if length == 0 {
        return None;
    }
    let mut buffer = vec![0u64; (length as usize).div_ceil(8)];
    let ptr = buffer.as_mut_ptr() as *mut u8;
    if unsafe { GetLogicalProcessorInformationEx(REL, ptr, &mut length) } == 0 {
        return None;
    }
    let (cores, _) = parse_core_records(&buffer, length as usize);
    let total: usize = cores.iter().map(|(_, mask)| mask.count_ones() as usize).sum();
    compute_hybrid_masks(&cores, total)
}

static MASKS: OnceLock<Option<(usize, usize)>> = OnceLock::new();
/// Cached `(p_mask, e_mask)`; `None` on homogeneous or >64-logical-CPU systems.
pub fn query_hybrid_core_masks() -> Option<(usize, usize)> {
    #[cfg(target_os = "windows")]
    let detect = windows_detect;
    #[cfg(not(target_os = "windows"))]
    let detect = || None;
    *MASKS.get_or_init(detect)
}

pub fn is_hybrid_cpu() -> bool {
    query_hybrid_core_masks().is_some()
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AffinityMode {
    PCores,
    ECores,
    AllCores,
}

impl AffinityMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::PCores => "P-Cores",
            Self::ECores => "E-Cores",
            Self::AllCores => "All Cores",
        }
    }

    fn mask(self, p_mask: usize, e_mask: usize) -> usize {
        match self {
            Self::PCores => p_mask,
            Self::ECores => e_mask,
            Self::AllCores => p_mask | e_mask,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AffinityTarget {
    pub pid: u32,
    pub start_time: u64,
    pub exe_path: Option<String>,
}

fn blocked(pid: u32, reason: &str) -> ActionResult {
    ActionResult::failure(format!("PID {pid}: affinity blocked ({reason})"))
}

/// Rejects protected, critical, self, and identity-mismatched targets before pinning.
pub fn set_process_affinity_verified(
    pid: u32,
    expected_start_time: u64,
    expected_exe_path: &str,
    mask: usize,
    label: &str,
) -> ActionResult {
    if pid <= 4
        || pid == std::process::id()
        || crate::platform::protected::is_process_protected(pid, expected_exe_path)
    {
        return blocked(pid, "protected process");
    }
    if expected_start_time == 0 {
        return blocked(pid, "Identity unavailable");
    }
    #[cfg(target_os = "windows")]
    {
        use crate::platform::handle::{
            is_handle_critical, query_process_creation_time, query_process_image_path, SafeHandle,
        };
        use crate::platform::protected::full_paths_match;
        use crate::platform::suspend::filetime_to_unix_secs;
        let Some(handle) = SafeHandle::open(AFFINITY_ACCESS, pid) else {
            return blocked(pid, "cannot open");
        };
        let Some(filetime) = query_process_creation_time(&handle) else {
            return blocked(pid, "no creation time");
        };
        if filetime_to_unix_secs(filetime).abs_diff(expected_start_time) > 2 {
            return blocked(pid, "start mismatch");
        }
        let Some(live_path) = query_process_image_path(&handle) else {
            return blocked(pid, "no image path");
        };
        if !expected_exe_path.trim().is_empty() && !full_paths_match(expected_exe_path, &live_path)
        {
            return blocked(pid, "path mismatch");
        }
        if is_handle_critical(&handle) {
            return blocked(pid, "critical process");
        }
        #[allow(unsafe_code)]
        let set = unsafe { SetProcessAffinityMask(handle.as_raw(), mask) };
        if set == 0 {
            return ActionResult::failure(format!("PID {pid}: affinity set failed"));
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (expected_start_time, expected_exe_path, mask, label);
        return ActionResult::failure("Affinity unavailable on this platform".to_owned());
    }
    let message = if label == "All Cores" {
        format!("PID {pid}: affinity reset to All Cores")
    } else {
        format!("PID {pid}: pinned to {label} (mask 0x{mask:X})")
    };
    ActionResult::success(message)
}

/// Runs the single/multi-PID affinity loop; returns the success count and status message.
pub fn apply_affinity_to_targets(
    targets: &[AffinityTarget],
    p_mask: usize,
    e_mask: usize,
    mode: AffinityMode,
) -> (usize, String) {
    if targets.is_empty() {
        return (0, "Affinity unchanged (no processes)".to_owned());
    }
    let mask = mode.mask(p_mask, e_mask);
    let (mut ok, mut last_error, mut last_ok) = (0usize, String::new(), String::new());
    for target in targets {
        let path = target.exe_path.as_deref().unwrap_or("");
        let result =
            set_process_affinity_verified(target.pid, target.start_time, path, mask, mode.label());
        if result.ok {
            ok += 1;
            last_ok = result.message;
        } else {
            last_error = result.message;
        }
    }
    let message = if targets.len() == 1 {
        if ok == 1 {
            last_ok
        } else {
            last_error
        }
    } else if ok == targets.len() {
        format!("Pinned {ok}/{} processes to {}", targets.len(), mode.label())
    } else {
        format!("Pinned {ok}/{} processes to {} (last: {last_error})", targets.len(), mode.label())
    };
    (ok, message)
}

#[cfg(test)]
#[path = "hybrid_cores_tests.rs"]
mod tests;
