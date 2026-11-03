//! OS memory collector for PULSE: gathers real RAM/Swap metrics via sysinfo
//! and Windows kernel metrics via K32GetPerformanceInfo.
//! Also provides process memory collection and self-memory trimming.

use sysinfo::{ProcessRefreshKind, System, UpdateKind};

use crate::modules::memory::delta_tracker::{ProcessDeltaTracker, ProcessMemoryScan};
use crate::modules::memory::model::{MemorySnapshot, ProcessSample};
use crate::modules::memory::win32_api::{collect_windows_kernel_metrics, KernelMetrics};

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
        let kernel_metrics: KernelMetrics = collect_windows_kernel_metrics();
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
/// Returns a vector of (PID, name, RAM bytes, virtual bytes, delta bytes/sec) tuples,
/// optionally detects memory spike events (RAM >= 85%), and a precomputed
/// `has_heavy_process` flag (any process >= 2.5 GiB) for O(1) status bar heuristic.
#[allow(dead_code)]
pub fn collect_process_memory(
    sys: &mut System,
    total_ram: u64,
    current_ram_pct: f32,
    delta_tracker: &mut ProcessDeltaTracker,
) -> (ProcessMemoryScan, bool) {
    // Refresh process list with memory info only; skip exe path re-query on each cycle
    sys.refresh_processes_specifics(
        ProcessRefreshKind::new().with_memory().with_exe(UpdateKind::OnlyIfNotSet),
    );

    let processes: Vec<ProcessSample> = sys
        .processes()
        .iter()
        .map(|(pid, process)| {
            let ram = process.memory();
            let mut virt = process.virtual_memory();
            // Normalize CFG virtual memory reservation (>= 1 TiB) to actual RAM
            if virt >= (1_u64 << 40) {
                virt = ram;
            }
            let exe_path =
                process.exe().and_then(|p| p.to_str()).filter(|s| !s.is_empty()).map(str::to_owned);
            (pid.as_u32(), process.name().to_string(), ram, virt, exe_path)
        })
        .collect();

    // Precompute heavy process flag for O(1) status bar access
    const HEAVY_THRESHOLD: u64 = 2560_u64 << 20; // 2.5 GiB
    let has_heavy_process = processes.iter().any(|(_, _, ram, _, _)| *ram >= HEAVY_THRESHOLD);

    // Track deltas
    let processes_with_delta = delta_tracker.track(&processes);

    // Detect memory spike using system-wide RAM usage (not the last process).
    let spike = crate::modules::memory::process_model::detect_memory_spike(
        &processes_with_delta,
        total_ram,
        current_ram_pct,
    );

    ((processes_with_delta, spike), has_heavy_process)
}

// Re-export Win32 API functions for backward compatibility
#[allow(unused_imports)]
pub use crate::modules::memory::win32_api::{own_working_set_mb, trim_own_working_set};
