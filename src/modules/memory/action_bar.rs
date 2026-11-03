//! Non-blocking memory action state: working-set trim, open location, and end task.

use crate::modules::memory::actions::ReclaimReport;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant};

pub const STATUS_TTL: Duration = Duration::from_secs(8);
pub const CONFIRM_TTL: Duration = Duration::from_secs(4);
/// Tooltip for the row actions while nothing is selected.
pub const NO_SELECTION_HINT: &str = "Click any process row in the table below to select it";
/// Tooltip when the PID safety gate rejects the selected process.
pub const PROTECTED_HINT: &str =
    "OS / protected processes cannot be modified by PULSE (safety gate)";
/// Tooltip while a background action is still running.
pub const BUSY_HINT: &str = "Another process action is still running - please wait";

/// Formats the honest trim result shown under the action bar.
pub fn format_trim_status(r: &ReclaimReport) -> String {
    let s = r.skipped_protected + r.skipped_access_denied;
    format!(
        "Working-set trim: {:.1} MB across {} procs ({} skipped)",
        r.reclaimed_bytes as f64 / 1_048_576.0,
        r.trimmed_count,
        s
    )
}

/// Shared state for the action bar: background result channel and confirmation timers.
#[derive(Debug, Default)]
pub struct MemoryActionBarState {
    pending_rx: Option<Receiver<String>>,
    last_status: Option<(Instant, String)>,
    confirm_kill_group: Option<(String, Instant)>,
}

impl MemoryActionBarState {
    pub fn set_status(&mut self, now: Instant, text: String) {
        self.last_status = Some((now, text));
    }
    pub fn status_text(&self, now: Instant) -> Option<&str> {
        self.last_status
            .as_ref()
            .and_then(|(at, t)| (now.duration_since(*at) < STATUS_TTL).then_some(t.as_str()))
    }
    pub fn begin_confirm_kill(&mut self, g: &str, now: Instant) {
        self.confirm_kill_group = Some((g.to_owned(), now));
    }
    pub fn cancel_confirm(&mut self) {
        self.confirm_kill_group = None;
    }
    pub fn confirm_kill_active(&self, g: &str, now: Instant) -> bool {
        self.confirm_kill_group
            .as_ref()
            .is_some_and(|(gr, at)| gr == g && now.duration_since(*at) < CONFIRM_TTL)
    }
    /// Polls the background worker without blocking the UI thread.
    pub(crate) fn poll(&mut self, now: Instant) {
        if let Some(rx) = self.pending_rx.take() {
            match rx.try_recv() {
                Ok(t) => self.last_status = Some((now, t)),
                Err(TryRecvError::Empty) => self.pending_rx = Some(rx),
                Err(TryRecvError::Disconnected) => {
                    self.last_status = Some((now, "Action thread stopped".to_owned()))
                }
            }
        }
        if self.last_status.as_ref().is_some_and(|(at, _)| now.duration_since(*at) >= STATUS_TTL) {
            self.last_status = None;
        }
        if self
            .confirm_kill_group
            .as_ref()
            .is_some_and(|(_, at)| now.duration_since(*at) >= CONFIRM_TTL)
        {
            self.confirm_kill_group = None;
        }
    }
    /// True while a background action is in flight; buttons stay disabled.
    pub fn is_busy(&self) -> bool {
        self.pending_rx.is_some()
    }
    /// Runs a job on a worker thread and reports its status back to the UI.
    /// Refuses to start a second job while one is still pending.
    pub(crate) fn spawn(&mut self, job: impl FnOnce() -> String + Send + 'static) {
        if self.pending_rx.is_some() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(job());
        });
        self.pending_rx = Some(rx);
    }
}
