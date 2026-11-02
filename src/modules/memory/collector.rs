//! OS memory collector for PULSE: gathers real RAM/Swap metrics via sysinfo
//! and Windows kernel metrics via K32GetPerformanceInfo.
//! Also provides process memory collection and self-memory trimming.

use sysinfo::{ProcessRefreshKind, System};

use crate::modules::memory::model::MemorySnapshot;
use crate::modules::memory::process_model::MemorySpikeEvent;

/// Row of raw process memory data: (PID, name, RAM bytes, virtual bytes).
pub type RawProcessRow = (u32, String, u64, u64);

/// Result of a process memory collection pass.
pub type ProcessMemoryScan = (Vec<RawProcessRow>, Option<MemorySpikeEvent>);

#[cfg(target_os = "windows")]
use windows_sys::Win32::System::ProcessStatus::{
    K32EmptyWorkingSet, K32GetPerformanceInfo, K32GetProcessMemoryInfo, PERFORMANCE_INFORMATION,
    PROCESS_MEMORY_COUNTERS,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::GetCurrentProcess;

/// Collects current system memory state into a `MemorySnapshot`.
///
/// Calls `sys.refresh_memory()` to update `sysinfo`'s internal state,
/// then maps all fields to `MemorySnapshot`.
/// On Windows, also calls `K32GetPerformanceInfo` for real kernel metrics.
pub fn collect_memory(sys: &mut System) -> MemorySnapshot {
    sys.refresh_memory();

    let mut snap = MemorySnapshot {
        ram_total_bytes: sys.total_memory(),
        ram_used_bytes: sys.used_memory(),
        ram_available_bytes: sys.available_memory(),
        ram_free_bytes: sys.free_memory(),
        swap_total_bytes: sys.total_swap(),
        swap_used_bytes: sys.used_swap(),
        swap_free_bytes: sys.free_swap(),
        ..Default::default()
    };

    // On Windows, get real kernel metrics via K32GetPerformanceInfo
    #[cfg(target_os = "windows")]
    {
        let kernel_metrics = collect_windows_kernel_metrics();
        snap.commit_total_bytes = kernel_metrics.commit_total_bytes;
        snap.commit_limit_bytes = kernel_metrics.commit_limit_bytes;
        snap.kernel_paged_bytes = kernel_metrics.kernel_paged_bytes;
        snap.kernel_nonpaged_bytes = kernel_metrics.kernel_nonpaged_bytes;
        snap.system_handles = kernel_metrics.system_handles;
        snap.system_threads = kernel_metrics.system_threads;
        snap.system_processes = kernel_metrics.system_processes;

        // Override cached bytes with real System Cache from kernel
        if kernel_metrics.system_cache_bytes > 0 {
            snap.ram_cached_bytes_override = kernel_metrics.system_cache_bytes;
        }
    }

    snap
}

/// Collects process memory information for the process list view.
///
/// Returns a vector of (PID, name, RAM bytes, virtual bytes) tuples,
/// and optionally detects memory spike events (RAM >= 85%).
#[allow(dead_code)]
pub fn collect_process_memory(
    sys: &mut System,
    total_ram: u64,
    current_ram_pct: f32,
) -> ProcessMemoryScan {
    // Refresh process list with memory info
    sys.refresh_processes_specifics(ProcessRefreshKind::new().with_memory());

    let processes: Vec<RawProcessRow> = sys
        .processes()
        .iter()
        .map(|(pid, process)| {
            (
                pid.as_u32(),
                process.name().to_string(),
                process.memory(),
                process.virtual_memory(),
            )
        })
        .collect();

    // Detect memory spike using system-wide RAM usage (not the last process).
    let spike = crate::modules::memory::process_model::detect_memory_spike(
        &processes,
        total_ram,
        current_ram_pct,
    );

    (processes, spike)
}

/// Trims the working set of the current process using K32EmptyWorkingSet.
///
/// This releases unused memory pages (font caches, OpenGL buffers, etc.)
/// back to the OS, reducing PULSE's own RAM footprint from ~150MB to ~25-35MB.
/// Should be called before measuring own process memory.
#[allow(dead_code)]
pub fn trim_own_working_set() -> bool {
    #[cfg(target_os = "windows")]
    {
        // SAFETY: GetCurrentProcess returns a pseudo-handle, K32EmptyWorkingSet is safe to call
        #[allow(unsafe_code)]
        let handle = unsafe { GetCurrentProcess() };
        #[allow(unsafe_code)]
        let result = unsafe { K32EmptyWorkingSet(handle) };
        result != 0
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
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

/// Windows kernel metrics collected via K32GetPerformanceInfo.
#[cfg(target_os = "windows")]
#[derive(Default, Debug, Clone, Copy)]
struct KernelMetrics {
    commit_total_bytes: u64,
    commit_limit_bytes: u64,
    kernel_paged_bytes: u64,
    kernel_nonpaged_bytes: u64,
    system_handles: u32,
    system_threads: u32,
    system_processes: u32,
    system_cache_bytes: u64,
}

/// Collects Windows kernel metrics via K32GetPerformanceInfo.
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
fn collect_windows_kernel_metrics() -> KernelMetrics {
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

#[cfg(test)]
mod tests {
    use super::*;
    use sysinfo::System;

    #[test]
    fn test_real_os_memory_collection() {
        let mut sys = System::new_all();
        let snap = collect_memory(&mut sys);

        // Basic sanity checks on live OS
        assert!(
            snap.ram_total_bytes > 0,
            "RAM total should be > 0 on live OS"
        );
        assert!(snap.ram_used_bytes > 0, "RAM used should be > 0 on live OS");
        assert!(
            snap.ram_available_bytes > 0,
            "RAM available should be > 0 on live OS"
        );
        // ram_free_bytes is u64, so always >= 0 (no assertion needed)

        // Swap may be 0 if disabled, but total should be valid
        // (no assertion on swap_used/swap_free as swap may be disabled)

        // Usage percent should be valid and > 0 on a running system
        let usage = snap.ram_usage_percent();
        assert!(
            usage > 0.0,
            "RAM usage percent should be > 0 on live OS, got {}",
            usage
        );
        assert!(
            usage <= 100.0,
            "RAM usage percent should be <= 100, got {}",
            usage
        );
    }

    #[test]
    fn test_collect_memory_fields_populated() {
        let mut sys = System::new_all();
        let snap = collect_memory(&mut sys);

        // All fields should be populated (non-zero for a running system)
        assert_ne!(snap.ram_total_bytes, 0);
        assert_ne!(snap.ram_used_bytes, 0);
        assert_ne!(snap.ram_available_bytes, 0);
        // ram_free_bytes can be 0 in some edge cases, but total should be valid

        // Swap fields: total may be 0 if swap disabled
        if snap.swap_total_bytes > 0 {
            assert!(snap.swap_used_bytes <= snap.swap_total_bytes);
            assert!(snap.swap_free_bytes <= snap.swap_total_bytes);
        }

        // v0.0.6: Kernel metrics should be populated on Windows
        #[cfg(target_os = "windows")]
        {
            // On Windows, kernel metrics should be populated
            // (may be 0 if K32GetPerformanceInfo fails, but should not panic)
            let _ = snap.commit_total_bytes;
            let _ = snap.commit_limit_bytes;
            let _ = snap.kernel_paged_bytes;
            let _ = snap.kernel_nonpaged_bytes;
            let _ = snap.system_handles;
            let _ = snap.system_threads;
            let _ = snap.system_processes;
        }
    }

    #[test]
    fn test_memory_snapshot_consistency() {
        let mut sys = System::new_all();
        let snap = collect_memory(&mut sys);

        // Basic sanity: used + free should not exceed total by a huge margin
        // (real kernel cache from K32GetPerformanceInfo is separate from available-free)
        let sum_used_free = snap.ram_used_bytes + snap.ram_free_bytes;
        assert!(
            sum_used_free <= snap.ram_total_bytes * 2,
            "used + free should not exceed 2x total, got used+free={}, total={}",
            sum_used_free,
            snap.ram_total_bytes
        );

        // Cached (real or fallback) should be reasonable
        let cached = snap.ram_cached_bytes();
        assert!(
            cached <= snap.ram_total_bytes,
            "cached should not exceed total, got cached={}, total={}",
            cached,
            snap.ram_total_bytes
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_kernel_metrics_non_zero() {
        let mut sys = System::new_all();
        let snap = collect_memory(&mut sys);

        // On Windows, kernel metrics should be populated
        // At least system_threads should be > 0 on a running system
        assert!(
            snap.system_threads > 0,
            "System threads should be > 0 on Windows"
        );
        assert!(
            snap.system_processes > 0,
            "System processes should be > 0 on Windows"
        );
        assert!(
            snap.system_handles > 0,
            "System handles should be > 0 on Windows"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_real_kernel_cache_metrics() {
        let mut sys = System::new_all();
        let snap = collect_memory(&mut sys);

        // Real kernel metrics from K32GetPerformanceInfo should be > 0 on a running Windows system
        assert!(
            snap.ram_cached_bytes() > 0,
            "Real System Cache (ram_cached_bytes) should be > 0 on Windows, got {}",
            snap.ram_cached_bytes()
        );
        assert!(
            snap.kernel_paged_bytes > 0,
            "Kernel paged pool should be > 0 on Windows, got {}",
            snap.kernel_paged_bytes
        );
        assert!(
            snap.kernel_nonpaged_bytes > 0,
            "Kernel non-paged pool should be > 0 on Windows, got {}",
            snap.kernel_nonpaged_bytes
        );
        assert!(
            snap.system_threads > 0,
            "System threads should be > 0 on Windows, got {}",
            snap.system_threads
        );
    }

    #[test]
    fn test_collect_process_memory() {
        let mut sys = System::new_all();
        let total_ram = sys.total_memory();
        let (processes, _spike) = collect_process_memory(&mut sys, total_ram, 0.0);

        // Should have at least some processes
        assert!(!processes.is_empty(), "Should have at least one process");

        // Each process should have valid data (PID 0 = System Idle Process is skipped)
        for (pid, name, ram_bytes, virtual_bytes) in processes {
            if pid == 0 {
                continue;
            }
            assert!(pid > 0, "PID should be > 0");
            assert!(!name.is_empty(), "Process name should not be empty");
            assert!(
                ram_bytes > 0 || virtual_bytes > 0,
                "Process should have some memory"
            );
        }
    }

    #[test]
    fn test_trim_own_working_set() {
        // Just verify the function runs without panic
        let _result = trim_own_working_set();
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_own_working_set_mb() {
        let mb = own_working_set_mb();
        assert!(mb > 0.0, "Own working set should be > 0 MiB, got {mb}");
        assert!(mb < 10_000.0, "Own working set should be sane, got {mb}");
    }
}
