//! CPU collector tests.

use super::{pid_contains, CpuCollector};
use crate::modules::cpu::table_sort::contains_ci;

#[test]
fn disk_rates_from_prev_first_sample_none() {
    assert_eq!(CpuCollector::disk_rates_from_prev(None, 1_000_000, 2_000_000, 1.0), (0.0, 0.0));
}
#[test]
fn disk_rates_from_prev_second_sample_finite() {
    let (r, w) = CpuCollector::disk_rates_from_prev(Some((0, 0)), 1_048_576, 2_097_152, 1.0);
    assert!((r - 1.0).abs() < 1e-6);
    assert!((w - 2.0).abs() < 1e-6);
}
#[test]
fn disk_rates_from_prev_counter_rollback_zeroes() {
    assert_eq!(CpuCollector::disk_rates_from_prev(Some((500, 500)), 100, 100, 1.0), (0.0, 0.0));
}
#[test]
fn pid_reuse_with_new_start_time_is_new() {
    let mut c = CpuCollector::new();
    c.seen_pids.insert((100, 1));
    assert!(!c.is_new(100, 1));
    assert!(c.is_new(100, 2));
    assert!(c.is_new(101, 1));
}
#[test]
fn disk_rate_uses_production_helper() {
    assert_eq!(CpuCollector::disk_rate(200, 100, 1.0), 100);
    assert_eq!(CpuCollector::disk_rate(100, 200, 1.0), 0);
    assert_eq!(CpuCollector::disk_rate(200, 100, 0.0), 0);
    assert_eq!(CpuCollector::disk_rate(1_000_000, 0, 2.0), 500_000);
}
#[test]
fn contains_ci_ascii_and_unicode() {
    assert!(contains_ci("Chrome.exe", "chrome"));
    assert!(contains_ci("CHROME.EXE", "chrome"));
    assert!(!contains_ci("chrome.exe", "edge"));
    assert!(contains_ci("", ""));
    let upper = "\u{041f}\u{0440}\u{043e}\u{0446}\u{0435}\u{0441}";
    let lower = "\u{043f}\u{0440}\u{043e}\u{0446}\u{0435}\u{0441}";
    assert!(contains_ci(upper, lower));
    assert!(contains_ci(lower, upper));
    assert!(!contains_ci(lower, "other"));
}
#[test]
fn pid_contains_edges() {
    assert!(pid_contains(0, "0"));
    assert!(pid_contains(99999, "999"));
    assert!(!pid_contains(12, "123"));
    assert!(!pid_contains(100, "9"));
    assert!(!pid_contains(99999, ""));
    assert!(!pid_contains(1234, "12a"));
}

mod qos_action {
    use crate::modules::cpu::qos_action::{
        build_throttling_masks, evict_oldest, is_system_process, open_process_file_location,
        resolve_restore_priority, target_for, ORIG_PRIORITY,
    };
    use std::sync::Mutex;
    use std::time::Instant;

    #[test]
    fn test_is_system_process_and_masks() {
        assert!(is_system_process("System") && is_system_process("svchost.exe"));
        assert!(!is_system_process("chrome.exe"));
        let (on_c, on_s) = build_throttling_masks(true);
        let (off_c, off_s) = build_throttling_masks(false);
        assert!(on_c != 0 && on_s != 0 && off_c != 0 && off_s == 0);
    }
    #[cfg(target_os = "windows")]
    #[test]
    fn test_target_for_rejects_none_on_windows() {
        assert!(target_for(123, "app.exe", None, None).is_none());
        assert!(target_for(123, "app.exe", Some(42), None).is_some());
    }
    #[test]
    fn open_location_rejects_system_pid_4() {
        assert!(open_process_file_location(4, "System", None, None).is_err());
    }
    #[test]
    fn priority_restore_and_oldest_eviction() {
        let key = (999_001u32, 777u64);
        let slot = ORIG_PRIORITY.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
        slot.lock().unwrap_or_else(|e| e.into_inner()).remove(&key);
        assert!(resolve_restore_priority(key).is_err());
        slot.lock().unwrap_or_else(|e| e.into_inner()).insert(key, (0x20, Instant::now()));
        assert_eq!(resolve_restore_priority(key).unwrap(), 0x20);
        slot.lock().unwrap_or_else(|e| e.into_inner()).remove(&key);
        let mut g = slot.lock().unwrap_or_else(|e| e.into_inner());
        g.clear();
        let base = Instant::now();
        for i in 0..512u32 {
            g.insert((10_000 + i, 1), (i, base + std::time::Duration::from_secs(i as u64)));
        }
        evict_oldest(&mut g, (20_000, 1));
        assert!(!g.contains_key(&(10_000, 1)) && g.contains_key(&(10_001, 1)));
        g.clear();
    }
}

#[cfg(windows)]
#[test]
fn live_eco_qos_enable_and_restore_roundtrip() {
    use crate::modules::memory::safe_handle::SafeHandle;
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};
    use windows_sys::Win32::System::Threading::{GetPriorityClass, IDLE_PRIORITY_CLASS};

    #[allow(unsafe_code)]
    fn query_prio(pid: u32) -> u32 {
        let handle = SafeHandle::open_process(pid).expect("open child for priority query");
        unsafe { GetPriorityClass(handle.as_raw()) }
    }

    struct ChildGuard(std::process::Child);
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    let child = Command::new("ping")
        .args(["-n", "30", "127.0.0.1"])
        .creation_flags(0x08000000)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn ping child");
    let guard = ChildGuard(child);
    let pid = guard.0.id();
    let initial_prio = query_prio(pid);
    assert_ne!(initial_prio, 0);
    let ct = crate::modules::memory::query_creation_time_by_pid(pid).expect("child creation time");
    let exe = crate::modules::memory::query_process_exe_path(pid);
    crate::modules::cpu::qos_action::set_process_eco_qos(
        pid,
        "ping.exe",
        Some(ct),
        exe.clone(),
        true,
    )
    .expect("enable EcoQoS on child");
    assert_eq!(crate::modules::cpu::qos_action::is_process_eco_qos(pid), Some(true));
    assert_eq!(query_prio(pid), IDLE_PRIORITY_CLASS);
    crate::modules::cpu::qos_action::set_process_eco_qos(pid, "ping.exe", Some(ct), exe, false)
        .expect("disable EcoQoS and restore priority on child");
    assert_eq!(crate::modules::cpu::qos_action::is_process_eco_qos(pid), Some(false));
    assert_eq!(query_prio(pid), initial_prio);
}
