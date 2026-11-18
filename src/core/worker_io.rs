//! Snapshot publication plumbing for the telemetry worker.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{mpsc, Arc};

use crate::core::{LatestSlot, SnapshotTx, SystemSnapshot};

/// Wakeup callback type: invoked after each snapshot publish to trigger UI repaint.
pub type Wakeup = Arc<dyn Fn() + Send + Sync + 'static>;

/// Global count of snapshots dropped due to a saturated sync channel.
static DROPPED_SNAPSHOTS: AtomicU64 = AtomicU64::new(0);

/// Global count of recovered collector panics.
static PANIC_RECOVERIES: AtomicU32 = AtomicU32::new(0);

/// Returns the number of snapshots dropped because the channel was full.
#[cfg(test)]
pub fn dropped_snapshots() -> u64 {
    DROPPED_SNAPSHOTS.load(Ordering::Relaxed)
}

/// Resets the dropped-snapshot counter (test helper).
#[cfg(test)]
pub fn reset_dropped_snapshots() {
    DROPPED_SNAPSHOTS.store(0, Ordering::Relaxed);
}

/// Returns the number of recovered collector panics.
#[cfg(test)]
pub fn panic_recoveries() -> u32 {
    PANIC_RECOVERIES.load(Ordering::Relaxed)
}

/// Increments the recovered-panic counter.
pub fn record_panic_recovery() {
    PANIC_RECOVERIES.fetch_add(1, Ordering::Relaxed);
}

/// Resets the panic-recovery counter (test helper).
#[cfg(test)]
pub fn reset_panic_recoveries() {
    PANIC_RECOVERIES.store(0, Ordering::Relaxed);
}

/// Result of publishing one snapshot to the channel.
pub enum PublishResult {
    Sent,
    Full,
    Disconnected,
}

/// Stores the freshest snapshot in the shared slot, then forwards it if the channel has room.
/// Calls the wakeup callback on successful send to trigger UI repaint (unless minimized).
pub fn publish(
    tx: &SnapshotTx,
    latest: &LatestSlot,
    snapshot: Arc<SystemSnapshot>,
    wakeup: &Wakeup,
    minimized: bool,
) -> PublishResult {
    latest.store(snapshot.clone());
    match tx.try_send(snapshot) {
        Ok(()) => {
            if !minimized {
                wakeup();
            }
            PublishResult::Sent
        }
        Err(mpsc::TrySendError::Full(_)) => {
            DROPPED_SNAPSHOTS.fetch_add(1, Ordering::Relaxed);
            (wakeup)();
            PublishResult::Full
        }
        Err(mpsc::TrySendError::Disconnected(_)) => PublishResult::Disconnected,
    }
}

/// Pure helper: late-only scheduler overshoot in milliseconds.
pub fn compute_scheduler_overshoot_ms(
    elapsed_ms: u32,
    target_sleep_ms: u32,
    was_forced_unpark: bool,
) -> u32 {
    if was_forced_unpark || elapsed_ms <= target_sleep_ms {
        0
    } else {
        elapsed_ms.saturating_sub(target_sleep_ms)
    }
}

/// Runs a collector tick closure, recovering from panics and incrementing counters.
#[cfg(test)]
pub fn catch_collector_tick<F>(f: F, counter: &mut u32) -> bool
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(()) => false,
        Err(_) => {
            *counter = counter.saturating_add(1);
            PANIC_RECOVERIES.fetch_add(1, Ordering::Relaxed);
            eprintln!("pulse worker: collector tick panicked; recovered (count={})", *counter);
            true
        }
    }
}

/// Alias for `catch_collector_tick`.
#[cfg(test)]
pub fn recover_collector_panic<F>(f: F, counter: &mut u32) -> bool
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    catch_collector_tick(f, counter)
}

/// Alias for `catch_collector_tick`.
#[cfg(test)]
pub fn catch_unwind_collector<F>(f: F, counter: &mut u32) -> bool
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    catch_collector_tick(f, counter)
}
