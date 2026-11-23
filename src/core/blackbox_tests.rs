//! Unit tests for the 60-second stutter black box.
use super::*;

fn sample(i: usize) -> BlackboxSample {
    BlackboxSample {
        seq: i as u64,
        timestamp: format!("12:00:{:02}", i % 60),
        cpu_pct: 10.0,
        peak_core_pct: 10.0,
        ram_pct: 40.0,
        commit_pct: 50.0,
        disk_mb_s: 5.0,
        ping_ms: 12.0,
        jitter_ms: 3.0,
        culprit_name: String::new(),
        verdict: None,
    }
}

fn input<'a>(time: &'a str) -> VerdictInput<'a> {
    VerdictInput {
        time,
        peak_core_pct: 50.0,
        commit_pct: 70.0,
        disk_mb_s: 5.0,
        jitter_ms: 4.0,
        packet_loss_pct: 0.0,
        disk_spike: false,
        mem_pressure: false,
        cpu_culprit: "",
        disk_culprit: "",
        mem_culprit: "",
    }
}

#[test]
fn ring_buffer_caps_at_sixty_samples() {
    state().lock().unwrap_or_else(|e| e.into_inner()).samples.clear();
    for i in 0..BLACKBOX_CAPACITY + 7 {
        push_sample(sample(i));
    }
    assert_eq!(sample_count(), BLACKBOX_CAPACITY);
}

#[test]
fn verdict_disk_signal() {
    let mut i = input("14:22:08");
    i.disk_mb_s = 410.0;
    i.disk_culprit = "MsMpEng.exe";
    let v = evaluate_stutter_verdict(&i);
    assert_eq!(v.as_deref(), Some("14:22:08 | Freeze risk: MsMpEng.exe (Disk 410 MB/s)"));
}

#[test]
fn verdict_disk_spike_signal() {
    let mut i = input("01:02:03");
    i.disk_spike = true;
    i.disk_culprit = "MsMpEng.exe";
    assert!(evaluate_stutter_verdict(&i).is_some_and(|s| s.contains("Freeze risk")));
}

#[test]
fn verdict_memory_signal() {
    let mut i = input("01:02:03");
    i.commit_pct = 93.0;
    i.mem_pressure = true;
    i.mem_culprit = "chrome.exe";
    assert!(evaluate_stutter_verdict(&i).is_some_and(|s| s.contains("Memory thrash: chrome.exe")));
}

#[test]
fn verdict_core_signal() {
    let mut i = input("01:02:03");
    i.peak_core_pct = 96.0;
    i.cpu_culprit = "game.exe";
    assert_eq!(evaluate_stutter_verdict(&i).as_deref(), Some("01:02:03 | 1-Core 96%: game.exe"));
}

#[test]
fn verdict_network_signal() {
    let mut i = input("01:02:03");
    i.jitter_ms = 31.0;
    assert!(evaluate_stutter_verdict(&i).is_some_and(|s| s.contains("Net spike: jitter 31 ms")));
}

#[test]
fn verdict_none_when_nominal() {
    assert_eq!(evaluate_stutter_verdict(&input("01:02:03")), None);
}

#[test]
fn verdict_expires_when_its_sample_leaves_the_ring() {
    let mut bb = state().lock().unwrap_or_else(|e| e.into_inner());
    bb.samples.clear();
    bb.last_verdict = None;
    drop(bb);
    let mut flagged = sample(0);
    flagged.verdict = Some("12:00:00 | Freeze risk: disk.exe (Disk 500 MB/s)".to_string());
    push_sample(flagged);
    assert!(latest_verdict().is_some());
    for i in 1..=BLACKBOX_CAPACITY {
        push_sample(sample(i));
    }
    assert_eq!(latest_verdict(), None);
}

#[test]
fn disk_spike_below_threshold_uses_disk_culprit() {
    let mut i = input("01:02:03");
    i.disk_spike = true;
    i.disk_mb_s = 80.0;
    i.cpu_culprit = "game.exe";
    i.disk_culprit = "MsMpEng.exe";
    let kind = verdict_kind(&i);
    assert_eq!(kind, Some(VerdictKind::Disk));
    assert_eq!(culprit_for_verdict(kind, "MsMpEng.exe", "chrome.exe", "game.exe"), "MsMpEng.exe");
}

#[test]
fn network_verdict_never_borrows_the_cpu_culprit() {
    let mut i = input("01:02:03");
    i.jitter_ms = 40.0;
    i.cpu_culprit = "game.exe";
    let kind = verdict_kind(&i);
    assert_eq!(kind, Some(VerdictKind::Network));
    assert_eq!(culprit_for_verdict(kind, "disk.exe", "chrome.exe", "game.exe"), "");
}

#[test]
fn restarted_worker_sample_at_id_one_is_accepted() {
    {
        let mut bb = state().lock().unwrap_or_else(|e| e.into_inner());
        bb.samples.clear();
        bb.last_key = 0;
        bb.last_verdict = None;
    }
    let snap = |uptime, id| {
        SystemSnapshot::new_with_jitter(
            uptime,
            1.0,
            Some(crate::modules::memory::model::MemorySnapshot {
                snapshot_id: id,
                ..Default::default()
            }),
            None,
            0,
        )
    };
    let first = snap(100, 1);
    observe(Some(&first));
    assert_eq!(sample_count(), 1);
    let restarted = snap(200, 1);
    observe(Some(&restarted));
    assert_eq!(sample_count(), 2);
}
