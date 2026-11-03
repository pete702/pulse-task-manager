//! Windows API bindings for memory collection: K32GetPerformanceInfo, K32GetProcessMemoryInfo, K32EmptyWorkingSet.
//! All functions require `#[allow(unsafe_code)]` and are Windows-only.

#[cfg(target_os = "windows")]
use windows_sys::Win32::System::ProcessStatus::{
    K32EmptyWorkingSet, K32GetPerformanceInfo, K32GetProcessMemoryInfo, PERFORMANCE_INFORMATION,
    PROCESS_MEMORY_COUNTERS,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::GetCurrentProcess;

/// Windows kernel metrics collected via K32GetPerformanceInfo.
#[cfg(target_os = "windows")]
#[derive(Default, Debug, Clone, Copy)]
pub struct KernelMetrics {
    pub commit_total_bytes: u64,
    pub commit_limit_bytes: u64,
    pub kernel_paged_bytes: u64,
    pub kernel_nonpaged_bytes: u64,
    pub system_handles: u32,
    pub system_threads: u32,
    pub system_processes: u32,
    pub system_cache_bytes: u64,
}

/// Collects Windows kernel metrics via K32GetPerformanceInfo.
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn collect_windows_kernel_metrics() -> KernelMetrics {
    let mut info: PERFORMANCE_INFORMATION = unsafe { std::mem::zeroed() };
    // CRITICAL: cb field MUST be initialized to size of structure before calling API
    info.cb = std::mem::size_of::<PERFORMANCE_INFORMATION>() as u32;

    // SAFETY: Calling Windows API K32GetPerformanceInfo with properly initialized struct
    let result =
        unsafe { K32GetPerformanceInfo(&mut info as *mut PERFORMANCE_INFORMATION, info.cb) };

    if result == 0 {
        // API call failed, return defaults
        return KernelMetrics::default();
    }

    let page_size = info.PageSize as u64;

    KernelMetrics {
        commit_total_bytes: info.CommitTotal as u64 * page_size,
        commit_limit_bytes: info.CommitLimit as u64 * page_size,
        kernel_paged_bytes: info.KernelPaged as u64 * page_size,
        kernel_nonpaged_bytes: info.KernelNonpaged as u64 * page_size,
        system_handles: info.HandleCount,
        system_threads: info.ThreadCount,
        system_processes: info.ProcessCount,
        system_cache_bytes: info.SystemCache as u64 * page_size,
    }
}

/// Non-Windows fallback for kernel metrics.
#[cfg(not(target_os = "windows"))]
#[derive(Default, Debug, Clone, Copy)]
pub struct KernelMetrics {
    pub commit_total_bytes: u64,
    pub commit_limit_bytes: u64,
    pub kernel_paged_bytes: u64,
    pub kernel_nonpaged_bytes: u64,
    pub system_handles: u32,
    pub system_threads: u32,
    pub system_processes: u32,
    pub system_cache_bytes: u64,
}

/// Non-Windows fallback (returns defaults).
#[cfg(not(target_os = "windows"))]
pub fn collect_windows_kernel_metrics() -> KernelMetrics {
    KernelMetrics::default()
}

/// Trims the working set of the current process using K32EmptyWorkingSet.
///
/// This releases unused memory pages (font caches, OpenGL buffers, etc.)
/// back to the OS, reducing PULSE's own RAM footprint from ~150MB to ~25-35MB.
/// Should be called before measuring own process memory.
#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub fn trim_own_working_set() -> bool {
    // SAFETY: GetCurrentProcess returns a pseudo-handle, K32EmptyWorkingSet is safe to call
    #[allow(unsafe_code)]
    let handle = unsafe { GetCurrentProcess() };
    #[allow(unsafe_code)]
    let result = unsafe { K32EmptyWorkingSet(handle) };
    result != 0
}

/// Non-Windows fallback.
#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn trim_own_working_set() -> bool {
    false
}

/// Returns PULSE's own working set in MiB via K32GetProcessMemoryInfo.
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn own_working_set_mb() -> f32 {
    let mut counters: PROCESS_MEMORY_COUNTERS = unsafe { std::mem::zeroed() };
    counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;

    // SAFETY: pseudo-handle is always valid; counters is properly sized and initialized.
    let result =
        unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut counters, counters.cb) };

    if result == 0 {
        0.0
    } else {
        (counters.WorkingSetSize as f64 / (1024.0 * 1024.0)) as f32
    }
}

/// Non-Windows fallback for own working set.
#[cfg(not(target_os = "windows"))]
pub fn own_working_set_mb() -> f32 {
    0.0
}
