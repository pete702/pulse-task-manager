//! Snapshot publication plumbing for the telemetry worker.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

use crate::core::{LatestSlot, SnapshotTx, SystemSnapshot};

/// Tracks the previous minimized state for restore wakeup detection.
static WAS_MINIMIZED: AtomicBool = AtomicBool::new(false);

/// Wakeup callback type: invoked after each snapshot publish to trigger UI repaint.
pub type Wakeup = Arc<dyn Fn() + Send + Sync + 'static>;

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
    WAS_MINIMIZED.swap(minimized, Ordering::Relaxed);
    latest.store(snapshot.clone());
    match tx.try_send(snapshot) {
        Ok(()) => {
            if !minimized {
                wakeup();
            }
            PublishResult::Sent
        }
        Err(mpsc::TrySendError::Full(_)) => PublishResult::Full,
        Err(mpsc::TrySendError::Disconnected(_)) => PublishResult::Disconnected,
    }
}
