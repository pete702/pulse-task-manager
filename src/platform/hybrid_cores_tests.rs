//! Tests for hybrid mask computation and affinity-target rejection.
use super::*;

#[test]
fn homogeneous_cpu_is_not_hybrid() {
    let cores = [(0u8, 0x0000_00FFusize), (0, 0x0000_FF00)];
    assert_eq!(compute_hybrid_masks(&cores, 16), None);
    assert_eq!(compute_hybrid_masks(&[], 8), None);
}

#[test]
fn hybrid_cpu_splits_p_and_e_masks() {
    let cores = [(1u8, 0x0000_00FFusize), (0, 0x0000_FF00)];
    assert_eq!(compute_hybrid_masks(&cores, 16), Some((0x00FF, 0xFF00)));
}

#[test]
fn more_than_64_logical_cpus_is_rejected() {
    let cores = [(1u8, u64::MAX as usize), (0, 1usize)];
    assert_eq!(compute_hybrid_masks(&cores, 0), None);
    assert_eq!(compute_hybrid_masks(&cores, 65), None);
}

#[test]
fn self_and_system_pids_are_rejected() {
    let self_pid = std::process::id();
    let r = set_process_affinity_verified(self_pid, 0, r"C:\Apps\pulse.exe", 0x1, "P-Cores");
    assert!(!r.ok);
    let r = set_process_affinity_verified(4, 0, r"C:\Windows\System32\csrss.exe", 0x1, "P-Cores");
    assert!(!r.ok);
}

#[test]
fn protected_system_path_is_rejected() {
    let r =
        set_process_affinity_verified(1234, 0, r"C:\Windows\System32\csrss.exe", 0x1, "P-Cores");
    assert!(!r.ok);
    assert!(r.message.contains("protected"));
}

#[test]
fn empty_target_loop_is_honest() {
    let (ok, msg) = apply_affinity_to_targets(&[], 0x1, 0x2, AffinityMode::PCores);
    assert_eq!(ok, 0);
    assert_eq!(msg, "Affinity unchanged (no processes)");
}

#[test]
fn zero_start_time_is_rejected_fail_closed() {
    let r = set_process_affinity_verified(1234, 0, r"C:\Apps\app.exe", 0x1, "P-Cores");
    assert!(!r.ok);
    assert!(r.message.contains("Identity unavailable"));
}

#[test]
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
fn unaligned_record_parsing_checks_bounds() {
    let need = std::mem::size_of::<LogicalProcessorInfoEx>();
    let mut words = vec![0u64; (need + 8).div_ceil(8)];
    let record = LogicalProcessorInfoEx {
        relationship: REL,
        size: need as u32,
        processor: ProcessorRelationship {
            flags: 0,
            efficiency_class: 1,
            reserved: [0; 20],
            group_count: 1,
            group_mask: [GroupAffinity { mask: 0xFF, group: 0, reserved: [0; 3] }],
        },
    };
    let mut shifted = words.clone();
    unsafe {
        let at = (shifted.as_mut_ptr() as *mut u8).add(1) as *mut LogicalProcessorInfoEx;
        std::ptr::write_unaligned(at, record);
    }
    let got = read_record(&shifted, 1);
    assert_eq!(got.processor.efficiency_class, 1);
    assert_eq!(got.processor.group_mask[0].mask, 0xFF);
    unsafe {
        let at = words.as_mut_ptr() as *mut LogicalProcessorInfoEx;
        std::ptr::write_unaligned(at, record);
    }
    let (cores, bad) = parse_core_records(&words, need);
    assert!(!bad);
    assert_eq!(cores, vec![(1u8, 0xFFusize)]);
    unsafe {
        let at = (words.as_mut_ptr() as *mut u8).add(4) as *mut u32;
        std::ptr::write_unaligned(at, need as u32 + 1);
    }
    let (_, bad) = parse_core_records(&words, need);
    assert!(bad);
}
