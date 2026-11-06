//! CPU snapshot collector split from collector for the 280-line guard.
use std::sync::Arc;

use sysinfo::{CpuRefreshKind, System};

use crate::modules::cpu::model::CpuSnapshot;
use crate::modules::cpu::win32_cpu::{read_cpu_brand, read_cpu_frequencies, warmup_cpu_refresh};

/// Collects current system CPU state into a `CpuSnapshot`.
/// Live MHz and brand come from Win32 OnceLock. See worker_loop.
pub fn collect_cpu(sys: &mut System) -> CpuSnapshot {
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
    let (base_mhz, current_mhz) = read_cpu_frequencies();
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
        raw_processes: Arc::new([]),
        process_count: 0,
    }
}

/// Two-pass CPU warmup: primes sysinfo deltas so first read is real.
/// Call once at worker startup before entering the main loop.
pub fn warmup_cpu(sys: &mut System) {
    warmup_cpu_refresh(sys);
}
