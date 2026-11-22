//! Unit tests for the (pid, creation_time) suspend registry and NTSTATUS gating.
use super::suspend::{
    apply_results, filetime_to_unix_secs, is_process_suspended, is_suspend_blocklisted, lock,
    matches_suspended_entry, reset_registry, resume_disposition, should_remove_entry, ActionResult,
    SuspendedEntry,
};
use std::sync::Mutex;

static TEST_LOCK: Mutex<()> = Mutex::new(());

fn seed(pid: u32, creation_time: u64) -> SuspendedEntry {
    let entry = SuspendedEntry { pid, creation_time };
    lock().push(entry.clone());
    entry
}

fn contains(pid: u32, creation_time: u64) -> bool {
    lock().iter().any(|e| e.pid == pid && e.creation_time == creation_time)
}

#[test]
fn blocklist_covers_critical_hosts_and_self() {
    let self_pid = std::process::id();
    for name in
        ["dwm.exe", "explorer.exe", "csrss.exe", "audiodg.exe", "TextInputHost.exe", "svchost.exe"]
    {
        assert!(is_suspend_blocklisted(name, "", 100, self_pid), "{name}");
    }
    assert!(is_suspend_blocklisted("notepad.exe", "", self_pid, self_pid));
    assert!(is_suspend_blocklisted("notepad.exe", "", 4, self_pid));
    assert!(!is_suspend_blocklisted("notepad.exe", r"C:\Tools\notepad.exe", 100, self_pid));
}

#[test]
fn filetime_conversion_matches_unix_epoch() {
    const TICKS: u64 = 10_000_000;
    const EPOCH: u64 = 11_644_473_600;
    assert_eq!(filetime_to_unix_secs((EPOCH + 3600) * TICKS), 3600);
    assert_eq!(filetime_to_unix_secs(0), 0);
}

#[test]
fn action_result_messages_are_stable() {
    assert!(ActionResult::success("ok").ok);
    let err = ActionResult::failure("blocked");
    assert!(!err.ok);
    assert_eq!(err.message, "blocked");
}

#[test]
fn identity_requires_both_pid_and_creation_time() {
    let entry = SuspendedEntry { pid: 7001, creation_time: 456 };
    assert!(matches_suspended_entry(&entry, 7001, 456));
    assert!(!matches_suspended_entry(&entry, 7002, 456));
    assert!(!matches_suspended_entry(&entry, 7001, 457));
    assert!(!matches_suspended_entry(&entry, 7001, 0));
}

#[test]
fn pid_reuse_prunes_stale_record_and_rejects_badge() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset_registry();
    seed(7101, 111);
    assert!(is_process_suspended(7101, 111));
    assert!(!is_process_suspended(7102, 111));
    assert!(contains(7101, 111), "record with the same identity must stay");
    assert!(is_process_suspended(7101, 111));

    // Windows reuses PID 7101 for an unrelated process with a new creation time.
    assert!(!is_process_suspended(7101, 999));
    assert!(!contains(7101, 111), "stale (pid, old_ct) record must be pruned");
    assert!(lock().is_empty());
}

#[test]
fn zero_creation_time_never_confirms_identity_but_keeps_record() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset_registry();
    seed(7201, 222);
    assert!(!is_process_suspended(7201, 0));
    assert!(contains(7201, 222), "unknown identity must not forget a frozen process");
}

#[test]
fn negative_ntstatus_retains_entry_and_reports_failure() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset_registry();
    let entry = seed(7301, 333);
    let counts = apply_results(&[(entry.clone(), Some(333), Some(-1), false)]);
    assert_eq!(counts, (0, 1));
    assert!(contains(7301, 333), "failed NtResumeProcess must keep the entry");
    assert!(!should_remove_entry(Some(-1), false));
    assert!(!resume_disposition(&entry, Some(333), Some(-1073741822), false));
}

#[test]
fn success_and_exited_remove_entries_and_report_true_counts() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset_registry();
    let ok = seed(7401, 444);
    seed(7402, 555);
    let counts = apply_results(&[
        (ok.clone(), Some(444), Some(0), false),
        (SuspendedEntry { pid: 7402, creation_time: 555 }, None, None, true),
    ]);
    assert_eq!(counts, (1, 0));
    assert!(!contains(7401, 444));
    assert!(!contains(7402, 555));
    assert!(should_remove_entry(Some(0), false));
    assert!(should_remove_entry(Some(1), false));
    assert!(should_remove_entry(None, true));
    assert!(resume_disposition(&ok, Some(444), Some(1), false));
}

#[test]
fn reused_pid_never_resumes_unrelated_process() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset_registry();
    let stale = seed(7501, 666);
    let counts = apply_results(&[(stale.clone(), Some(777), None, false)]);
    assert_eq!(counts, (0, 0));
    assert!(!contains(7501, 666), "stale entry must be dropped, not resumed");
    assert!(resume_disposition(&stale, Some(777), None, false));
    assert!(!resume_disposition(&stale, None, None, false));
}

#[test]
fn failed_open_or_query_retains_entry_and_increments_failed() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset_registry();
    let entry = seed(7601, 888);
    let counts = apply_results(&[(entry.clone(), None, None, false)]);
    assert_eq!(counts, (0, 1));
    assert!(contains(7601, 888), "access denied must keep the frozen record");
    assert!(!resume_disposition(&entry, None, None, false));
}

#[test]
fn confirmed_exit_drops_entry_without_counting_failure() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset_registry();
    let entry = seed(7701, 999);
    let counts = apply_results(&[(entry.clone(), None, None, true)]);
    assert_eq!(counts, (0, 0));
    assert!(!contains(7701, 999), "confirmed exit must be dropped cleanly");
    assert!(resume_disposition(&entry, None, None, true));
}
