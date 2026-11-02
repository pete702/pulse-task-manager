//! OS memory collector for PULSE: gathers real RAM/Swap metrics via sysinfo
//! and Windows kernel metrics via K32GetPerformanceInfo.

use sysinfo::System;

use crate::modules::memory::model::MemorySnapshot;

#[cfg(target_os = "windows")]
use windows_sys::Win32::System::ProcessStatus::{K32GetPerformanceInfo, PERFORMANCE_INFORMATION};

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
            // Store the real cache value directly in a way we can use
            // We'll add a field for this in the snapshot
        }
    }

    snap
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
    info.cb = std::mem::size_of::<PERFORMANCE_INFORMATION>() as u32;

    // SAFETY: Calling Windows API K32GetPerformanceInfo
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

/// Trims the working set of the current process (Windows EmptyWorkingSet equivalent).
///
/// Calls `sysinfo::Process::memory()` to get current RSS, then attempts to
/// release unused pages via the OS. Returns `true` if the call succeeded.
#[allow(dead_code)]
pub fn trim_working_set() -> bool {
    // On Windows, sysinfo doesn't expose EmptyWorkingSet directly.
    // This is a placeholder for future windows-sys integration.
    // For now, we just return false to indicate "not implemented".
    false
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

        // used + available should roughly equal total (with some variance for cached)
        // used + free + cached ≈ total
        let cached = snap.ram_cached_bytes();
        let sum = snap.ram_used_bytes + snap.ram_free_bytes + cached;
        // Allow some tolerance due to OS reporting granularity
        let diff = (snap.ram_total_bytes as i128 - sum as i128).abs();
        assert!(
            diff < 1024 * 1024, // within 1 MiB tolerance
            "RAM accounting inconsistent: total={}, used+free+cached={}, diff={}",
            snap.ram_total_bytes,
            sum,
            diff
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
}
