//! OS memory collector for PULSE: gathers real RAM/Swap metrics via sysinfo
//! and Windows kernel metrics via K32GetPerformanceInfo.

use std::sync::Arc;

use sysinfo::{ProcessRefreshKind, System};

use crate::modules::memory::constants::MAX_SANE_VIRTUAL_BYTES;
use crate::modules::memory::delta_tracker::{ProcessDeltaTracker, ProcessMemoryScan};
use crate::modules::memory::model::{MemorySnapshot, ProcessSample};
#[cfg(target_os = "windows")]
use crate::modules::memory::win32_api::{collect_windows_kernel_metrics, KernelMetrics};

/// Collects current system memory state into a `MemorySnapshot`.
/// `scan_duration_ms` is left at the default here; the worker loop stamps it
/// once with the full telemetry tick duration so the metric stays honest.
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
        if kernel_metrics.system_cache_bytes > 0 {
            snap.ram_cached_bytes_override = kernel_metrics.system_cache_bytes;
        }
        snap.kernel_metrics_valid = kernel_metrics.is_valid;
    }

    snap
}

/// Collects process memory information for the process list view.
///
/// When `refresh_processes` is `false`, sysinfo process iteration is skipped
/// completely and an empty row vector is returned.
///
/// Reuses `sys.processes()` when called after `collect_process_cpu` on the
/// same worker tick (the CPU pass already refreshed with
/// `.with_memory().with_cpu().without_exe()`); falls back to refreshing
/// memory only when the process list is empty.
pub fn collect_process_memory(
    sys: &mut System,
    total_ram: u64,
    current_ram_pct: f32,
    delta_tracker: &mut ProcessDeltaTracker,
    refresh_processes: bool,
) -> (ProcessMemoryScan, bool) {
    if !refresh_processes {
        return ((Vec::new(), None), false);
    }

    // Skip second sysinfo scan when worker_loop already refreshed with
    // .with_memory().with_cpu().without_exe() on the same tick.
    if sys.processes().is_empty() {
        // Use without_exe() to avoid querying executable paths for all 200+ processes
        // (exe paths are queried on-demand via query_process_exe_path when "Open Location" is clicked)
        sys.refresh_processes_specifics(ProcessRefreshKind::new().with_memory().without_exe());
    }

    let processes: Vec<ProcessSample> = sys
        .processes()
        .iter()
        .map(|(pid, process)| {
            let ram = process.memory();
            let mut virt = process.virtual_memory();
            if virt >= MAX_SANE_VIRTUAL_BYTES {
                virt = ram;
            }
            // exe_path will be None since we use without_exe()
            // It will be resolved on-demand via query_process_exe_path when needed
            let exe_path: Option<Arc<str>> = None;
            (pid.as_u32(), Arc::from(process.name()), ram, virt, exe_path, process.start_time())
        })
        .collect();

    const HEAVY_THRESHOLD: u64 = 2560_u64 << 20; // 2.5 GiB
    let has_heavy_process = processes.iter().any(|(_, _, ram, _, _, _)| *ram >= HEAVY_THRESHOLD);
    let processes_with_delta = delta_tracker.track(&processes);
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
