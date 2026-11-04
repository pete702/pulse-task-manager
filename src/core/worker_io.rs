//! Snapshot publication plumbing for the telemetry worker.

use std::sync::{mpsc, Arc};

use crate::core::{LatestSlot, SnapshotTx, SystemSnapshot};

/// Wakeup callback type: invoked after each snapshot publish to trigger UI repaint.
pub type Wakeup = Arc<dyn Fn() + Send + Sync + 'static>;

/// Result of publishing one snapshot to the channel.
pub enum PublishResult {
    Sent,
    Full,
    Disconnected,
}

/// Stores the freshest snapshot in the shared slot, then forwards it if the channel has room.
/// Calls the wakeup callback on successful send to trigger UI repaint.
pub fn publish(
    tx: &SnapshotTx,
    latest: &LatestSlot,
    snapshot: Arc<SystemSnapshot>,
    wakeup: &Wakeup,
) -> PublishResult {
    latest.store(snapshot.clone());
    match tx.try_send(snapshot) {
        Ok(()) => {
            wakeup();
            PublishResult::Sent
        }
        Err(mpsc::TrySendError::Full(_)) => PublishResult::Full,
        Err(mpsc::TrySendError::Disconnected(_)) => PublishResult::Disconnected,
    }
}
