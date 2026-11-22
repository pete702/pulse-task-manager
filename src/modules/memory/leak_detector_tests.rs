//! Tests for the bounded memory-leak slope tracker.
use super::*;

const MB: u64 = 1024 * 1024;

fn climb(steps: usize, step_mb: u64) -> Vec<Checkpoint> {
    (0..steps).map(|i| (i as u64 * SAMPLE_INTERVAL_SECS, (100 + i as u64 * step_mb) * MB)).collect()
}

#[test]
fn steady_five_minute_climb_is_a_leak() {
    let samples = climb(11, 7);
    assert_eq!(evaluate_leak_slope(&samples), Some(14));
    assert_eq!(format_leak_badge(14), "[LEAK +14 MB/m]");
}

#[test]
fn short_spike_is_not_a_leak() {
    let samples = climb(10, 25);
    assert!(samples.last().unwrap().0 - samples.first().unwrap().0 < MIN_SPAN_SECS);
    assert_eq!(evaluate_leak_slope(&samples), None);
}

#[test]
fn sawtooth_with_gc_drops_is_not_a_leak() {
    let samples: Vec<Checkpoint> = (0..12)
        .map(|i| {
            let high = 100 + (i / 2) * 24;
            let bytes = if i % 2 == 0 { high } else { high - 18 };
            (i * SAMPLE_INTERVAL_SECS, bytes * MB)
        })
        .collect();
    assert_eq!(evaluate_leak_slope(&samples), None);
}

#[test]
fn early_burst_then_plateau_is_not_a_leak() {
    let samples: Vec<Checkpoint> = (0..12)
        .map(|i| {
            let bytes = if i == 0 { 100 * MB } else { 250 * MB };
            (i * SAMPLE_INTERVAL_SECS, bytes)
        })
        .collect();
    assert_eq!(evaluate_leak_slope(&samples), None);
}

#[test]
fn pid_reuse_does_not_inherit_ram_history() {
    history().lock().unwrap_or_else(|e| e.into_inner()).clear();
    record_at(&[(7, 100, 100 * MB)], 0);
    record_at(&[(7, 200, 110 * MB)], 30);
    {
        let map = history().lock().unwrap_or_else(|e| e.into_inner());
        assert!(!map.contains_key(&(7, 100)));
        assert_eq!(map[&(7, 200)].len(), 1);
    }
    assert_eq!(get_process_leak_mb_per_min(7, 100), None);
}

#[test]
fn sampling_interval_and_pruning_are_bounded() {
    history().lock().unwrap_or_else(|e| e.into_inner()).clear();
    record_at(&[(1, 1000, 100 * MB), (2, 2000, 50 * MB)], 0);
    record_at(&[(1, 1000, 200 * MB), (2, 2000, 60 * MB)], 10);
    assert_eq!(history().lock().unwrap_or_else(|e| e.into_inner())[&(1, 1000)].len(), 1);
    record_at(&[(1, 1000, 200 * MB), (2, 2000, 60 * MB)], 30);
    assert_eq!(history().lock().unwrap_or_else(|e| e.into_inner())[&(1, 1000)].len(), 2);
    record_at(&[(2, 2000, 60 * MB)], 60);
    assert!(!history().lock().unwrap_or_else(|e| e.into_inner()).contains_key(&(1, 1000)));

    let many: Vec<(u32, u64, u64)> =
        (0..200).map(|i| (i, i as u64 + 1, (i as u64 + 1) * MB)).collect();
    record_at(&many, 90);
    assert!(history().lock().unwrap_or_else(|e| e.into_inner()).len() <= MAX_PROCESSES);
}
