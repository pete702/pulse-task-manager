//! OS CPU collector for PULSE: gathers real CPU metrics via sysinfo
//! and computes per-process CPU percentages in a single pass.

use std::sync::Arc;

use sysinfo::{CpuRefreshKind, ProcessRefreshKind, System};

use crate::modules::cpu::model::{CpuSnapshot, RawProcessCpuRow};
use crate::modules::cpu::win32_cpu::{read_cpu_brand, read_cpu_frequencies, warmup_cpu_refresh};

/// Collects current system CPU state into a `CpuSnapshot`.
/// Uses Win32 CallNtPowerInformation for live frequency (CurrentMhz) and base frequency (MaxMhz).
pub fn collect_cpu(sys: &mut System) -> CpuSnapshot {
    // Cheap refresh: live MHz and brand come from Win32 OnceLock. See worker_loop.
    sys.refresh_cpu_specifics(CpuRefreshKind::new().with_cpu_usage());

    let cpus = sys.cpus();
    let logical_cores = cpus.len();
    let mut per_core_pct = Vec::with_capacity(logical_cores);
    let mut total_usage = 0.0f32;
    let mut hottest_core_idx = 0;
    let mut hottest_core_pct = 0.0f32;

    for (idx, cpu) in cpus.iter().enumerate() {
        let usage = cpu.cpu_usage();
        per_core_pct.push(usage);
        total_usage += usage;
        if usage > hottest_core_pct {
            hottest_core_pct = usage;
            hottest_core_idx = idx;
        }
    }

    let total_pct = (total_usage / logical_cores.max(1) as f32).clamp(0.0, 100.0);

    // Live frequency via Win32 (CurrentMhz reflects Turbo Boost / SpeedShift)
    // Base frequency (MaxMhz) for reference
    let (base_mhz, current_mhz) = read_cpu_frequencies();

    // Clean CPU brand string
    let brand = read_cpu_brand(sys);

    CpuSnapshot {
        total_pct,
        per_core_pct: per_core_pct.into(),
        hottest_core_idx,
        hottest_core_pct,
        frequency_mhz: current_mhz,
        base_mhz,
        logical_cores,
        brand,
        processes: Arc::new([]),
        raw_processes: Arc::new([]),
        process_count: 0,
    }
}

/// Collects process CPU information for the process list view.
///
/// When `refresh_processes` is `false`, sysinfo process iteration is skipped
/// completely and an empty row vector is returned.
///
/// This function uses the SAME sysinfo refresh pass as memory collection
/// by including `.with_cpu()` in the ProcessRefreshKind.
/// Returns raw rows only; CpuTableState formats from raw rows so unused
/// display items are never pre-built in the worker.
pub fn collect_process_cpu(
    sys: &mut System,
    logical_cores: usize,
    refresh_processes: bool,
) -> Vec<RawProcessCpuRow> {
    if !refresh_processes {
        return Vec::new();
    }

    // Use without_exe() to avoid querying executable paths for all 200+ processes
    // (exe paths are queried on-demand via query_process_exe_path when "Open Location" is clicked)
    // Include .with_cpu() to get per-process CPU usage in the SAME pass as memory
    sys.refresh_processes_specifics(
        ProcessRefreshKind::new().with_memory().with_cpu().without_exe(),
    );

    let mut raw_rows: Vec<RawProcessCpuRow> = sys
        .processes()
        .iter()
        .map(|(pid, process)| {
            let cpu_usage = (process.cpu_usage() / logical_cores.max(1) as f32).clamp(0.0, 100.0);
            let ram = process.memory();
            RawProcessCpuRow {
                pid: pid.as_u32(),
                name: Arc::from(process.name()),
                cpu_usage,
                ram_bytes: ram,
                exe_path: None,      // Will be resolved on-demand
                creation_time: None, // Will be resolved on-demand via Win32
                start_time: process.start_time(),
            }
        })
        .collect();

    // Sort raw rows by CPU descending (same as processed items)
    raw_rows.sort_by(|a, b| b.cpu_usage.total_cmp(&a.cpu_usage));

    raw_rows
}

/// Zero-alloc PID substring check using a stack buffer.
pub(crate) fn pid_contains(pid: u32, needle: &str) -> bool {
    let mut buf = [0u8; 10];
    let mut pos = 10;
    let mut n = pid;
    loop {
        pos -= 1;
        buf[pos] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    std::str::from_utf8(&buf[pos..]).unwrap_or("").contains(needle)
}

/// Two-pass CPU warmup: primes sysinfo's internal deltas so first real read returns correct CPU%.
/// Call once at worker startup before entering the main loop.
pub fn warmup_cpu(sys: &mut System) {
    warmup_cpu_refresh(sys);
}
