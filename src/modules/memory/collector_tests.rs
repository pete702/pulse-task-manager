//! Unit tests for memory collector (sysinfo + Windows kernel metrics).

#[cfg(test)]
mod tests {
    use crate::modules::memory::collector::{
        collect_memory, collect_process_memory, ProcessDeltaTracker,
    };
    use crate::modules::memory::win32_api::{own_working_set_mb, trim_own_working_set};
    use sysinfo::System;

    #[test]
    fn test_real_os_memory_collection() {
        let mut sys = System::new_all();
        let snap = collect_memory(&mut sys);
        assert!(snap.ram_total_bytes > 0, "RAM total > 0");
        assert!(snap.ram_used_bytes > 0, "RAM used > 0");
        assert!(snap.ram_available_bytes > 0, "RAM available > 0");
        let usage = snap.ram_usage_percent();
        assert!(usage > 0.0 && usage <= 100.0, "RAM usage% in (0,100], got {}", usage);
    }

    #[test]
    fn test_collect_memory_fields_populated() {
        let mut sys = System::new_all();
        let snap = collect_memory(&mut sys);
        assert_ne!(snap.ram_total_bytes, 0);
        assert_ne!(snap.ram_used_bytes, 0);
        assert_ne!(snap.ram_available_bytes, 0);
        if snap.swap_total_bytes > 0 {
            assert!(snap.swap_used_bytes <= snap.swap_total_bytes);
            assert!(snap.swap_free_bytes <= snap.swap_total_bytes);
        }
        #[cfg(target_os = "windows")]
        {
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
        let sum_used_free = snap.ram_used_bytes + snap.ram_free_bytes;
        assert!(sum_used_free <= snap.ram_total_bytes * 2, "used+free <= 2x total");
        let cached = snap.ram_cached_bytes();
        assert!(cached <= snap.ram_total_bytes, "cached <= total");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_kernel_metrics_non_zero() {
        let mut sys = System::new_all();
        let snap = collect_memory(&mut sys);
        assert!(snap.system_threads > 0, "system_threads > 0");
        assert!(snap.system_processes > 0, "system_processes > 0");
        assert!(snap.system_handles > 0, "system_handles > 0");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_real_kernel_cache_metrics() {
        let mut sys = System::new_all();
        let snap = collect_memory(&mut sys);
        assert!(snap.ram_cached_bytes() > 0, "ram_cached_bytes > 0");
        assert!(snap.kernel_paged_bytes > 0, "kernel_paged_bytes > 0");
        assert!(snap.kernel_nonpaged_bytes > 0, "kernel_nonpaged_bytes > 0");
        assert!(snap.system_threads > 0, "system_threads > 0");
    }

    #[test]
    fn test_collect_process_memory() {
        let mut sys = System::new_all();
        let total_ram = sys.total_memory();
        let mut delta_tracker = ProcessDeltaTracker::new();
        let (processes, _spike) =
            collect_process_memory(&mut sys, total_ram, 0.0, &mut delta_tracker);
        assert!(!processes.is_empty(), "at least one process");
        for (pid, name, ram_bytes, virtual_bytes, _delta, _exe) in &processes {
            if *pid == 0 {
                continue;
            }
            assert!(*pid > 0, "PID > 0");
            assert!(!name.is_empty(), "name not empty");
            assert!(*ram_bytes > 0 || *virtual_bytes > 0, "has memory");
        }
        #[cfg(target_os = "windows")]
        assert!(
            processes.iter().any(|row| row.5.is_some()),
            "at least one process exposes an exe path"
        );
    }

    #[test]
    fn test_trim_own_working_set() {
        let _ = trim_own_working_set();
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_own_working_set_mb() {
        let mb = own_working_set_mb();
        assert!(mb > 0.0, "own RSS > 0 MiB, got {mb}");
        assert!(mb < 10_000.0, "own RSS sane, got {mb}");
    }
}
